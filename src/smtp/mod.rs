mod commands;
mod extensions;
mod response;

use core::fmt::Debug;
use embedded_nal::{nb::block, AddrType, Dns, SocketAddr, TcpClientStack, TcpError};

pub use self::commands::ClientId;
use self::{
    commands::{Command, Data, Ehlo, MailFrom, RcptTo},
    extensions::{auth::Auth, EhloInfo},
    response::{ResponseError, ResponseParser},
};
use crate::{
    auth::Credential,
    io::{TcpStream, WithBuf},
    message::Mail,
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

        // server greeting
        ResponseParser::new(&mut stream).expect_code(b"220")?;

        let client_id = client_id.unwrap_or(ClientId::localhost());
        let ehlo_info = Ehlo(client_id).execute(&mut stream)?;

        if let Some(credential) = auth {
            let ehlo_info = &ehlo_info;

            Auth {
                credential,
                ehlo_info,
            }
            .execute(&mut stream)?;
        }

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
}

#[derive(Debug)]
pub enum ConnectError<E>
where
    E: TcpError,
{
    IoError(E),
    NoMem,
    FormatError,
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
    #[allow(unused)]
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

impl<T: TcpClientStack> SmtpClientSession<'_, T> {
    pub fn send(&mut self, mail: &Mail) -> Result<(), SendError<T::Error>> {
        let stream = &mut self.stream;

        let sender = mail.from.map(|m| m.address);
        MailFrom(sender).execute(&mut *stream)?;

        let receiver = mail
            .to
            .iter()
            .chain(mail.cc)
            .chain(mail.bcc)
            .map(|m| m.address);
        RcptTo(&receiver).execute(&mut *stream)?;

        Data(mail).execute(stream)
    }
}

#[derive(Debug)]
pub enum SendError<E: TcpError> {
    IoError(E),
    NoMem,
    SendFailed,
    UnexpectedResponse,
}

impl<E: TcpError> From<E> for SendError<E> {
    fn from(value: E) -> Self {
        Self::IoError(value)
    }
}

impl<E: TcpError> From<ResponseError<'_, E>> for SendError<E> {
    fn from(value: ResponseError<'_, E>) -> Self {
        match value {
            ResponseError::ReplyCodeError(_) => Self::SendFailed,
            ResponseError::ReadError(e) => Self::IoError(e),
            ResponseError::NoMem => Self::NoMem,
            ResponseError::FormatError => Self::UnexpectedResponse,
        }
    }
}
