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
    writer: &'a mut W,
    buffer: &'a mut [u8],
    filled: usize,
}

impl<'a, W> BufWriter<'a, W>
where
    W: Write,
{
    pub fn new(writer: &'a mut W, buffer: &'a mut [u8]) -> Self {
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
        // FIXME: Expose nb async API but blocking impl for now for simplicity
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
