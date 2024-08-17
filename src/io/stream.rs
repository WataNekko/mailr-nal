use core::mem::ManuallyDrop;

use embedded_nal::{nb, SocketAddr, TcpClientStack};

use super::{Read, Write};

pub struct TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    stack: &'a mut T,
    socket: ManuallyDrop<T::TcpSocket>,
}

impl<'a, T> TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    pub fn new(stack: &'a mut T, remote: SocketAddr) -> Result<Self, T::Error> {
        // FIXME: Blocking connect for now for simplicity
        let mut socket = stack.socket()?;
        nb::block!(stack.connect(&mut socket, remote))?;

        Ok(Self {
            stack,
            socket: ManuallyDrop::new(socket),
        })
    }

    #[inline]
    fn internal_close(&mut self) -> Result<(), T::Error> {
        self.stack.close(
            // SAFETY: should be safe as the socket would never be touched again after close
            unsafe { ManuallyDrop::take(&mut self.socket) },
        )
    }

    /// Explicitly close the socket, returning an error if any. Dropping the stream would do the same
    /// thing, except the result would be ignored.
    pub fn close(self) -> Result<(), T::Error> {
        let mut me = ManuallyDrop::new(self);
        me.internal_close()
    }
}

impl<'a, T> Drop for TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    fn drop(&mut self) {
        let _ = self.internal_close();
    }
}

impl<'a, T> Read for TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    type Error = T::Error;

    fn read(&mut self, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        self.stack.receive(&mut self.socket, buffer)
    }
}

impl<'a, T> Write for TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    type Error = T::Error;

    fn write(&mut self, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        self.stack.send(&mut self.socket, buffer)
    }
}
