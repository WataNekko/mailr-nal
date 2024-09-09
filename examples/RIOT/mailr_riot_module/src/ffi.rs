use mailr_nal::smtp::SmtpClient;

use crate::nal::{SingleSockTcpStack, SocketAddrWrapper};

macro_rules! try_riot {
    ($e: expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return e.number() as _,
        }
    };
}

#[no_mangle]
pub extern "C" fn smtp_hello_world(
    t: *mut riot_sys::sock_tcp_t,
    a: &riot_sys::sock_tcp_ep_t,
) -> i32 {
    let mut buffer = [0; 1024];
    let stack: &mut SingleSockTcpStack = unsafe { core::mem::transmute(t) };

    let client = SmtpClient::new(stack, &mut buffer).connect(SocketAddrWrapper::from(a));

    client.unwrap();

    0
}
