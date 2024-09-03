use std::env::{self, VarError};

pub struct TestContext {
    pub plain_port: u16,
    pub tls_port: u16,
    pub username: String,
    pub password: String,
}

impl TestContext {
    pub fn setup() -> Self {
        const AUTH_USER_ENV: &str = "AUTH_USER";
        const AUTH_PASS_ENV: &str = "AUTH_PASS";
        const PLAIN_PORT_ENV: &str = "PLAIN_PORT";
        const TLS_PORT_ENV: &str = "TLS_PORT";

        let user = env::var(AUTH_USER_ENV);
        let pass = env::var(AUTH_PASS_ENV);

        let (username, password) = match (user, pass) {
            (Err(VarError::NotPresent), Err(VarError::NotPresent)) => {
                ("mock".into(), "123456".into())
            }
            (Ok(user), Ok(pass)) => (user, pass),
            invalid => panic!(
                "Provide both AUTH_USER and AUTH_PASS env, or none. Got: {:?}",
                invalid
            ),
        };

        let plain_port = match env::var(PLAIN_PORT_ENV) {
            Err(VarError::NotPresent) => 2525,
            Ok(port) => port
                .parse()
                .unwrap_or_else(|_| panic!("{} must be a u16. Got: {}", PLAIN_PORT_ENV, port)),
            invalid => panic!("{:?}", invalid),
        };

        let tls_port = match env::var(TLS_PORT_ENV) {
            Err(VarError::NotPresent) => 5870,
            Ok(port) => port
                .parse()
                .unwrap_or_else(|_| panic!("{} must be a u16. Got: {}", TLS_PORT_ENV, port)),
            invalid => panic!("{:?}", invalid),
        };

        Self {
            plain_port,
            tls_port,
            username,
            password,
        }
    }
}
