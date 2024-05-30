use std::fmt;

pub mod config;
#[cfg(feature = "wizard")]
pub mod wizard;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackendKind {
    None,
    #[cfg(feature = "imap")]
    Imap,
    #[cfg(feature = "maildir")]
    Maildir,
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            #[cfg(feature = "imap")]
            Self::Imap => write!(f, "IMAP"),
            #[cfg(feature = "maildir")]
            Self::Maildir => write!(f, "Maildir"),
        }
    }
}
