use embedded_nal::nb;

pub trait NbFuture {
    type Output;
    type Error: core::fmt::Debug;

    fn poll(&mut self) -> nb::Result<Self::Output, Self::Error>;

    fn block(&mut self) -> Result<Self::Output, Self::Error> {
        loop {
            match self.poll() {
                Err(nb::Error::WouldBlock) => {} // busy wait until not blocked
                Err(nb::Error::Other(e)) => return Err(e),
                Ok(o) => return Ok(o),
            }
        }
    }
}

impl<T, O, E> NbFuture for T
where
    T: FnMut() -> nb::Result<O, E>,
    E: core::fmt::Debug,
{
    type Output = O;
    type Error = E;

    #[inline]
    fn poll(&mut self) -> nb::Result<Self::Output, Self::Error> {
        self()
    }
}

macro_rules! ready {
    ($e:expr) => {{
        use embedded_nal::nb;

        match $e {
            Err(nb::Error::WouldBlock) => return Err(nb::Error::WouldBlock),
            Err(nb::Error::Other(e)) => Err(e),
            Ok(o) => Ok(o),
        }
    }};
}

pub(crate) use ready;

#[cfg(test)]
mod test {
    use core::convert::Infallible;

    use nb::block;

    use super::*;

    #[test]
    fn simple_fut() {
        struct AddFut(i32, i32);
        impl NbFuture for AddFut {
            type Output = i32;
            type Error = Infallible;

            fn poll(&mut self) -> nb::Result<Self::Output, Self::Error> {
                Ok(self.0 + self.1)
            }
        }

        let mut fut = AddFut(1, 2);
        let res = fut.poll();
        assert_eq!(res, Ok(3), "Future should resolve immediately");
    }

    #[test]
    fn multi_state_fut() {
        enum State<'a> {
            GetInput,
            ProcessData(&'a str),
            PrintOutput(&'a str),
        }

        struct MultiStateFut<'a> {
            state: State<'a>,
            output_buffer: &'a mut [u8],
        }

        impl<'a> MultiStateFut<'a> {
            fn new(output_buffer: &'a mut [u8]) -> Self {
                Self {
                    state: State::GetInput,
                    output_buffer,
                }
            }
        }

        #[derive(Debug, PartialEq)]
        struct ProcessingError;

        impl<'a> NbFuture for MultiStateFut<'a> {
            type Output = usize;
            type Error = ProcessingError;

            fn poll(&mut self) -> nb::Result<Self::Output, Self::Error> {
                match self.state {
                    State::GetInput => {
                        let input = "!!Hello!!!!";
                        self.state = State::ProcessData(input);
                        Err(nb::Error::WouldBlock)
                    }
                    State::ProcessData(s) => {
                        let processed = s.trim_matches('!');
                        if processed.len() == s.len() {
                            return Err(nb::Error::Other(ProcessingError));
                        }

                        self.state = State::PrintOutput(processed);
                        Err(nb::Error::WouldBlock)
                    }
                    State::PrintOutput(s) => {
                        let mut end = 0;

                        let print = b"print: ";
                        end += print.len();
                        self.output_buffer[..end].copy_from_slice(print);

                        let s = s.as_bytes();
                        self.output_buffer[end..end + s.len()].copy_from_slice(s);
                        end += s.len();

                        Ok(end)
                    }
                }
            }
        }

        let mut buf = [0; 32];
        let mut fut = MultiStateFut::new(&mut buf);
        assert_eq!(fut.poll(), Err(nb::Error::WouldBlock));
        assert_eq!(fut.poll(), Err(nb::Error::WouldBlock));

        let res = fut.poll();
        let n = res.expect("future done");
        assert_eq!(&buf[..n], b"print: Hello");
    }

    #[test]
    fn nb_function_boilerplate() {
        fn add(a: i32, b: i32) -> impl NbFuture<Output = i32, Error = Infallible> {
            struct AddFut(i32, i32);
            impl NbFuture for AddFut {
                type Output = i32;
                type Error = Infallible;

                fn poll(&mut self) -> nb::Result<Self::Output, Self::Error> {
                    Ok(self.0 + self.1)
                }
            }
            AddFut(a, b)
        }

        let mut fut = add(1, 2);
        let res = fut.poll();
        assert_eq!(res, Ok(3), "Future should resolve immediately");
    }

