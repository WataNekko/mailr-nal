mod read;
pub use read::*;

mod write;
pub use write::*;

mod stream;
pub use stream::*;

pub struct WithBuf<'a, T>(pub T, pub &'a mut [u8]);

impl<'a, R> From<&'a mut WithBuf<'_, R>> for BufReader<'a, R>
where
    R: Read,
{
    fn from(value: &'a mut WithBuf<'_, R>) -> Self {
        Self::new(&mut value.0, value.1)
    }
}

impl<'a, R> From<&'a mut WithBuf<'_, R>> for BufWriter<'a, R>
where
    R: Write,
{
    fn from(value: &'a mut WithBuf<'_, R>) -> Self {
        Self::new(&mut value.0, value.1)
    }
}
