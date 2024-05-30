//! # Account configuration
//!
//! Module dedicated to account configuration.

use std::sync::Arc;

use color_eyre::eyre::Result;
use email::watch::config::WatchHook;
use serde::{Deserialize, Serialize};

use crate::backend::config::BackendConfig;

/// The account configuration.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountConfig {
    /// The defaultness of the current account.
    ///
    /// When watching, if no account name is explicitly given, this
    /// one will be used by default.
    pub default: Option<bool>,

    /// The name of the mailbox to watch changes for.
    ///
    /// One account configuration allows to watch only one folder. If
    /// you need to watch multiple folder, then create multiple
    /// accounts (one per folder).
    pub folder: Option<String>,

    /// The backend configuration.
    pub backend: BackendConfig,

    /// The message added watch hook.
    ///
    /// Hook to execute when a new message arrives in the configured
    /// mailbox.
    pub on_message_added: Option<WatchHook>,
}

impl AccountConfig {
    /// Configure the current account configuration.
    ///
    /// This function is mostly used to replace undefined keyring
    /// entries by default ones, based on the given account name.
    pub fn configure(&mut self, _account_name: &str) -> Result<()> {
        match &mut self.backend {
            #[cfg(feature = "imap")]
            BackendConfig::Imap(_config) => {
                #[cfg(feature = "keyring")]
                _config
                    .auth
                    .replace_undefined_keyring_entries(&_account_name)?;
            }
            #[cfg(feature = "maildir")]
            BackendConfig::Maildir(_) => {
                //
            }
        }

        Ok(())
    }

    pub fn into_account_config(
        self,
        name: String,
    ) -> (BackendConfig, Arc<email::account::config::AccountConfig>) {
        (
            self.backend,
            Arc::new(email::account::config::AccountConfig {
                name,
                envelope: Some(email::envelope::config::EnvelopeConfig {
                    watch: Some(email::envelope::watch::config::WatchEnvelopeConfig {
                        received: self.on_message_added,
                        any: None,
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        )
    }
}