    #[test]
    fn nb_function_returning_closure() {
        fn add(a: i32, b: i32) -> impl NbFuture<Output = i32, Error = Infallible> {
            move || Ok(a + b)
        }

        let mut fut = add(1, 2);
        let res = fut.poll();
        assert_eq!(res, Ok(3), "Future should resolve immediately");

        fn block_forever() -> impl NbFuture<Output = (), Error = Infallible> {
            || Err(nb::Error::WouldBlock)
        }

        let mut fut = block_forever();
        assert_eq!(
            fut.poll(),
            Err(nb::Error::WouldBlock),
            "Closure should be able to return WouldBlock"
        );

        fn multi_state(a: i32) -> impl NbFuture<Output = i32, Error = Infallible> {
            enum State {
                First,
                Second(i32),
            }
            let mut state = State::First;

            move || match state {
                State::First => {
                    state = State::Second(a * a);
                    Err(nb::Error::WouldBlock)
                }
                State::Second(i) => Ok(i / 2),
            }
        }

        let mut fut = multi_state(4);
        assert_eq!(fut.poll(), Err(nb::Error::WouldBlock)); // 4 * 4 = 16
        assert_eq!(fut.poll(), Ok(8)); // 16 / 2 = 8
    }

    #[test]
    fn chaining_multi_state_fut() {
        #[derive(Debug, PartialEq)]
        struct ProcessingError;

        /// (data.len() ^ data.len()) ^ 2 - (data.len() ^ data.len())
        fn process(data: &str) -> impl NbFuture<Output = f32, Error = ProcessingError> + '_ {
            enum State<'a, P>
            where
                P: NbFuture<Output = f32, Error = ProcessingError>,
            {
                Unprocessed(&'a str),
                Len(usize),
                Subprocessing1(P),
                Subprocessing2(f32, P),
            }
            let mut state = State::Unprocessed(data);

            move || match state {
                State::Unprocessed(s) => {
                    state = State::Len(s.len());
                    Err(nb::Error::WouldBlock)
                }
                State::Len(u) => {
                    let i = u as _;
                    let fut = subprocess(i, i.unsigned_abs());
                    state = State::Subprocessing1(fut);
                    Err(nb::Error::WouldBlock)
                }
                State::Subprocessing1(ref mut fut) => {
                    let res = fut.poll()?;
                    let fut = subprocess(res as _, 2);
                    state = State::Subprocessing2(res, fut);
                    Err(nb::Error::WouldBlock)
                }
                State::Subprocessing2(f, ref mut fut) => {
                    let res = fut.poll()?;
                    Ok(res - f)
                }
            }
        }

        /// (data ^ exp) as f32
        fn subprocess(data: i32, exp: u32) -> impl NbFuture<Output = f32, Error = ProcessingError> {
            enum State {
                Unprocessed(i32),
                Powed(i32),
            }
            let mut state = State::Unprocessed(data);

            move || match state {
                State::Unprocessed(i) => {
                    let powed = i
                        .checked_pow(exp)
                        .ok_or(nb::Error::Other(ProcessingError))?;
                    state = State::Powed(powed);
                    Err(nb::Error::WouldBlock)
                }
                State::Powed(i) => Ok(i as _),
            }
        }

        let mut fut = process("hello"); // len() == 5
        let res = fut.block(); // (5 ^ 5) ^ 2 - (5 ^ 5) == 9762500
        assert_eq!(res, Ok(9762500_f32));

        let mut fut = process("01234567890123456789"); // len() == 20
        let res = fut.block(); // 20 ^ 20 = overflow
        assert_eq!(res, Err(ProcessingError));
    }

    #[test]
    fn ready_macro() {
        #[derive(Debug)]
        struct Negative;

        fn poll_inc(i: i32) -> nb::Result<i32, Negative> {
            if i.is_negative() {
                Err(nb::Error::Other(Negative))
            } else {
                Ok(i + 1)
            }
        }
        fn poll_compute() -> nb::Result<i32, Infallible> {
            let res = ready!(poll_inc(3)).expect("not negative");
            Ok(res * 4)
        }

        let res = block!(poll_compute()).expect("infallible");
        assert_eq!(res, 16);
    }
}
