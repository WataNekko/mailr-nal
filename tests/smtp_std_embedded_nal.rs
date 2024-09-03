#[cfg(test)]
mod smtp {
    use std::env::{self, VarError};

    use mailr_nal::{
        auth::Credential,
        smtp::{ClientId, ConnectError, SmtpClient},
    };

    struct TestContext {
        plain_port: u16,
        tls_port: u16,
        username: String,
        password: String,
    }

    impl TestContext {
        fn setup() -> Self {
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

    #[test]
    fn connect_no_auth() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], plain_port))
            .expect("connected without authentication");
    }

    #[test]
    fn connect_with_client_id() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];
        const CLIENT_ID: &str = "example.com";

        let _ = SmtpClient::new(&mut stack, &mut buf)
            .with_client_id(Some(ClientId::new(CLIENT_ID)))
            .with_client_id(ClientId::new(CLIENT_ID))
            .with_client_id(Some(CLIENT_ID.into()))
            .connect(([127, 0, 0, 1], plain_port))
            .expect("connected with client id");
    }

    #[test]
    fn connect_unexpected_auth_fail() {
        let TestContext {
            plain_port,
            username,
            password,
            ..
        } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let result = SmtpClient::new(&mut stack, &mut buf)
            .with_auth(Some(Credential::new(&username, &password)))
            .connect(([127, 0, 0, 1], plain_port));

        assert!(
            matches!(result, Err(ConnectError::AuthUnsupported)),
            "Connect should fail if user expects auth but server cannot provide. Got: {:?}",
            result,
        );
    }

    #[test]
    fn connect_hostname_dns() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut dns = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .connect_with_hostname(&mut dns, "localhost", plain_port)
            .expect("connected to hostname");
    }
}
