use embedded_nal::TcpClientStack;
use enumset::{enum_set, EnumSet};

use super::{EhloInfo, SmtpExtension};
use crate::{
    auth::Credential,
    io::{TcpStream, WithBuf},
    smtp::{commands::Command, ConnectError},
};

/// Mask of all AUTH extension flags. `&` with this to check if AUTH is supported.
pub const AUTH_EXTENSION_MASK: EnumSet<SmtpExtension> =
    enum_set!(SmtpExtension::AuthLogin | SmtpExtension::AuthPlain);

/// AUTH command for SMTP authentication extension (https://www.rfc-editor.org/rfc/rfc4954).
pub struct Auth<'cred, 'ehlo> {
    pub credential: Credential<'cred>,
    pub ehlo_info: &'ehlo EhloInfo,
}

impl<T: TcpClientStack> Command<T> for Auth<'_, '_> {
    type Output = ();
    type Error = ConnectError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        let Self {
            credential,
            ehlo_info,
        } = self;

        let supported = ehlo_info.extensions & AUTH_EXTENSION_MASK;
        if supported.is_empty() {
            return Err(ConnectError::AuthUnsupported);
        }

        for mechanism in supported.iter() {
            match mechanism {
                SmtpExtension::AuthPlain => match AuthPlain(credential).execute(&mut *stream) {
                    Ok(()) => return Ok(()),
                    Err(ConnectError::AuthFailed) => continue,
                    Err(e) => return Err(e),
                },
                SmtpExtension::AuthLogin => match AuthLogin(credential).execute(&mut *stream) {
                    Ok(()) => return Ok(()),
                    Err(ConnectError::AuthFailed) => continue,
                    Err(e) => return Err(e),
                },
                _ => unreachable!(),
            }
        }

        Err(ConnectError::AuthFailed)
    }
}

struct AuthPlain<'cred>(Credential<'cred>);

impl<T: TcpClientStack> Command<T> for AuthPlain<'_> {
    type Output = ();
    type Error = ConnectError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        let Self(credential) = self;

        Err(ConnectError::AuthFailed)
    }
}

struct AuthLogin<'cred>(Credential<'cred>);

impl<T: TcpClientStack> Command<T> for AuthLogin<'_> {
    type Output = ();
    type Error = ConnectError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        let Self(credential) = self;

        Err(ConnectError::AuthFailed)
    }
}
