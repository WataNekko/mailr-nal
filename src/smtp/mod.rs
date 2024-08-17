mod response;

use embedded_nal::{nb::block, Dns, SocketAddr, TcpClientStack};
use response::{ResponseError, ResponseParser};

use crate::{
    auth::Credential,
    io::{BufReader, TcpStream},
};

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
}

impl<'a, T> SmtpClientConnector<'a, T>
where
    T: TcpClientStack + 'a,
{
    pub fn with_auth(mut self, cred: Option<Credential<'a>>) -> Self {
        self.auth = cred;
        self
    }

    // FIXME: Blocking for simplicity
    pub fn connect(
        self,
        remote: impl Into<SocketAddr>,
    ) -> Result<SmtpClientSession<'a, T>, ConnectError<T::Error>> {
        let Self {
            stack,
            buffer: buf,
            auth,
        } = self;

        let mut stream =
            TcpStream::new(stack, remote.into()).map_err(|e| ConnectError::IoError(e))?;

        ResponseParser::new(&mut stream, buf).expect_code(b"220")?;

        Ok(SmtpClientSession { stream, buf })
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
        let addr = block!(dns.get_host_by_name(hostname, embedded_nal::AddrType::Either))
            .map_err(|e| ConnectHostnameError::DnsError(e))?;

        Ok(self.connect((addr, port))?)
    }
}

#[derive(Debug)]
pub enum ConnectError<E> {
    IoError(E),
    NoMem,
    AuthFailed,
    AuthUnsupported,
    UnexpectedResponse,
}

impl<'a, E> From<ResponseError<'a, E>> for ConnectError<E> {
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
pub enum ConnectHostnameError<DE, E> {
    DnsError(DE),
    ConnectError(ConnectError<E>),
}

impl<DE, E> From<ConnectError<E>> for ConnectHostnameError<DE, E> {
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
