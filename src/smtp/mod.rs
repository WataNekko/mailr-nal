mod commands;
mod extensions;
mod response;

use core::fmt::Debug;
use embedded_nal::{nb::block, AddrType, Dns, SocketAddr, TcpClientStack, TcpError};
use enumset::EnumSet;

pub use self::commands::ClientId;
use self::commands::Ehlo;
use self::extensions::{EhloInfo, SmtpExtension};
use self::response::{ReplyLine, ResponseError, ResponseParser};
use crate::{
    auth::Credential,
    io::{BufWriter, TcpStream, WithBuf},
};

pub struct SmtpClient;

impl SmtpClient {
    pub fn new<'a, T>(stack: &'a mut T, buffer: &'a mut [u8]) -> SmtpClientConnector<'a, T>
    where
        T: TcpClientStack,
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
    T: TcpClientStack,
{
    stack: &'a mut T,
    buffer: &'a mut [u8],
    auth: Option<Credential<'a>>,
    client_id: Option<ClientId<'a>>,
}

impl<'a, T> SmtpClientConnector<'a, T>
where
    T: TcpClientStack,
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

        let stream = TcpStream::new(stack, remote.into()).map_err(|e| ConnectError::IoError(e))?;
        let mut stream = WithBuf(stream, buffer);

        Self::server_greeting(&mut stream)?;

        let client_id = client_id.unwrap_or(ClientId::localhost());
        let ehlo_info = Self::ehlo(&mut stream, client_id)?;

        Ok(SmtpClientSession { stream, ehlo_info })
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

    fn server_greeting(stream: &mut WithBuf<TcpStream<T>>) -> Result<(), ConnectError<T::Error>> {
        ResponseParser::new(stream).expect_code(b"220")?;
        Ok(())
    }

    fn ehlo(
        stream: &mut WithBuf<TcpStream<T>>,
        client_id: ClientId,
    ) -> Result<EhloInfo, ConnectError<T::Error>> {
        {
            let mut stream = BufWriter::from(&mut *stream);
            write!(stream, "{}", Ehlo(client_id))?;
        }

        let mut response = ResponseParser::new(stream);

        {
            // skip first greeting line
            let ReplyLine {
                code: b"250",
                has_next: true,
                ..
            } = response.next_line()?
            else {
                return Err(ConnectError::UnexpectedResponse);
            };
        }

        // process extensions
        let mut extensions = EnumSet::new();

        loop {
            let ReplyLine {
                code: b"250",
                text,
                has_next,
            } = response.next_line()?
            else {
                return Err(ConnectError::UnexpectedResponse);
            };

            let mut words = text.split(' ');

            let ext = words.next().ok_or(ConnectError::UnexpectedResponse)?;

            extensions |= match ext {
                "AUTH" => EnumSet::from_iter(words.map(|mech| match mech {
                    "PLAIN" => SmtpExtension::AuthPlain.into(),
                    "LOGIN" => SmtpExtension::AuthLogin.into(),
                    _ => EnumSet::empty(),
                })),
                _ => EnumSet::empty(),
            };

            if !has_next {
                break;
            }
        }

        Ok(EhloInfo { extensions })
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

impl<E> From<E> for ConnectError<E>
where
    E: TcpError,
{
    fn from(value: E) -> Self {
        Self::IoError(value)
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
    stream: WithBuf<'a, TcpStream<'a, T>>,
    ehlo_info: EhloInfo,
}

impl<'a, T> Debug for SmtpClientSession<'a, T>
where
    T: TcpClientStack,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SmtpClientSession")
    }
}
