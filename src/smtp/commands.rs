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

pub struct Ehlo<'a>(pub(crate) ClientId<'a>);
