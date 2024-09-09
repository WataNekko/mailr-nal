use core::ptr::{addr_of, addr_of_mut};

use embedded_nal::{nb, SocketAddr, TcpClientStack, TcpError, TcpErrorKind};
use riot_wrappers::{
    error::{NegativeErrorExt, NumericError},
    println,
    socket::UdpEp,
};

#[derive(Debug)]
pub struct TcpNumericError(NumericError);

impl TcpNumericError {
    pub fn number(&self) -> isize {
        self.0.number()
    }

    pub fn again_is_wouldblock(self) -> nb::Error<Self> {
        self.0.again_is_wouldblock().map(|e| Self(e))
    }
}

impl TcpError for TcpNumericError {
    fn kind(&self) -> TcpErrorKind {
        TcpErrorKind::Other
    }
}

impl From<NumericError> for TcpNumericError {
    fn from(value: NumericError) -> Self {
        Self(value)
    }
}

pub(crate) struct SocketAddrWrapper(SocketAddr);

impl From<SocketAddrWrapper> for riot_sys::sock_tcp_ep_t {
    fn from(value: SocketAddrWrapper) -> Self {
        use SocketAddr::*;

        // Constructing via default avoids using the volatile names of the union types
        let mut ep: riot_sys::sock_tcp_ep_t = Default::default();

        ep.family = match value.0 {
            V4(_) => riot_sys::AF_INET as _,
            V6(_) => riot_sys::AF_INET6 as _,
        };
        ep.netif = match value.0 {
            V4(_) => 0,
            V6(a) => a.scope_id() as _,
        };
        ep.port = value.0.port();
        match value.0 {
            V4(a) => {
                ep.addr.ipv4 = a.ip().octets();
            }
            V6(a) => {
                ep.addr.ipv6 = a.ip().octets();
            }
        }

        ep
    }
}

impl From<&riot_sys::sock_tcp_ep_t> for SocketAddrWrapper {
    fn from(value: &riot_sys::sock_tcp_ep_t) -> Self {
        let addr = match value.family as _ {
            riot_sys::AF_INET6 => embedded_nal::SocketAddrV6::new(
                // unsafe: Access to C union whose type was just checked
                unsafe { value.addr.ipv6.into() },
                value.port,
                0,
                value.netif.into(),
            )
            .into(),

            riot_sys::AF_INET => embedded_nal::SocketAddrV4::new(
                // unsafe: Access to C union whose type was just checked
                unsafe { value.addr.ipv4.into() },
                value.port,
            )
            .into(),

            _ => panic!("Endpoint not expressible in embedded_nal"),
        };

        Self(addr)
    }
}

impl From<SocketAddrWrapper> for SocketAddr {
    fn from(value: SocketAddrWrapper) -> Self {
        value.0
    }
}

pub struct SockTcpClientStack(riot_sys::sock_tcp_t);

impl TcpClientStack for SockTcpClientStack {
    type TcpSocket = ();
    type Error = TcpNumericError;

    fn socket(&mut self) -> Result<Self::TcpSocket, Self::Error> {
        Ok(())
    }

    fn connect(
        &mut self,
        socket: &mut Self::TcpSocket,
        remote: SocketAddr,
    ) -> nb::Result<(), Self::Error> {
        let mut remote: riot_sys::sock_tcp_ep_t = SocketAddrWrapper(remote).into();

        unsafe { riot_sys::sock_tcp_connect(&mut self.0, addr_of!(remote), 0, 0) }
            .negative_to_error()
            .map_err(|e| Self::Error::from(e).again_is_wouldblock())?;

        Ok(())
    }

    fn send(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &[u8],
    ) -> nb::Result<usize, Self::Error> {
        todo!()
    }

    fn receive(
        &mut self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        unsafe {
            riot_sys::sock_tcp_read(
                &mut self.0,
                buffer.as_mut_ptr() as _,
                buffer.len().try_into().unwrap_or(u32::MAX),
                0,
            )
        }
        .negative_to_error()
        .map_err(|e| Self::Error::from(e).again_is_wouldblock())
        .map(|n| n as _)
    }

    fn close(&mut self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        todo!()
    }
}
