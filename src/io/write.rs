use core::fmt::Debug;

use embedded_nal::nb::{self, block};

pub trait Write {
    type Error: Debug;

    fn write(&mut self, buffer: &[u8]) -> nb::Result<usize, Self::Error>;

    // FIXME: Blocking for simplicity
    fn write_all(&mut self, mut buffer: &[u8]) -> Result<(), Self::Error> {
        while !buffer.is_empty() {
            let written = block!(self.write(buffer))?;
            buffer = &buffer[written..];
        }
        Ok(())
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

    // FIXME: Blocking for now for simplicity
    pub fn flush(&mut self) -> Result<(), W::Error> {
        self.writer.write_all(&self.buffer[..self.filled])?;
        self.filled = 0;
        Ok(())
    }

    fn write_to_buffer(&mut self, data: &[u8]) {
        self.buffer[self.filled..self.filled + data.len()].copy_from_slice(data);
        self.filled += data.len();
    }

    // FIXME: Blocking for now for simplicity
    pub fn write(&mut self, data: &[u8]) -> Result<(), W::Error> {
        if self.filled + data.len() > self.buffer.len() {
            self.flush()?;
        }

        if data.len() >= self.buffer.len() {
            self.writer.write_all(data)?;
        } else {
            self.write_to_buffer(data);
        }
        Ok(())
    }
}

impl<'a, W> Drop for BufWriter<'a, W>
where
    W: Write,
{
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
