use embedded_nal::{nb, TcpError, TcpErrorKind};
use mailr_nal::smtp::{ConnectError, SendError};
use riot_wrappers::error::NumericError;

#[derive(Debug)]
pub struct TcpNumericError(NumericError);

impl TcpNumericError {
    pub fn number(&self) -> isize {
        self.0.number()
    }

    pub fn again_is_wouldblock(self) -> nb::Error<Self> {
        self.0.again_is_wouldblock().map(|e| Self(e))
    }
}

impl TcpError for TcpNumericError {
    fn kind(&self) -> TcpErrorKind {
        TcpErrorKind::Other
    }
}

impl From<NumericError> for TcpNumericError {
    fn from(value: NumericError) -> Self {
        Self(value)
    }
}

impl From<ConnectError<TcpNumericError>> for TcpNumericError {
    fn from(value: ConnectError<TcpNumericError>) -> Self {
        let err = match value {
            ConnectError::IoError(e) => return e,
            ConnectError::NoMem => riot_sys::ENOBUFS,
            ConnectError::FormatError => riot_sys::EPROTO,
            ConnectError::AuthFailed => riot_sys::EACCES,
            ConnectError::AuthUnsupported => riot_sys::EOPNOTSUPP,
            ConnectError::UnexpectedResponse => riot_sys::EPROTO,
        };
        NumericError::from_constant(err as _).into()
    }
}

impl From<SendError<TcpNumericError>> for TcpNumericError {
    fn from(value: SendError<TcpNumericError>) -> Self {
        let err = match value {
            SendError::IoError(e) => return e,
            SendError::NoMem => riot_sys::ENOBUFS,
            SendError::SendFailed => riot_sys::EPROTO,
            SendError::UnexpectedResponse => riot_sys::EPROTO,
        };
        NumericError::from_constant(err as _).into()
    }
}
