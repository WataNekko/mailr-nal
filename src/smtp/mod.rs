mod commands;
mod response;

use core::fmt::Debug;
use embedded_nal::{nb::block, AddrType, Dns, SocketAddr, TcpClientStack, TcpError};

pub use self::commands::ClientId;
use self::response::{ResponseError, ResponseParser};
use crate::{auth::Credential, io::TcpStream};

pub struct SmtpClient;

impl SmtpClient {
    pub fn new<'a, T>(stack: &'a mut T, buffer: &'a mut [u8]) -> SmtpClientConnector<'a, T>
    where
        T: TcpClientStack + 'a,
    {
        SmtpClientConnector {
            stack,
            buffer,
            auth: None,
            client_id: None,
        }
    }
}

pub struct SmtpClientConnector<'a, T>
where
    T: TcpClientStack + 'a,
{
    stack: &'a mut T,
    buffer: &'a mut [u8],
    auth: Option<Credential<'a>>,
    client_id: Option<ClientId<'a>>,
}

impl<'a, T> SmtpClientConnector<'a, T>
where
    T: TcpClientStack + 'a,
{
    pub fn with_auth(mut self, value: impl Into<Option<Credential<'a>>>) -> Self {
        self.auth = value.into();
        self
    }

    pub fn with_client_id(mut self, value: impl Into<Option<ClientId<'a>>>) -> Self {
        self.client_id = value.into();
        self
    }

    // FIXME: Blocking for simplicity
    pub fn connect(
        self,
        remote: impl Into<SocketAddr>,
    ) -> Result<SmtpClientSession<'a, T>, ConnectError<T::Error>> {
        let Self {
            stack,
            buffer,
            auth,
            client_id,
        } = self;

        let mut stream =
            TcpStream::new(stack, remote.into()).map_err(|e| ConnectError::IoError(e))?;

        Self::server_greeting(&mut stream, buffer)?;

        let client_id = client_id.unwrap_or(ClientId::localhost());
        Self::ehlo(&mut stream, buffer, client_id)?;

        Ok(SmtpClientSession {
            stream,
            buf: buffer,
        })
    }

    // FIXME: Blocking for simplicity
    pub fn connect_with_hostname<D>(
        self,
        dns: &mut D,
        hostname: &str,
        port: u16,
    ) -> Result<SmtpClientSession<'a, T>, ConnectHostnameError<D::Error, T::Error>>
    where
        D: Dns,
    {
        let addr = block!(dns.get_host_by_name(hostname, AddrType::Either))
            .map_err(|e| ConnectHostnameError::DnsError(e))?;

        Ok(self.connect((addr, port))?)
    }

    fn server_greeting(
        stream: &mut TcpStream<T>,
        buffer: &mut [u8],
    ) -> Result<(), ConnectError<T::Error>> {
        ResponseParser::new(stream, buffer).expect_code(b"220")?;
        Ok(())
    }

    fn ehlo(
        stream: &mut TcpStream<T>,
        buffer: &mut [u8],
        client_id: ClientId,
    ) -> Result<(), ConnectError<T::Error>> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConnectError<E>
where
    E: TcpError,
{
    IoError(E),
    NoMem,
    AuthFailed,
    AuthUnsupported,
    UnexpectedResponse,
}

impl<'a, E> From<ResponseError<'a, E>> for ConnectError<E>
where
    E: TcpError,
{
    fn from(value: ResponseError<'a, E>) -> Self {
        match value {
            ResponseError::ReplyCodeError(_) | ResponseError::FormatError => {
                Self::UnexpectedResponse
            }
            ResponseError::ReadError(e) => Self::IoError(e),
            ResponseError::NoMem => Self::NoMem,
        }
    }
}

#[derive(Debug)]
pub enum ConnectHostnameError<DE, E>
where
    DE: Debug,
    E: TcpError,
{
    DnsError(DE),
    ConnectError(ConnectError<E>),
}

impl<DE, E> From<ConnectError<E>> for ConnectHostnameError<DE, E>
where
    DE: Debug,
    E: TcpError,
{
    fn from(value: ConnectError<E>) -> Self {
        Self::ConnectError(value)
    }
}

pub struct SmtpClientSession<'a, T>
where
    T: TcpClientStack,
{
    stream: TcpStream<'a, T>,
    buf: &'a [u8],
}

impl<'a, T> Debug for SmtpClientSession<'a, T>
where
    T: TcpClientStack,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SmtpClientSession")
    }
}
