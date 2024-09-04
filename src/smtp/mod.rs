mod commands;
mod extensions;
mod response;

use core::{fmt::Debug, mem::ManuallyDrop};
use embedded_nal::{nb::block, AddrType, Dns, SocketAddr, TcpClientStack, TcpError};

pub use self::commands::ClientId;
use self::{
    commands::{Command, Data, DataMessage, Ehlo, MailFrom, Quit, RcptTo},
    extensions::{auth::Auth, EhloInfo},
    response::{ResponseError, ResponseParser},
};
use crate::{
    auth::Credential,
    io::{TcpStream, WithBuf},
    message::{Envelope, Mail},
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
        let stream = WithBuf(stream, buffer);
        let mut stream = QuitOnDrop(stream);

        // server greeting
        ResponseParser::new(&mut stream.0).expect_code(b"220")?;

        let client_id = client_id.unwrap_or(ClientId::localhost());
        let ehlo_info = Ehlo(client_id).execute(&mut stream.0)?;

        if let Some(credential) = auth {
            let ehlo_info = &ehlo_info;

            Auth {
                credential,
                ehlo_info,
            }
            .execute(&mut stream.0)?;
        }

        Ok(SmtpClientSession {
            stream: stream.into_inner(),
            ehlo_info,
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

/// For clean up on `connect` fails.
/// FIXME: integrate this into `SmtpClientSession` struct would be nice.
struct QuitOnDrop<'a, 'b, T: TcpClientStack>(WithBuf<'a, TcpStream<'b, T>>);

impl<T: TcpClientStack> Drop for QuitOnDrop<'_, '_, T> {
    fn drop(&mut self) {
        let _ = Quit.execute(&mut self.0);
    }
}

impl<'a, 'b, T: TcpClientStack> QuitOnDrop<'a, 'b, T> {
    pub fn into_inner(self) -> WithBuf<'a, TcpStream<'b, T>> {
        let me = ManuallyDrop::new(self);

        // SAFETY: safe to extract inner as it's never touched again otherwise.
        unsafe { core::ptr::read(&me.0) }
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
    fn send_internal<S, I>(
        &mut self,
        envelope: Envelope<S, I>,
        message: impl DataMessage,
    ) -> Result<(), SendError<T::Error>>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let stream = &mut self.stream;

        let Envelope {
            sender_addr,
            receiver_addrs,
        } = envelope;

        MailFrom(sender_addr).execute(&mut *stream)?;
        RcptTo(receiver_addrs).execute(&mut *stream)?;
        Data(message).execute(stream)
    }

    #[inline]
    pub fn send(&mut self, mail: &Mail) -> Result<(), SendError<T::Error>> {
        let sender = mail.from.map(|m| m.address);
        let receivers = mail
            .to
            .iter()
            .chain(mail.cc)
            .chain(mail.bcc)
            .map(|m| m.address);

        let envelope = Envelope::new(sender, receivers);

        self.send_internal(envelope, mail)
    }

    #[inline]
    pub fn send_raw<S, I>(
        &mut self,
        envelope: Envelope<S, I>,
        message: &str,
    ) -> Result<(), SendError<T::Error>>
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        self.send_internal(envelope, message)
    }

    pub fn close(self) -> Result<(), T::Error> {
        let mut me = ManuallyDrop::new(self);
        Quit.execute(&mut me.stream)?;

        // SAFETY: `stream` is behind `ManuallyDrop` and is never touched again
        // so it's safe to convert here.
        unsafe { core::ptr::read(&me.stream).0.close() }
    }
}

impl<T: TcpClientStack> Drop for SmtpClientSession<'_, T> {
    fn drop(&mut self) {
        let _ = Quit.execute(&mut self.stream);
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
