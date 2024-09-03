#[cfg(test)]
mod connect {
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

#[cfg(test)]
mod send {
    use mailr_nal::{
        message::{Envelope, Mail, Mailbox},
        smtp::SmtpClient,
    };
    use test_common::TestContext;

    #[test]
    fn mail_message() {
        let _ = Mail::new()
            .from("Smith@bar.com")
            .to(&["Jones@foo.com"])
            .cc(&["Green@foo.com"])
            .bcc(&["Brown@foo.com"])
            .subject("Test mail")
            .body("Blah blah blah...\r\n..etc. etc. etc.");
    }

    #[cfg(todo)]
    #[test]
    fn send_mail() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let mut client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        let mail = Mail::new()
            .from("Smith@bar.com")
            .to(&["Jones@foo.com"])
            .cc(&["Green@foo.com"])
            .bcc(&["Brown@foo.com"])
            .subject("Test mail")
            .body("Blah blah blah...\r\n..etc. etc. etc.");

        client.send(&mail).expect("sent first message successfully");
    }

    #[cfg(todo)]
    #[test]
    fn send_multiple_mails() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let mut client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        let mail = Mail::new()
            .from("Smith@bar.com")
            .to(&["Jones@foo.com"])
            .cc(&["Green@foo.com"])
            .bcc(&["Brown@foo.com"])
            .subject("Test mail")
            .body("Blah blah blah...\r\n..etc. etc. etc.");

        client.send(&mail).expect("sent first message successfully");

        let mail = mail.subject("Test mail 2");

        client
            .send(&mail)
            .expect("sent second message successfully");
    }

    #[cfg(todo)]
    #[test]
    #[ignore]
    fn send_raw() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let mut client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        let envelope = Envelope::new("Smith@bar.com", "Jones@foo.com");
        let raw_message = concat!(
            "From:<Smith@bar.com>\r\n",
            "To:<Jones@foo.com>\r\n",
            "Subject: Test mail\r\n",
            "\r\n",
            "Blah blah blah...\r\n",
            "..etc. etc. etc."
        );

        client
            .send_raw(envelope, raw_message)
            .expect("sent successfully");
    }

    #[cfg(todo)]
    #[test]
    #[ignore]
    fn close() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let client = SmtpClient::new(&mut stack, &mut buf)
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        client.close().expect("close successfully");
    }
}
