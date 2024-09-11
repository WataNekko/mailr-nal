#![allow(non_camel_case_types)]

use core::{
    ffi::{self, CStr},
    str::Utf8Error,
};

use mailr_nal::{
    message::{Envelope, Mail, Mailbox},
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

#[repr(C)]
pub struct smtp_auth_credential_t {
    username: *const ffi::c_char,
    password: *const ffi::c_char,
}

#[repr(C)]
pub struct smtp_connect_info_t {
    sock: *mut riot_sys::sock_tcp_t,
    buffer: *mut u8,
    buffer_len: usize,
    remote: *const riot_sys::sock_tcp_ep_t,
    auth: *const smtp_auth_credential_t,
}

#[no_mangle]
pub unsafe extern "C" fn smtp_connect(
    session: *mut smtp_session_t,
    info: *mut smtp_connect_info_t,
) -> ffi::c_int {
    let Some(info) = info.as_mut() else {
        panic!("AS");
    };

    let Some(remote) = info.remote.as_ref() else {
        panic!("AS");
    };

    let stack: &mut SingleSockTcpStack = core::mem::transmute(info.sock);
    let buffer = FFISlice::new(info.buffer, info.buffer_len);
    let remote = SocketAddrWrapper::from(remote);

    let result = SmtpClient::new(stack, buffer).connect(remote);
    let client = result.unwrap();

    session.write(client);

    0
}

#[no_mangle]
pub unsafe extern "C" fn smtp_close(session: *mut smtp_session_t) -> ffi::c_int {
    session.read().close().unwrap();

    0
}

fn ffi_to_str(value: *const ffi::c_char) -> Option<Result<&'static str, Utf8Error>> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(value) }.to_str())
    }
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
        let Some(address) = ffi_to_str(value.address).transpose()? else {
            return Ok(None);
        };
        let name = ffi_to_str(value.name).transpose()?;

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
) -> ffi::c_int {
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
            subject: ffi_to_str(*subject).transpose().unwrap(),
            body: ffi_to_str(*body).transpose().unwrap(),
        }
    };

    session.send(mail).unwrap();

    0
}

#[repr(C)]
pub struct mailr_envelope_t {
    pub sender_addr: *const ffi::c_char,
    pub receiver_addrs: FFISlice<*const ffi::c_char>,
}

#[no_mangle]
pub unsafe extern "C" fn smtp_send_raw(
    session: *mut smtp_session_t,
    envelope: *const mailr_envelope_t,
    data: *const ffi::c_char,
) -> ffi::c_int {
    let Some(session) = session.as_mut() else {
        panic!("WHAT")
    };

    let Some(envelope) = envelope.as_ref() else {
        panic!("ASS")
    };

    let envelope = Envelope {
        sender_addr: ffi_to_str(envelope.sender_addr).transpose().unwrap(),
        receiver_addrs: envelope
            .receiver_addrs
            .as_ref()
            .iter()
            .filter_map(|s| ffi_to_str(*s).transpose().ok().flatten()),
    };
    let Some(data) = ffi_to_str(data).transpose().unwrap() else {
        panic!("ASD");
    };

    session.send_raw(envelope, data).unwrap();

    0
}
