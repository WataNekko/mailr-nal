pub struct Credential<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

impl<'a> Credential<'a> {
    pub fn new(username: &'a str, password: &'a str) -> Self {
        Self { username, password }
    }
}
