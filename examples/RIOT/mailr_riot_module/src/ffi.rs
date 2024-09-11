#![allow(non_camel_case_types)]

use core::{
    ffi::{self, CStr},
    str::Utf8Error,
};

use mailr_nal::{
    message::{Mail, Mailbox},
    smtp::{SmtpClient, SmtpClientSession},
};

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
#[derive(Clone, Copy)]
pub struct FFISlice<T> {
    pub data: *mut T,
    pub len: usize,
}

impl<T> FFISlice<T> {
    pub fn new(data: *mut T, len: usize) -> Self {
        Self { data, len }
    }
}

impl<T> AsMut<[T]> for FFISlice<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.len) }
    }
}

impl<T> AsRef<[T]> for FFISlice<T> {
    fn as_ref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data, self.len) }
    }
}

pub type smtp_session_t<'a> = SmtpClientSession<'a, SingleSockTcpStack, FFISlice<u8>>;

#[no_mangle]
pub unsafe extern "C" fn smtp_connect(
    session: *mut smtp_session_t,
    sock: *mut riot_sys::sock_tcp_t,
    buffer: *mut u8,
    buffer_len: usize,
    remote: &riot_sys::sock_tcp_ep_t,
) -> i32 {
    let stack: &mut SingleSockTcpStack = core::mem::transmute(sock);
    let buffer = FFISlice::new(buffer, buffer_len);
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct mailr_mailbox_t {
    pub address: *const ffi::c_char,
    pub name: *const ffi::c_char,
}

impl TryFrom<&mailr_mailbox_t> for Option<Mailbox<'_>> {
    type Error = Utf8Error;

    fn try_from(value: &mailr_mailbox_t) -> Result<Self, Self::Error> {
        let address = if value.address.is_null() {
            return Ok(None);
        } else {
            unsafe { CStr::from_ptr(value.address) }.to_str()?
        };

        let name = if value.name.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(value.name) }.to_str()?)
        };

        Ok(Some(Mailbox { name, address }))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct mailr_message_t {
    pub from: mailr_mailbox_t,
    pub to: FFISlice<mailr_mailbox_t>,
    pub cc: FFISlice<mailr_mailbox_t>,
    pub bcc: FFISlice<mailr_mailbox_t>,
    pub subject: *const ffi::c_char,
    pub body: *const ffi::c_char,
}

#[no_mangle]
pub unsafe extern "C" fn smtp_send(
    session: *mut smtp_session_t,
    mail: *const mailr_message_t,
) -> i32 {
    let Some(session) = session.as_mut() else {
        panic!("WHAT")
    };

    let mail = {
        let Some(mailr_message_t {
            from,
            to,
            cc,
            bcc,
            subject,
            body,
        }) = mail.as_ref()
        else {
            panic!("AS");
        };

        let into_mailbox = |mb| Option::<Mailbox>::try_from(mb).ok().flatten();

        Mail {
            from: from.try_into().unwrap(),
            to: to.as_ref().iter().filter_map(into_mailbox),
            cc: cc.as_ref().iter().filter_map(into_mailbox),
            bcc: bcc.as_ref().iter().filter_map(into_mailbox),
            subject: if subject.is_null() {
                None
            } else {
                Some(unsafe { CStr::from_ptr(*subject).to_str().unwrap() })
            },
            body: if body.is_null() {
                None
            } else {
                Some(unsafe { CStr::from_ptr(*body).to_str().unwrap() })
            },
        }
    };

    session.send(mail).unwrap();

    0
}
