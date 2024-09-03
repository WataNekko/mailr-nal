#[cfg(test)]
mod connect {
    use std::env::{self, VarError};

    use mailr_nal::{
        auth::Credential,
        smtp::{ClientId, ConnectError, SmtpClient},
    };
    use test_common::TestContext;

    #[test]
    fn no_auth() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], plain_port))
            .expect("connected without authentication");
    }

    #[test]
    fn with_client_id() {
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
    fn unexpected_auth_fail() {
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
    fn hostname_dns() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut dns = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .connect_with_hostname(&mut dns, "localhost", plain_port)
            .expect("connected to hostname");
    }
}
