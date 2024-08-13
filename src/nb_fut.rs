use embedded_nal::nb;

pub trait NbFuture {
    type Output;
    type Error: core::fmt::Debug;

    fn poll(&mut self) -> nb::Result<Self::Output, Self::Error>;
}

#[cfg(test)]
mod test {
    use core::convert::Infallible;

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
}
