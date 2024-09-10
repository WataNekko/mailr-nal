use core::{ffi::c_uchar, slice};

use mailr_nal::smtp::{SmtpClient, SmtpClientSession};

use crate::nal::{SingleSockTcpStack, SocketAddrWrapper};

macro_rules! try_riot {
    ($e: expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return e.number() as _,
        }
    };
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct smtp_session_t {
    sock: *mut SingleSockTcpStack,
    buffer: *mut u8,
}

#[no_mangle]
pub extern "C" fn testtst(s: SmtpClientSession<SingleSockTcpStack>) {}

#[no_mangle]
pub extern "C" fn smtp_connect(
    session: *mut smtp_session_t,
    sock: *mut riot_sys::sock_tcp_t,
    buffer: *mut c_uchar,
    buffer_len: usize,
    remote: &riot_sys::sock_tcp_ep_t,
) -> i32 {
    let stack: &mut SingleSockTcpStack = unsafe { core::mem::transmute(sock) };
    let buffer = unsafe { slice::from_raw_parts_mut(buffer, buffer_len) };
    let remote = SocketAddrWrapper::from(remote);

    let result = SmtpClient::new(stack, buffer).connect(remote);

    let client = result.unwrap();

    0
}
