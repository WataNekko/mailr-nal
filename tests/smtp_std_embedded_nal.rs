#[cfg(test)]
mod smtp {
    use mailr_nal::{auth::Credential, SmtpClient};

    #[test]
    fn connect_no_auth() {
        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], 2525))
            .expect("connected without authentication");
    }

    #[test]
    fn connect_unexpected_auth_fail() {
        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let result = SmtpClient::new(&mut stack, &mut buf)
            .with_auth(Some(Credential::new("mock", "123456")))
            .connect(([127, 0, 0, 1], 2525));

        assert!(
            !result.is_ok(),
            "Connect should fail if user expects auth but server cannot provide",
        );
    }

    #[test]
    fn connect_hostname_dns() {
        let mut stack = std_embedded_nal::Stack;
        let mut dns = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let _client = SmtpClient::new(&mut stack, &mut buf)
            .connect_with_hostname(&mut dns, "localhost", 2525)
            .expect("connected to hostname");
    }
}
