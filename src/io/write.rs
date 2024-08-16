use embedded_nal::nb;

use crate::nb_fut::{ready, NbFuture};

pub trait Write {
    type Error;

    fn write(&mut self, buffer: &[u8]) -> nb::Result<usize, Self::Error>;

    #[inline]
    fn write_all(&mut self, mut buffer: &[u8]) -> impl NbFuture<Output = (), Error = Self::Error> {
        move || {
            while !buffer.is_empty() {
                let written = ready!(self.write(buffer))?;
                buffer = &buffer[written..];
            }
            Ok(())
        }
    }
}

pub struct BufWriter<'a, W>
where
    W: Write,
{
    writer: W,
    buffer: &'a mut [u8],
    filled: usize,
}

impl<'a, W> BufWriter<'a, W>
where
    W: Write,
{
    pub fn new(writer: W, buffer: &'a mut [u8]) -> Self {
        Self {
            writer,
            buffer,
            filled: 0,
        }
    }

    pub fn flush(&mut self) -> impl NbFuture<Output = (), Error = W::Error> + '_ {
        let mut fut = self.writer.write_all(&self.buffer[..self.filled]);
        let filled = &mut self.filled;
        move || {
            fut.poll()?;
            *filled = 0;
            Ok(())
        }
    }

    fn write_to_buffer(&mut self, data: &[u8]) {
        self.buffer[self.filled..self.filled + data.len()].copy_from_slice(data);
        self.filled += data.len();
    }

    pub fn write(
        &'a mut self,
        data: &'a [u8],
    ) -> impl NbFuture<Output = (), Error = W::Error> + 'a {
        move || {
            if self.filled + data.len() > self.buffer.len() {
                self.flush().block()?;
            }

            if data.len() >= self.buffer.len() {
                self.writer.write_all(data).block()?;
            } else {
                self.write_to_buffer(data);
            }
            Ok(())
        }
        // enum State<F, W> {
        //     Start,
        //     FlushFut(F),
        //     Write,
        //     WriteFut(W),
        // }
        // let mut state = State::Start;

        // move || loop {
        //     match state {
        //         State::Start => {
        //             state = if self.filled + data.len() > self.buffer.len() {
        //                 State::FlushFut(self.flush())
        //             } else {
        //                 State::Write
        //             };
        //         }
        //         State::FlushFut(ref mut f) => {
        //             f.poll()?;
        //             state = State::Write;
        //         }
        //         State::Write => {
        //             if data.len() >= self.buffer.len() {
        //                 state = State::WriteFut(self.writer.write_all(data));
        //             } else {
        //                 self.write_to_buffer(data);
        //                 return Ok(());
        //             }
        //         }
        //         State::WriteFut(ref mut w) => w.poll()?,
        //     }
        // }
    }
}

impl<'a, W> Drop for BufWriter<'a, W>
where
    W: Write,
{
    fn drop(&mut self) {
        let _ = self.flush().block();
    }
}
