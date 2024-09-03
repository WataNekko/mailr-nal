#[derive(Clone, Copy, Debug)]
pub struct Mailbox<'a> {
    pub name: Option<&'a str>,
    pub address: &'a str,
}

impl<'a> Mailbox<'a> {
    // FIXME: validate input
    pub fn new(address: &'a str) -> Self {
        Self {
            address,
            name: None,
        }
    }

    // FIXME: validate input
    pub fn with_name(name: &'a str, address: &'a str) -> Self {
        Self {
            name: Some(name),
            address,
        }
    }
}

impl<'a> From<&'a str> for Mailbox<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl core::fmt::Display for Mailbox<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}<{}>", self.name.unwrap_or(""), self.address)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Envelope<'a> {
    pub from_addr: &'a str,
    pub to_addr: &'a str,
}

impl<'a> Envelope<'a> {
    // FIXME: validate input
    pub fn new(from: &'a str, to: &'a str) -> Self {
        Self {
            from_addr: from,
            to_addr: to,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mail<'a> {
    pub from: Option<Mailbox<'a>>,
    pub to: &'a [Mailbox<'a>],
    pub cc: &'a [Mailbox<'a>],
    pub bcc: &'a [Mailbox<'a>],
    pub subject: Option<&'a str>,
    pub body: Option<&'a str>,
}

impl<'a> Mail<'a> {
    pub fn new() -> Self {
        Self {
            from: None,
            to: &[],
            cc: &[],
            bcc: &[],
            subject: None,
            body: None,
        }
    }

    pub fn from(mut self, value: impl Into<Mailbox<'a>>) -> Self {
        self.from = Some(value.into());
        self
    }

    pub fn to(mut self, value: &'a [Mailbox<'a>]) -> Self {
        self.to = value;
        self
    }

    pub fn cc(mut self, value: &'a [Mailbox<'a>]) -> Self {
        self.cc = value;
        self
    }

    pub fn bcc(mut self, value: &'a [Mailbox<'a>]) -> Self {
        self.bcc = value;
        self
    }

    pub fn subject(mut self, value: impl Into<Option<&'a str>>) -> Self {
        self.subject = value.into();
        self
    }

    pub fn body(mut self, value: impl Into<Option<&'a str>>) -> Self {
        self.body = value.into();
        self
    }
}
