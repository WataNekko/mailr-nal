use core::str;
use std::{env, fs, io::Write};

use base64::engine::{general_purpose::STANDARD, Engine};
use embedded_nal::{nb::block, AddrType, Dns, IpAddr, SocketAddr, TcpClientStack};

mod tls {
    use std::{
        io::{Read, Write},
        net::TcpStream,
    };

    use convert::SocketAddr;
    use embedded_nal::{TcpClientStack, TcpError};
    use native_tls::{HandshakeError, TlsConnector, TlsStream};

    mod convert {
        use std::{
            io,
            net::{self, ToSocketAddrs},
            option::IntoIter,
        };

        /// Wrapper around the `std` IP address type that converts to `non_std`
        /// counterpart and vice versa.
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct IpAddr(net::IpAddr);

        impl From<embedded_nal::IpAddr> for IpAddr {
            fn from(input: embedded_nal::IpAddr) -> Self {
                match input {
                    embedded_nal::IpAddr::V4(i) => Self(i.octets().into()),
                    embedded_nal::IpAddr::V6(i) => Self(i.octets().into()),
                }
            }
        }

        impl From<IpAddr> for embedded_nal::IpAddr {
            fn from(s: IpAddr) -> embedded_nal::IpAddr {
                match s.0 {
                    net::IpAddr::V4(i) => i.octets().into(),
                    net::IpAddr::V6(i) => i.octets().into(),
                }
            }
        }

        impl From<IpAddr> for net::IpAddr {
            fn from(s: IpAddr) -> net::IpAddr {
                s.0
            }
        }

        /// Wrapper around the `std` socket address type that converts to `non_std`
        /// counterpart and vice versa.
        #[derive(Debug, Clone, Copy)]
        pub(crate) struct SocketAddr(net::SocketAddr);

        impl ToSocketAddrs for SocketAddr {
            type Iter = IntoIter<net::SocketAddr>;
            fn to_socket_addrs(&self) -> io::Result<IntoIter<net::SocketAddr>> {
                self.0.to_socket_addrs()
            }
        }

        impl From<net::SocketAddr> for SocketAddr {
            fn from(input: net::SocketAddr) -> Self {
                Self(input)
            }
        }

        impl From<SocketAddr> for net::SocketAddr {
            fn from(s: SocketAddr) -> net::SocketAddr {
                s.0
            }
        }

        impl From<embedded_nal::SocketAddr> for SocketAddr {
            fn from(input: embedded_nal::SocketAddr) -> Self {
                Self((IpAddr::from(input.ip()).0, input.port()).into())
            }
        }

        impl From<SocketAddr> for embedded_nal::SocketAddr {
            fn from(s: SocketAddr) -> embedded_nal::SocketAddr {
                (IpAddr(s.0.ip()), s.0.port()).into()
            }
        }
    }

    enum TlsSocketState {
        Unconnected,
        Connected(TlsStream<TcpStream>),
    }
    pub struct TlsSocket(TlsSocketState);

    #[derive(Debug)]
    pub struct TlsError;

    impl TcpError for TlsError {
        fn kind(&self) -> embedded_nal::TcpErrorKind {
            embedded_nal::TcpErrorKind::Other
        }
    }

    impl From<std::io::Error> for TlsError {
        fn from(_value: std::io::Error) -> Self {
            Self
        }
    }

    impl<T> From<HandshakeError<T>> for TlsError {
        fn from(_value: HandshakeError<T>) -> Self {
            Self
        }
    }

    pub struct TlsStack(TlsConnector);

    impl TlsStack {
        pub fn new(tls: TlsConnector) -> Self {
            Self(tls)
        }
    }

    impl TcpClientStack for TlsStack {
        type TcpSocket = TlsSocket;

        type Error = TlsError;

        fn socket(&mut self) -> Result<Self::TcpSocket, Self::Error> {
            Ok(TlsSocket(TlsSocketState::Unconnected))
        }

        fn connect(
            &mut self,
            socket: &mut Self::TcpSocket,
            remote: embedded_nal::SocketAddr,
        ) -> embedded_nal::nb::Result<(), Self::Error> {
            let stream = TcpStream::connect(SocketAddr::from(remote)).map_err(TlsError::from)?;
            let stream = self
                .0
                .connect("localhost", stream)
                .map_err(TlsError::from)?;
            socket.0 = TlsSocketState::Connected(stream);
            Ok(())
        }

