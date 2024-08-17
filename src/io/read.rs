use core::{fmt::Debug, ops::Range, str};

use embedded_nal::nb::{self, block};

pub trait Read {
    type Error: Debug;

    fn read(&mut self, buffer: &mut [u8]) -> nb::Result<usize, Self::Error>;
}

pub struct BufReader<'a, R>
where
    R: Read,
{
    reader: &'a mut R,
    buf: &'a mut [u8],
    filled: Range<usize>,
}

impl<'a, R> BufReader<'a, R>
where
    R: Read,
{
    pub fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        Self {
            reader,
            buf,
            filled: 0..0,
        }
    }

    /// Consume `amt` amount of bytes and return the consumed block.
    fn consume(&mut self, amt: usize) -> &[u8] {
        let consumed = &self.buf[self.filled.start..self.filled.start + amt];
        self.filled.start += amt;
        if self.filled.is_empty() {
            self.filled = 0..0
        }
        consumed
    }

    /// Get the available filled block of the buffer without consuming any bytes.
    fn get_filled(&self) -> &[u8] {
        &self.buf[self.filled.clone()]
    }

    /// Fill the buffer with more data by reading from the reader, then return the newly read bytes.
    fn fill_buf(&mut self) -> nb::Result<&[u8], R::Error> {
        let amt = self.reader.read(&mut self.buf[self.filled.end..])?;
        let filled = &self.buf[self.filled.end..self.filled.end + amt];
        self.filled.end += amt;
        Ok(filled)
    }

    /// Return a block of buffered data until the byte where predicate `p` returns true, or EOF is met,
    /// reading from the Reader if there's not enough buffered bytes.
    ///
    /// If the buffer is full without having found the needed byte, `FullBuffer` error is returned with all
    /// the buffered data up to that point returned.
    ///
    // FIXME: Blocking for now for simplicity
    pub fn read_until<P>(&mut self, p: P) -> Result<&[u8], BufReaderError<R::Error>>
    where
        P: FnMut(&u8) -> bool + Copy,
    {
        let mut checked_block_size = 0;
        let mut unchecked_block = self.get_filled();

        loop {
            if let Some(pos) = unchecked_block.iter().position(p) {
                return Ok(self.consume(checked_block_size + pos + 1));
            }

            if self.filled.end >= self.buf.len() {
                if self.filled.start == 0 {
                    return Err(BufReaderError::FullBuffer(self.consume(self.filled.len())));
                }

                // We've filled until the end without finding what we need, but some bytes at the front
                // of the buffer has been consumed (didn't fail with FullBuffer).
                // Let's move the filled block to front of the buffer to attempt to fill the remaining space
                self.buf.copy_within(self.filled.clone(), 0);
                self.filled = 0..self.filled.len();
            }

            checked_block_size = self.filled.len();

            unchecked_block =
                block!(self.fill_buf()).map_err(|e| BufReaderError::ReaderError(e))?;
            if unchecked_block.is_empty() {
                // because we moved the filled block to front earlier, leaving space to be filled,
                // if we get nothing back after a read, EOF must have occurred.
                return Ok(self.consume(checked_block_size));
            }
        }
    }

    // FIXME: Blocking for now for simplicity
    #[inline]
    pub fn read_str_until<P>(&mut self, p: P) -> Result<&str, BufReaderError<R::Error>>
    where
        P: FnMut(&u8) -> bool + Copy,
    {
        self.read_until(p).and_then(|data| {
            str::from_utf8(data).map_err(|e| BufReaderError::DecodeFailed(data, e))
        })
    }

    // FIXME: Blocking for now for simplicity
    pub fn read_line(&mut self) -> Result<&str, BufReaderError<R::Error>> {
        self.read_str_until(|&byte| byte == b'\n')
            .map(|line| line.trim_end_matches("\r\n"))
    }
}

#[derive(Debug, PartialEq)]
pub enum BufReaderError<'a, E>
where
    E: Debug,
{
    FullBuffer(&'a [u8]),
    ReaderError(E),
    DecodeFailed(&'a [u8], str::Utf8Error),
}
