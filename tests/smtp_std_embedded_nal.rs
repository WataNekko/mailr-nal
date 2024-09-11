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

        let _client = SmtpClient::new(&mut stack, &mut buf[..])
            .connect(([127, 0, 0, 1], plain_port))
            .expect("connected without authentication");
    }

    #[test]
    fn with_client_id() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];
        const CLIENT_ID: &str = "example.com";

        let _ = SmtpClient::new(&mut stack, &mut buf[..])
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

        let result = SmtpClient::new(&mut stack, &mut buf[..])
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

        let _client = SmtpClient::new(&mut stack, &mut buf[..])
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
            .from(Mailbox::with_name("Smith", "Smith@bar.com"))
            .to(&["Jones@foo.com".into()])
            .cc(&["Green@foo.com".into()])
            .bcc(&["Brown@foo.com".into()])
            .bcc(&[Mailbox::new("Brown@foo.com")])
            .subject(None)
            .subject("Test mail")
            .body(None)
            .body("Blah blah blah...\r\n..etc. etc. etc.");
    }

    #[test]
    fn send_mail() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let mut client = SmtpClient::new(&mut stack, &mut buf[..])
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        let to = ["Jones@foo.com".into(), "John@foo.com".into()];
        let cc = ["Green@foo.com".into()];
        let bcc = ["Brown@foo.com".into()];
        let mail = Mail::new()
            .from("Smith@bar.com")
            .to(&to)
            .cc(&cc)
            .bcc(&bcc)
            .subject("Test mail")
            .body("Blah blah blah...\r\n..etc. etc. etc.");

        client.send(mail).expect("sent first message successfully");
    }

    #[test]
    fn send_multiple_mails() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let mut client = SmtpClient::new(&mut stack, &mut buf[..])
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        let to = ["Jones@foo.com".into(), "John@foo.com".into()];
        let cc = ["Green@foo.com".into()];
        let bcc = ["Brown@foo.com".into()];
        let mail = Mail::new()
            .from("Smith@bar.com")
            .to(&to)
            .cc(&cc)
            .bcc(&bcc)
            .subject("Test mail")
            .body("Blah blah blah...\r\n..etc. etc. etc.");

        client
            .send(mail.clone())
            .expect("sent first message successfully");

        let mail = mail.subject("Test mail 2");

        client.send(mail).expect("sent second message successfully");
    }

    #[test]
    fn send_raw() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let mut client = SmtpClient::new(&mut stack, &mut buf[..])
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        let envelope = Envelope::new(
            "Smith@bar.com",
            ["Jones@foo.com"]
                .iter()
                .chain(&["Jane@bar.com"])
                .chain(&Some("John@baz.org")),
        );
        let raw_message = concat!(
            "From:<Smith@bar.com>\r\n",
            "To:<Jones@foo.com>\r\n",
            "Subject: Raw mail sending\r\n",
            "\r\n",
            "Blah blah blah...\r\n",
            "..etc. etc. etc."
        );

        client
            .send_raw(envelope, raw_message)
            .expect("sent successfully");
    }

    #[test]
    fn close() {
        let TestContext { plain_port, .. } = TestContext::setup();

        let mut stack = std_embedded_nal::Stack;
        let mut buf = [0; 1024];

        let client = SmtpClient::new(&mut stack, &mut buf[..])
            .connect(([127, 0, 0, 1], plain_port))
            .unwrap();

        client.close().expect("close successfully");
    }
}