        fn send(
            &mut self,
            socket: &mut Self::TcpSocket,
            buffer: &[u8],
        ) -> embedded_nal::nb::Result<usize, Self::Error> {
            let TlsSocketState::Connected(ref mut conn) = socket.0 else {
                return Err(TlsError)?;
            };
            let n = conn.write(buffer).map_err(TlsError::from)?;
            conn.flush().map_err(TlsError::from)?;
            Ok(n)
        }

        fn receive(
            &mut self,
            socket: &mut Self::TcpSocket,
            buffer: &mut [u8],
        ) -> embedded_nal::nb::Result<usize, Self::Error> {
            let TlsSocketState::Connected(ref mut conn) = socket.0 else {
                return Err(TlsError)?;
            };
            Ok(conn.read(buffer).map_err(TlsError::from)?)
        }

        fn close(&mut self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
            match socket.0 {
                TlsSocketState::Connected(mut conn) => conn.shutdown().map_err(Into::into),
                TlsSocketState::Unconnected => Ok(()),
            }
        }
    }
}

#[cfg(doc)]
mod io {
    use core::str;
    use embedded_nal::{nb::block, TcpClientStack};
    use std::ops::Range;

    pub struct LineReader<'a, T: TcpClientStack> {
        stack: &'a mut T,
        sock: &'a mut T::TcpSocket,
        buf: &'a mut [u8],
        start: usize,
        end: usize,
    }

    impl<'a, T: TcpClientStack> LineReader<'a, T> {
        pub fn new(stack: &'a mut T, sock: &'a mut T::TcpSocket, buf: &'a mut [u8]) -> Self {
            LineReader {
                stack,
                sock,
                buf,
                start: 0,
                end: 0,
            }
        }

        fn pos(byte: u8, buf: &[u8]) -> Option<usize> {
            buf.iter().position(|&b| b == byte)
        }

        fn move_range_to_front(&mut self, range: Range<usize>) {
            self.start = 0;
            self.end = range.len();
            self.buf.copy_within(range, 0);
        }

        fn no_crlf(mut i: &[u8]) -> &[u8] {
            str::from_utf8(i)
                .unwrap()
                .trim_end_matches("\r\n")
                .as_bytes()
        }

        fn read_new_bytes(&mut self) -> usize {
            let read = block!(self.stack.receive(self.sock, &mut self.buf[self.end..])).unwrap();
            println!(
                ">> READ ({}): {}\nNow ({}): {}",
                read,
                str::from_utf8(Self::no_crlf(&self.buf[self.end..self.end + read])).unwrap(),
                read + (self.end - self.start),
                str::from_utf8(Self::no_crlf(&self.buf[self.start..self.end + read])).unwrap()
            );
            self.end += read;
            read
        }

        fn read_until_byte_starting_from_pos(
            &mut self,
            byte: u8,
            unchecked_start: usize,
        ) -> Option<Range<usize>> {
            let mut pos = Self::pos(byte, &self.buf[unchecked_start..self.end])?;
            pos += unchecked_start;

            let start = self.start;
            self.start = pos + 1;

            Some(start..pos)
        }

        fn read_until(&mut self, byte: u8) -> Option<&[u8]> {
            if let Some(line_bounds) = self.read_until_byte_starting_from_pos(byte, self.start) {
                return Some(&self.buf[line_bounds]);
            }

            // move remaining bytes to front
            self.move_range_to_front(self.start..self.end);

            loop {
                // read new bytes since we're missing EOL
                let prev_end = self.end;
                if self.read_new_bytes() == 0 {
                    return None; //EOF
                }

                if let Some(line_bounds) = self.read_until_byte_starting_from_pos(byte, prev_end) {
                    return Some(&self.buf[line_bounds]);
                }
            }
        }
        pub fn read_line(&mut self) -> Option<&[u8]> {
            let line = self.read_until(b'\n')?;
            Some(
                line.split_last()
                    .filter(|(&last, _)| last == b'\r')
                    .map_or(line, |(_, rest)| rest),
            )
        }
        pub fn read_word(&mut self) -> Option<&[u8]> {
            self.read_until(b' ')
        }
    }

    pub fn send_all<T: TcpClientStack>(stack: &mut T, sock: &mut T::TcpSocket, mut buf: &[u8]) {
        while buf.len() > 0 {
            let n = block!(stack.send(sock, buf)).unwrap();
            buf = &buf[n..];
        }
    }
}

use enumset::{enum_set, EnumSet, EnumSetType};
use mailr_nal::io::{BufReader, BufWriter, TcpStream, Write as _};
// use io::*;
use core::fmt::Debug;
use native_tls::{Certificate, TlsConnector};
use tls::TlsStack;

