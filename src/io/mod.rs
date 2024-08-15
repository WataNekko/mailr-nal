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
}

impl<'a, T> Reader for TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    type Error = T::Error;

    fn read(&mut self, buffer: &mut [u8]) -> nb::Result<usize, Self::Error> {
        self.stack.receive(&mut self.socket, buffer)
    }
}

impl<'a, T> Writer for TcpStream<'a, T>
where
    T: TcpClientStack + 'a,
{
    type Error = T::Error;

    fn write(&mut self, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        self.stack.send(&mut self.socket, buffer)
    }
}
