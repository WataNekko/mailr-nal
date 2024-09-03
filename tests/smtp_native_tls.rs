mod tls {
    use convert::SocketAddr;
    use embedded_nal::{TcpClientStack, TcpError};
    use native_tls::{Certificate, HandshakeError, TlsConnector, TlsStream};
    use std::{
        fs,
        io::{Read, Write},
        net::TcpStream,
    };

    mod convert {
        use std::{
            io,
            net::{self, ToSocketAddrs},
            option::IntoIter,
        };

        /// Wrapper around the `std` IP address type that converts to `non_std`
        /// counterpart and vice versa.
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct IpAddr(net::IpAddr);

        impl From<embedded_nal::IpAddr> for IpAddr {
            fn from(input: embedded_nal::IpAddr) -> Self {
                match input {
                    embedded_nal::IpAddr::V4(i) => Self(i.octets().into()),
                    embedded_nal::IpAddr::V6(i) => Self(i.octets().into()),
                }
            }
        }

        impl From<IpAddr> for embedded_nal::IpAddr {
            fn from(s: IpAddr) -> embedded_nal::IpAddr {
                match s.0 {
                    net::IpAddr::V4(i) => i.octets().into(),
                    net::IpAddr::V6(i) => i.octets().into(),
                }
            }
        }

        impl From<IpAddr> for net::IpAddr {
            fn from(s: IpAddr) -> net::IpAddr {
                s.0
            }
        }

        /// Wrapper around the `std` socket address type that converts to `non_std`
        /// counterpart and vice versa.
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct SocketAddr(net::SocketAddr);

        impl ToSocketAddrs for SocketAddr {
            type Iter = IntoIter<net::SocketAddr>;
            fn to_socket_addrs(&self) -> io::Result<IntoIter<net::SocketAddr>> {
                self.0.to_socket_addrs()
            }
        }

        impl From<net::SocketAddr> for SocketAddr {
            fn from(input: net::SocketAddr) -> Self {
                Self(input)
            }
        }

        impl From<SocketAddr> for net::SocketAddr {
            fn from(s: SocketAddr) -> net::SocketAddr {
                s.0
            }
        }

        impl From<embedded_nal::SocketAddr> for SocketAddr {
            fn from(input: embedded_nal::SocketAddr) -> Self {
                Self((IpAddr::from(input.ip()).0, input.port()).into())
            }
        }

        impl From<SocketAddr> for embedded_nal::SocketAddr {
            fn from(s: SocketAddr) -> embedded_nal::SocketAddr {
                (IpAddr(s.0.ip()), s.0.port()).into()
            }
        }
    }

    enum TlsSocketState {
        Unconnected,
        Connected(TlsStream<TcpStream>),
    }
    pub struct TlsSocket(TlsSocketState);

    #[derive(Debug)]
    pub struct TlsError;

    impl TcpError for TlsError {
        fn kind(&self) -> embedded_nal::TcpErrorKind {
            embedded_nal::TcpErrorKind::Other
        }
    }

    impl From<std::io::Error> for TlsError {
        fn from(_value: std::io::Error) -> Self {
            Self
        }
    }

    impl<T> From<HandshakeError<T>> for TlsError {
        fn from(_value: HandshakeError<T>) -> Self {
            Self
        }
    }

    pub struct TlsStack(TlsConnector);

    impl TlsStack {
        pub fn new(custom_cert_path: Option<String>) -> Self {
            let cert = custom_cert_path
                .map(|path| fs::read(path).unwrap())
                .map(|cert| Certificate::from_pem(&cert).unwrap());

            let tls = match cert {
                Some(cert) => TlsConnector::builder().add_root_certificate(cert).build(),
                None => TlsConnector::new(),
            }
            .unwrap();

            Self(tls)
        }
    }

    impl TcpClientStack for TlsStack {
        type TcpSocket = TlsSocket;
        type Error = TlsError;

        fn socket(&mut self) -> Result<Self::TcpSocket, Self::Error> {
            Ok(TlsSocket(TlsSocketState::Unconnected))
        }

        fn connect(
            &mut self,
            socket: &mut Self::TcpSocket,
            remote: embedded_nal::SocketAddr,
        ) -> embedded_nal::nb::Result<(), Self::Error> {
            let stream = TcpStream::connect(SocketAddr::from(remote)).map_err(TlsError::from)?;
            let stream = self
                .0
                .connect("localhost", stream)
                .map_err(TlsError::from)?;
            socket.0 = TlsSocketState::Connected(stream);
            Ok(())
        }

        fn send(
            &mut self,
            socket: &mut Self::TcpSocket,
            buffer: &[u8],
        ) -> embedded_nal::nb::Result<usize, Self::Error> {
            let TlsSocketState::Connected(ref mut conn) = socket.0 else {
                return Err(TlsError)?;
            };
            let n = conn.write(buffer).map_err(TlsError::from)?;
            conn.flush().map_err(TlsError::from)?;
            Ok(n)
        }

        fn receive(
            &mut self,
            socket: &mut Self::TcpSocket,
            buffer: &mut [u8],
        ) -> embedded_nal::nb::Result<usize, Self::Error> {
            let TlsSocketState::Connected(ref mut conn) = socket.0 else {
                return Err(TlsError)?;
            };
            Ok(conn.read(buffer).map_err(TlsError::from)?)
        }

        fn close(&mut self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
            match socket.0 {
                TlsSocketState::Connected(mut conn) => conn.shutdown().map_err(Into::into),
                TlsSocketState::Unconnected => Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod connect {
    use super::*;
    use mailr_nal::{auth::Credential, smtp::SmtpClient};
    use test_common::TestContext;

    #[test]
    fn no_auth() {
        let TestContext {
            tls_port, tls_cert, ..
        } = TestContext::setup();

        let mut stack = tls::TlsStack::new(tls_cert);
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], tls_port))
            .expect("connected without authentication");
    }

    #[test]
    fn auth_success() {
        let TestContext {
            tls_port,
            tls_cert,
            username,
            password,
            ..
        } = TestContext::setup();

        let mut stack = tls::TlsStack::new(tls_cert);
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .with_auth(Some(Credential::new(&username, &password)))
            .connect(([127, 0, 0, 1], tls_port))
            .expect("should authenticate successfully");
    }
}
