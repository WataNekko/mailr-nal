pub mod auth;

use enumset::{EnumSet, EnumSetType};

/// Enum containing all SMTP extension flags.
#[derive(EnumSetType, Debug)]
pub enum SmtpExtension {
    AuthPlain,
    AuthLogin,
}

pub struct EhloInfo {
    pub extensions: EnumSet<SmtpExtension>,
}