#[derive(EnumSetType, Debug)]
enum SmtpExtension {
    AuthPlain,
    AuthLogin,
    Size,
}

const AUTH_SET: EnumSet<SmtpExtension> =
    enum_set!(SmtpExtension::AuthLogin | SmtpExtension::AuthPlain);

fn run<T: TcpClientStack>(stack: &mut T, remote: SocketAddr) {
    let mut stream = TcpStream::new(stack, remote).unwrap();
    let mut buf = [0; 1024];

    {
        let mut stream = BufReader::new(&mut stream, &mut buf);
        let line = stream.read_line().unwrap();
        if !line.starts_with("220") {
            panic!("WAT THE HELL. NOT 220: '{}'", &line[..3]);
        }
    }

    {
        let mut stream = BufWriter::new(&mut stream, &mut buf);
        stream.write(b"EHLO localhost\r\n").unwrap();
        println!("sent EHLO");
    }

    let mut exts: EnumSet<SmtpExtension> = EnumSet::new();
    {
        let mut stream = BufReader::new(&mut stream, &mut buf);
        loop {
            let line = stream.read_line().unwrap();

            if !line.starts_with("250") {
                panic!("**EXT code {}", &line[..3])
            }

            let words = &mut line[4..].split(' ');
            let ext = words.next().unwrap();
            println!("**EXT: {}", ext);

            exts |= match ext {
                "SIZE" => SmtpExtension::Size.into(),
                "AUTH" => EnumSet::from_iter(words.map(|mech| match mech {
                    "PLAIN" => SmtpExtension::AuthPlain.into(),
                    "LOGIN" => SmtpExtension::AuthLogin.into(),
                    _ => EnumSet::empty(),
                })),
                _ => EnumSet::empty(),
            };

            if line.chars().nth(3) == Some(' ') {
                break;
            }
        }
        println!("**EXTS**: {:?}", exts);
    }

    // Auth
    'auth: {
        let auth_mechs = exts & AUTH_SET;
        if auth_mechs.is_empty() {
            println!("!NO AUTH. BUMMER");
            break 'auth;
        }

        let name = b"mock";
        let password = b"123456";

        for mech in auth_mechs.iter() {
            println!("/// {:?}", mech);
            match mech {
                SmtpExtension::AuthPlain => {
                    {
                        let mut buffer = Vec::new();

                        write!(
                            &mut buffer,
                            "\0{}\0{}",
                            str::from_utf8(name).unwrap(),
                            str::from_utf8(password).unwrap()
                        )
                        .unwrap();

                        let mut cod = [0; 1024];
                        let res = STANDARD.encode_slice(&buffer, &mut cod).unwrap();
                        let cod = &cod[..res];

                        buffer.clear();
                        write!(
                            &mut buffer,
                            "AUTH PLAIN {}\r\n",
                            str::from_utf8(cod).unwrap()
                        )
                        .unwrap();
                        BufWriter::new(&mut stream, &mut buf)
                            .write(&buffer)
                            .unwrap();
                    }
                    if BufReader::new(&mut stream, &mut buf)
                        .read_line()
                        .is_ok_and(|line| line.starts_with("235"))
                    {
                        break 'auth;
                    }
                }
                SmtpExtension::AuthLogin => {
                    {
                        BufWriter::new(&mut stream, &mut buf)
                            .write(b"AUTH LOGIN\r\n")
                            .unwrap();
                    }
                    {
                        if !BufReader::new(&mut stream, &mut buf)
                            .read_line()
                            .is_ok_and(|line| line.starts_with("334"))
                        {
                            continue;
                        }
                    }
                    {
                        let mut cod = [0; 1024];
                        let res = STANDARD.encode_slice(&name, &mut cod).unwrap();
                        let mut w = BufWriter::new(&mut stream, &mut buf);
                        w.write(&cod[..res]).unwrap();
                        w.write(b"\r\n").unwrap();
                    }
                    {
                        if !BufReader::new(&mut stream, &mut buf)
                            .read_line()
                            .is_ok_and(|line| line.starts_with("334"))
                        {
                            continue;
                        }
                    }
                    {
                        let mut cod = [0; 1024];
                        let res = STANDARD.encode_slice(&password, &mut cod).unwrap();
                        let mut w = BufWriter::new(&mut stream, &mut buf);
                        w.write(&cod[..res]).unwrap();
                        w.write(b"\r\n").unwrap();
                    }
                    {
                        if BufReader::new(&mut stream, &mut buf)
                            .read_line()
                            .is_ok_and(|line| line.starts_with("235"))
                        {
                            break 'auth;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        panic!("FAILED AUTH!!!!!!!");
    }

    // SEND MAIL
    {
        println!("SEDNING MAIL");

        let from = ("Me", "me@test.com");
        let to = ("my love", "girl@heaven.org");
        let subject = "Test mail";
        let body = concat!(
            "My love\n",
            "XXX\n",
            "One day <3\n",
            ".\n",
            ".With love.\r\n",
            ".asd\n",
            ".\r\n",
            "..hello\r\n",
            "..\r\n",
            ".\r\n",
            "..\r\n",
            // "\n"
        );

        let mut vec = Vec::new();

        write!(&mut vec, "MAIL FROM:<{}>\r\n", from.1).unwrap();
        BufWriter::new(&mut stream, &mut buf).write(&vec).unwrap();

        if !BufReader::new(&mut stream, &mut buf)
            .read_line()
            .is_ok_and(|line| line.starts_with("250"))
        {
            panic!("SEND FAILED!!!");
        }

        vec.clear();
        write!(&mut vec, "RCPT TO:<{}>\r\n", to.1).unwrap();
        stream.write_all(&vec).unwrap();

        if !BufReader::new(&mut stream, &mut buf)
            .read_line()
            .is_ok_and(|line| line.starts_with("250"))
        {
            panic!("SEND FAILED!!!");
        }

        stream.write_all(b"DATA\r\n").unwrap();

        if !BufReader::new(&mut stream, &mut buf)
            .read_line()
            .is_ok_and(|line| line.starts_with("354"))
        {
            panic!("SEND FAILED!!!");
        }

        vec.clear();
        write!(&mut vec, "From:{}<{}>\r\n", from.0, from.1).unwrap();
        stream.write_all(&vec).unwrap();
        vec.clear();
        write!(&mut vec, "To:{}<{}>\r\n", to.0, to.1).unwrap();
        stream.write_all(&vec).unwrap();
        vec.clear();
        write!(&mut vec, "Subject:{}\r\n", subject).unwrap();
        stream.write_all(&vec).unwrap();
        stream.write_all(b"\r\n").unwrap();

        fn seq_ending_pos(b: &[u8], seq: &[u8]) -> Option<usize> {
            let pos = b.windows(seq.len()).position(|w| w == seq);
            pos.map(|p| p + seq.len() - 1)
        }

        let mut rest = body.as_bytes();
        loop {
            match seq_ending_pos(rest, b"\r\n.") {
                Some(pos) => {
                    stream.write_all(&rest[..=pos]).unwrap();
                    rest = &rest[pos..];
                }
                None => {
                    stream.write_all(rest).unwrap();
                    break;
                }
            };
        }
        stream.write_all(b"\r\n.\r\n").unwrap();

        if !BufReader::new(&mut stream, &mut buf)
            .read_line()
            .is_ok_and(|line| line.starts_with("250"))
        {
            panic!("SEND FAILED!!!");
        }
    }

    // QUIT
    {
        println!("QUITING");

        stream.write_all(b"QUIT\r\n").unwrap();
        if !BufReader::new(&mut stream, &mut buf)
            .read_line()
            .is_ok_and(|line| line.starts_with("221"))
        {
            panic!("ERROR WHEN QUIT!!!!!!!");
        }
    }
}

fn main() {
    let mut stack = std_embedded_nal::Stack;
    let ip_addr = block!(stack.get_host_by_name("localhost", AddrType::Either));

    let cert = env::var("CA")
        .ok()
        .map(|path| fs::read(path).unwrap())
        .map(|cert| Certificate::from_pem(&cert).unwrap());

    if cert.is_some() || env::var("TLS").is_ok_and(|tls| tls == "1") {
        let tls = match cert {
            Some(cert) => TlsConnector::builder().add_root_certificate(cert).build(),
            None => TlsConnector::new(),
        }
        .unwrap();
        let mut stack = TlsStack::new(tls);

        run_with_stack(&mut stack, ip_addr.unwrap());
    } else {
        run_with_stack(&mut stack, ip_addr.unwrap());
    };

    fn run_with_stack<T: TcpClientStack>(stack: &mut T, ip_addr: IpAddr) {
        run(
            stack,
            (
                ip_addr,
                env::var("PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(2525),
            )
                .into(),
        );
    }

    println!("End");
}
