use embedded_nal::TcpClientStack;
use mailr_nal::smtp::SmtpClient;

use crate::nal::{SockTcpClientStack, SocketAddrWrapper};

macro_rules! try_riot {
    ($e: expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return e.number() as _,
        }
    };
}

static mut TCP_STACK: SockTcpClientStack = SockTcpClientStack;

#[no_mangle]
pub extern "C" fn smtp_hello_world(a: &riot_sys::sock_tcp_ep_t) -> i32 {
    let mut buffer = [0; 1024];

    let client =
        SmtpClient::new(unsafe { &mut TCP_STACK }, &mut buffer).connect(SocketAddrWrapper::from(a));

    client.unwrap();

    0
}
