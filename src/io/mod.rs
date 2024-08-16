use core::mem::ManuallyDrop;

use embedded_nal::{nb, TcpClientStack};

mod read;
pub use read::*;

mod write;
pub use write::*;

pub struct TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    stack: &'a mut T,
    socket: T::TcpSocket,
}

impl<'a, T> TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    pub fn new(stack: &'a mut T, socket: T::TcpSocket) -> Self {
        Self { stack, socket }
    }

    #[inline]
    fn internal_close(&mut self) -> Result<(), T::Error> {
        self.stack.close(
            // SAFETY: socket is never used again after close so it would be safe to move
            // out of the mut ref.
            unsafe { core::ptr::read(&self.socket) },
        )
    }

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
