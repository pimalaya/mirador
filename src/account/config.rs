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
pub struct TomlAccountConfig {
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

impl TomlAccountConfig {
    /// Configure the current account configuration.
    ///
    /// This function is mostly used to replace undefined keyring
    /// entries by default ones, based on the given account name.
    pub fn configure(&mut self, #[allow(unused_variables)] account_name: &str) -> Result<()> {
        #[cfg(all(feature = "imap", feature = "keyring"))]
        if let BackendConfig::Imap(config) = &mut self.backend {
            config.auth.replace_empty_secrets(&account_name)?;
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
