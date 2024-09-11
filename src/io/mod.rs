mod read;
pub use read::*;

mod write;
pub use write::*;

mod stream;
pub use stream::*;

#[repr(C)]
pub struct WithBuf<T, B: AsMut<[u8]>>(pub T, pub B);

impl<'a, R, B> From<&'a mut WithBuf<R, B>> for BufReader<'a, R>
where
    R: Read,
    B: AsMut<[u8]>,
{
    fn from(value: &'a mut WithBuf<R, B>) -> Self {
        Self::new(&mut value.0, value.1.as_mut())
    }
}

impl<'a, R, B> From<&'a mut WithBuf<R, B>> for BufWriter<'a, R>
where
    R: Write,
    B: AsMut<[u8]>,
{
    fn from(value: &'a mut WithBuf<R, B>) -> Self {
        Self::new(&mut value.0, value.1.as_mut())
    }
}
