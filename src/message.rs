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
pub struct Envelope<'a, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    pub sender_addr: Option<&'a str>,
    pub receiver_addrs: I,
}

impl<'a, S, I> Envelope<'a, S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    // FIXME: validate input
    pub fn new(from: impl Into<Option<&'a str>>, to: impl IntoIterator<IntoIter = I>) -> Self {
        Self {
            sender_addr: from.into(),
            receiver_addrs: to.into_iter(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mail<'a, To, Cc, Bcc>
where
    To: Iterator<Item = &'a Mailbox<'a>>,
    Cc: Iterator<Item = &'a Mailbox<'a>>,
    Bcc: Iterator<Item = &'a Mailbox<'a>>,
{
    pub from: Option<Mailbox<'a>>,
    pub to: To,
    pub cc: Cc,
    pub bcc: Bcc,
    pub subject: Option<&'a str>,
    pub body: Option<&'a str>,
}

type NoMailboxIter<'a> = core::option::Iter<'a, Mailbox<'a>>;

impl<'a> Mail<'a, NoMailboxIter<'a>, NoMailboxIter<'a>, NoMailboxIter<'a>> {
    pub fn new() -> Self {
        Self {
            from: None,
            to: None.iter(),
            cc: None.iter(),
            bcc: None.iter(),
            subject: None,
            body: None,
        }
    }
}

impl<'a, To, Cc, Bcc> Mail<'a, To, Cc, Bcc>
where
    To: Iterator<Item = &'a Mailbox<'a>>,
    Cc: Iterator<Item = &'a Mailbox<'a>>,
    Bcc: Iterator<Item = &'a Mailbox<'a>>,
{
    pub fn from(mut self, value: impl Into<Mailbox<'a>>) -> Self {
        self.from = Some(value.into());
        self
    }

    pub fn replace_to<I>(self, value: impl IntoIterator<IntoIter = I>) -> (Mail<'a, I, Cc, Bcc>, To)
    where
        I: Iterator<Item = &'a Mailbox<'a>>,
    {
        let mail = Mail {
            from: self.from,
            to: value.into_iter(),
            cc: self.cc,
            bcc: self.bcc,
            subject: self.subject,
            body: self.body,
        };

        (mail, self.to)
    }

    pub fn to<I>(self, value: impl IntoIterator<IntoIter = I>) -> Mail<'a, I, Cc, Bcc>
    where
        I: Iterator<Item = &'a Mailbox<'a>>,
    {
        self.replace_to(value).0
    }

    pub fn replace_cc<I>(self, value: impl IntoIterator<IntoIter = I>) -> (Mail<'a, To, I, Bcc>, Cc)
    where
        I: Iterator<Item = &'a Mailbox<'a>>,
    {
        let mail = Mail {
            from: self.from,
            to: self.to,
            cc: value.into_iter(),
            bcc: self.bcc,
            subject: self.subject,
            body: self.body,
        };

        (mail, self.cc)
    }

    pub fn cc<I>(self, value: impl IntoIterator<IntoIter = I>) -> Mail<'a, To, I, Bcc>
    where
        I: Iterator<Item = &'a Mailbox<'a>>,
    {
        self.replace_cc(value).0
    }

    pub fn replace_bcc<I>(
        self,
        value: impl IntoIterator<IntoIter = I>,
    ) -> (Mail<'a, To, Cc, I>, Bcc)
    where
        I: Iterator<Item = &'a Mailbox<'a>>,
    {
        let mail = Mail {
            from: self.from,
            to: self.to,
            cc: self.cc,
            bcc: value.into_iter(),
            subject: self.subject,
            body: self.body,
        };

        (mail, self.bcc)
    }

    pub fn bcc<I>(self, value: impl IntoIterator<IntoIter = I>) -> Mail<'a, To, Cc, I>
    where
        I: Iterator<Item = &'a Mailbox<'a>>,
    {
        self.replace_bcc(value).0
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
