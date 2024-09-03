use enumset::{enum_set, EnumSet};

use super::SmtpExtension;

/// Mask of all AUTH extension flags. `&` with this to check if AUTH is supported.
pub const AUTH_EXTENSION_MASK: EnumSet<SmtpExtension> =
    enum_set!(SmtpExtension::AuthLogin | SmtpExtension::AuthPlain);
