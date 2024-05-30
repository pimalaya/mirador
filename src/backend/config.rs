//! # Backned configuration
//!
//! Module dedicated to backend configuration.

#[cfg(feature = "imap")]
use email::imap::config::ImapConfig;
#[cfg(feature = "maildir")]
use email::maildir::config::MaildirConfig;
use serde::{Deserialize, Serialize};

/// The backend-specific configuration.
///
/// Represents all valid backends managed by Mirador with their
/// specific configuration.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum BackendConfig {
    /// The IMAP backend configuration.
    #[cfg(feature = "imap")]
    Imap(ImapConfig),

    /// The Maildir backend configuration.
    #[cfg(feature = "maildir")]
    Maildir(MaildirConfig),
}
