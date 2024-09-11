use mailr_nal::smtp::{SmtpClient, SmtpClientSession};

use crate::nal::{SingleSockTcpStack, SocketAddrWrapper};

#[cfg(hide)]
macro_rules! try_riot {
    ($e: expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => return e.number() as _,
        }
    };
}

#[repr(C)]
pub struct SmtpBufferSlice(pub *mut u8, pub usize);

impl AsMut<[u8]> for SmtpBufferSlice {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.0, self.1) }
    }
}

#[allow(non_camel_case_types)]
pub type smtp_session_t<'a> = SmtpClientSession<'a, SingleSockTcpStack, SmtpBufferSlice>;

#[no_mangle]
pub unsafe extern "C" fn smtp_connect(
    session: *mut smtp_session_t,
    sock: *mut riot_sys::sock_tcp_t,
    buffer: *mut u8,
    buffer_len: usize,
    remote: &riot_sys::sock_tcp_ep_t,
) -> i32 {
    let stack: &mut SingleSockTcpStack = core::mem::transmute(sock);
    let buffer = SmtpBufferSlice(buffer, buffer_len);
    let remote = SocketAddrWrapper::from(remote);

    let result = SmtpClient::new(stack, buffer).connect(remote);
    let client = result.unwrap();

    session.write(client);

    0
}

#[no_mangle]
pub unsafe extern "C" fn smtp_close(session: *mut smtp_session_t) -> i32 {
    session.read().close().unwrap();

    0
}
