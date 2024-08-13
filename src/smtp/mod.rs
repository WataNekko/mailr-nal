use embedded_nal::{nb::block, Dns, SocketAddr, TcpClientStack};

use crate::auth::Credential;

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

    pub fn connect(
        &mut self,
        remote: impl Into<SocketAddr>,
    ) -> Result<SmtpClientSession, T::Error> {
        let mut sock = self.stack.socket()?;
        let remote = remote.into();
        block!(self.stack.connect(&mut sock, remote))?;

        Ok(SmtpClientSession)
    }

    pub fn connect_with_hostname<D>(
        &mut self,
        dns: &mut D,
        hostname: &str,
        port: u16,
    ) -> Result<SmtpClientSession, ConnectHostnameError<D::Error, T::Error>>
    where
        D: Dns,
    {
        let addr = block!(dns.get_host_by_name(hostname, embedded_nal::AddrType::Either))
            .map_err(|e| ConnectHostnameError::DnsError(e))?;
        self.connect((addr, port))
            .map_err(|e| ConnectHostnameError::ConnectError(e))
    }
}

#[derive(Debug)]
pub enum ConnectHostnameError<DErr, TErr> {
    DnsError(DErr),
    ConnectError(TErr),
}

pub struct SmtpClientSession;
