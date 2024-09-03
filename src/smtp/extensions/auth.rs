use base64::engine::{general_purpose::STANDARD as BASE64, Engine};
use core::fmt::Write;
use embedded_nal::TcpClientStack;
use enumset::{enum_set, EnumSet};

use super::{EhloInfo, SmtpExtension};
use crate::{
    auth::Credential,
    io::{BufWriter, TcpStream, WithBuf},
    smtp::{
        commands::Command,
        response::{ResponseError, ResponseParser},
        ConnectError,
    },
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
                #[allow(unreachable_patterns)]
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
        let Self(Credential { username, password }) = self;

        // FIXME: The max credential length of 512 octets should be RFC compliant
        // (https://www.rfc-editor.org/rfc/rfc4616#section-2). But is there another way to
        // work around using a fixed buffer? E.g., utilize `stream`'s buffer somehow, or
        // use a base64 in-place encoding implementation to avoid using an intermediate buffer
        // like this.
        let mut auth_buffer = heapless::String::<512>::new();
        write!(auth_buffer, "\0{}\0{}", username, password).map_err(|_| ConnectError::NoMem)?;

        // FIXME: reuse stream buffer somehow.
        let mut encoded_auth = [0; 1024];
        let n = BASE64
            .encode_slice(auth_buffer, &mut encoded_auth)
            .map_err(|_| ConnectError::NoMem)?;

        let encoded_auth =
            core::str::from_utf8(&encoded_auth[..n]).map_err(|_| ConnectError::FormatError)?;

        {
            let mut stream = BufWriter::from(&mut *stream);
            write!(stream, "AUTH PLAIN {}\r\n", encoded_auth)?;
        }

        ResponseParser::new(stream)
            .expect_code(b"235")
            .map_err(|e| match e {
                ResponseError::ReplyCodeError(_) => ConnectError::AuthFailed,
                e => e.into(),
            })
    }
}

struct AuthLogin<'cred>(Credential<'cred>);

impl<T: TcpClientStack> Command<T> for AuthLogin<'_> {
    type Output = ();
    type Error = ConnectError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        let Self(Credential { username, password }) = self;

        BufWriter::from(&mut *stream).write(b"AUTH LOGIN\r\n")?;

        ResponseParser::new(&mut *stream)
            .expect_code(b"334")
            .map_err(|e| match e {
                ResponseError::ReplyCodeError(_) => ConnectError::AuthFailed,
                e => e.into(),
            })?;

        // username
        {
            // FIXME: reuse stream buffer somehow.
            let mut encoded = [0; 512];
            let n = BASE64
                .encode_slice(username, &mut encoded)
                .map_err(|_| ConnectError::NoMem)?;

            let mut stream = BufWriter::from(&mut *stream);
            stream.write(&encoded[..n])?;
            stream.write(b"\r\n")?;
        }

        ResponseParser::new(&mut *stream)
            .expect_code(b"334")
            .map_err(|e| match e {
                ResponseError::ReplyCodeError(_) => ConnectError::AuthFailed,
                e => e.into(),
            })?;

        // password
        {
            // FIXME: reuse stream buffer somehow.
            let mut encoded = [0; 512];
            let n = BASE64
                .encode_slice(password, &mut encoded)
                .map_err(|_| ConnectError::NoMem)?;

            let mut stream = BufWriter::from(&mut *stream);
            stream.write(&encoded[..n])?;
            stream.write(b"\r\n")?;
        }

        ResponseParser::new(&mut *stream)
            .expect_code(b"235")
            .map_err(|e| match e {
                ResponseError::ReplyCodeError(_) => ConnectError::AuthFailed,
                e => e.into(),
            })
    }
}
