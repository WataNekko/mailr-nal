use embedded_nal::TcpClientStack;
use enumset::EnumSet;

use super::{
    extensions::{EhloInfo, SmtpExtension},
    response::{ReplyLine, ResponseParser},
    ConnectError, SendError,
};
use crate::{
    io::{BufWriter, TcpStream, WithBuf, Write},
    message::Mail,
};

/// An SMTP command that can be executed (e.g., EHLO, MAIL, RCPT, etc.).
pub trait Command<T>
where
    T: TcpClientStack,
{
    type Output;
    type Error;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error>;
}

/// Domain or address literal that identifies the client (https://www.rfc-editor.org/rfc/rfc5321#section-4.1.1.1)
#[repr(transparent)]
pub struct ClientId<'a>(&'a str);

impl<'a> ClientId<'a> {
    // FIXME: validate input
    pub fn new(id: &'a str) -> Self {
        Self(id)
    }

    pub const fn localhost() -> Self {
        Self("localhost")
    }
}

impl<'a> From<&'a str> for ClientId<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl core::fmt::Display for ClientId<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// EHLO command for greeting and register supported SMTP extensions
/// (https://www.rfc-editor.org/rfc/rfc5321#section-4.1.1.1).
pub struct Ehlo<'a>(pub(crate) ClientId<'a>);

impl<T: TcpClientStack> Command<T> for Ehlo<'_> {
    type Output = EhloInfo;
    type Error = ConnectError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        let Self(client_id) = self;

        {
            let mut stream = BufWriter::from(&mut *stream);
            write!(stream, "EHLO {}\r\n", client_id)?;
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

/// MAIL FROM command.
pub struct MailFrom<'a>(pub Option<&'a str>);

impl<T: TcpClientStack> Command<T> for MailFrom<'_> {
    type Output = ();
    type Error = SendError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        let sender = self.0.unwrap_or("");

        {
            let mut stream = BufWriter::from(&mut *stream);
            write!(stream, "MAIL FROM:<{}>\r\n", sender)?;
        }

        ResponseParser::new(stream).expect_code(b"250")?;
        Ok(())
    }
}

/// RCPT TO command.
pub struct RcptTo<S, I>(pub I)
where
    S: AsRef<str>,
    I: Iterator<Item = S>;

impl<T, S, I> Command<T> for RcptTo<S, I>
where
    T: TcpClientStack,
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    type Output = ();
    type Error = SendError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        for receiver in self.0 {
            {
                let mut stream = BufWriter::from(&mut *stream);
                write!(stream, "RCPT TO:<{}>\r\n", receiver.as_ref())?;
            }
            ResponseParser::new(&mut *stream).expect_code(b"250")?;
        }

        Ok(())
    }
}

/// DATA command.
pub struct Data<'a, 'mail>(pub &'a Mail<'mail>);

impl Data<'_, '_> {
    fn write_sanitized<W: Write>(
        writer: &mut BufWriter<W>,
        mut data: &str,
    ) -> Result<(), W::Error> {
        const DELIM: &str = "\r\n.";

        loop {
            let pos = data.find(DELIM).map(|p| p + DELIM.len() - 1);

            match pos {
                Some(pos) => {
                    write!(writer, "{}", &data[..=pos])?;
                    data = &data[pos..];
                }
                None => {
                    write!(writer, "{}", data)?;
                    break;
                }
            };
        }

        Ok(())
    }
}

impl<T: TcpClientStack> Command<T> for Data<'_, '_> {
    type Output = ();
    type Error = SendError<T::Error>;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        let mail = self.0;

        BufWriter::from(&mut *stream).write(b"DATA\r\n")?;
        ResponseParser::new(&mut *stream).expect_code(b"354")?;

        {
            let mut stream = BufWriter::from(&mut *stream);

            if let Some(from) = mail.from {
                write!(stream, "From:{}\r\n", from)?;
            }

            if let Some((first, rest)) = mail.to.split_first() {
                write!(stream, "To:{}", first)?;
                for rcv in rest {
                    write!(stream, ",{}", rcv)?;
                }
                write!(stream, "\r\n")?;
            }

            if let Some((first, rest)) = mail.cc.split_first() {
                write!(stream, "Cc:{}", first)?;
                for rcv in rest {
                    write!(stream, ",{}", rcv)?;
                }
                write!(stream, "\r\n")?;
            }

            if let Some(subject) = mail.subject {
                write!(stream, "Subject:{}\r\n", subject)?;
            }

            if let Some(body) = mail.body {
                write!(stream, "\r\n")?;
                Self::write_sanitized(&mut stream, body)?;

                if !body.ends_with("\r\n") {
                    write!(stream, "\r\n")?;
                }
            }

            write!(stream, ".\r\n")?;
        }

        ResponseParser::new(stream).expect_code(b"250")?;
        Ok(())
    }
}

/// QUIT command
pub struct Quit;

impl<T: TcpClientStack> Command<T> for Quit {
    type Output = ();
    type Error = T::Error;

    fn execute(self, stream: &mut WithBuf<TcpStream<T>>) -> Result<Self::Output, Self::Error> {
        BufWriter::from(stream).write(b"QUIT\r\n")
    }
}
