/// Domain or address literal that identifies the client (https://www.rfc-editor.org/rfc/rfc5321#section-4.1.1.1)
#[repr(transparent)]
pub struct ClientId<'a>(&'a str);

impl<'a> ClientId<'a> {
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

pub struct Ehlo<'a>(pub(crate) ClientId<'a>);

impl core::fmt::Display for Ehlo<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EHLO {}\r\n", self.0)
    }
}
