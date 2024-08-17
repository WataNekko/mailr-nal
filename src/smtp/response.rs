use core::{fmt::Debug, str};

use crate::io::{BufReader, BufReaderError, Read};

pub struct ResponseParser<'a, R>(BufReader<'a, R>)
where
    R: Read;

impl<'a, R> ResponseParser<'a, R>
where
    R: Read,
{
    pub fn new(reader: &'a mut R, buffer: &'a mut [u8]) -> Self {
        Self(BufReader::new(reader, buffer))
    }

    pub fn expect_code(mut self, code: &[u8]) -> Result<(), ResponseError<'a, R::Error>> {
        loop {
            let line = match self.next_line() {
                Ok(line) => line,
                Err(e) => {
                    // SAFETY: transmute to cast away the lifetime. By returning early the error, Rust
                    // thinks we've borrowed self for this whole block. But if it's not an error, we should
                    // be safe to borrow the value again since self's lifetime hasn't left the block.
                    // The function signature's explicit lifetime ensure this is safe.
                    return Err(unsafe { core::mem::transmute(e) });
                }
            };

            if line.code != code {
                return Err(ResponseError::ReplyCodeError(
                    // SAFETY: same as above, transmute to cast the lifetime away.
                    unsafe { core::mem::transmute(line.code) },
                ));
            }

            if !line.to_be_continued {
                break;
            }
        }
        Ok(())
    }

    /// Return the next reply line and whether the reply continues (expecting another line)
    fn next_line(&mut self) -> Result<ReplyLine, ResponseError<R::Error>> {
        let line = self.0.read_line()?.as_bytes();

        let (code, text) = line.split_at_checked(3).ok_or(ResponseError::FormatError)?;
        let (text, to_be_continued) = text
            .split_first()
            .map(|(&first, rest)| (rest, first == b'-'))
            .unwrap_or((b"", false));

        Ok(ReplyLine {
            code,
            text: str::from_utf8(text).map_err(|_| ResponseError::FormatError)?,
            to_be_continued,
        })
    }
}

pub enum ResponseError<'a, E>
where
    E: Debug,
{
    ReplyCodeError(&'a [u8]),
    ReadError(E),
    NoMem,
    FormatError,
}

impl<'a, E> From<BufReaderError<'a, E>> for ResponseError<'a, E>
where
    E: Debug,
{
    fn from(value: BufReaderError<'a, E>) -> Self {
        match value {
            BufReaderError::FullBuffer(_) => Self::NoMem,
            BufReaderError::ReaderError(e) => Self::ReadError(e),
            BufReaderError::DecodeFailed(_, _) => Self::FormatError,
        }
    }
}

struct ReplyLine<'a> {
    code: &'a [u8],
    text: &'a str,
    to_be_continued: bool,
}
