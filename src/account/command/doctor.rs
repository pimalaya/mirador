//! # Doctor account command
//!
//! This module contains the [`clap`] command for checking up left and
//! right backends integrity of a given account.

use std::sync::Arc;

use clap::Parser;
use color_eyre::eyre::Result;
use email::backend::context::BackendContextBuilder;
#[cfg(feature = "imap")]
use email::imap::ImapContextBuilder;
#[cfg(feature = "maildir")]
use email::maildir::MaildirContextBuilder;
use pimalaya_tui::terminal::config::TomlConfig as _;
use tracing::instrument;

use crate::{
    account::arg::name::OptionalAccountNameArg, backend::config::BackendConfig, config::TomlConfig,
};

/// Check up the given account.
///
/// This command performs a checkup of the given account. It checks if
/// the configuration is valid, if backend can be created and if
/// sessions work as expected.
#[derive(Debug, Parser)]
pub struct DoctorAccountCommand {
    #[command(flatten)]
    pub account: OptionalAccountNameArg,
}

impl DoctorAccountCommand {
    #[instrument(skip_all)]
    pub async fn execute(self, config: &TomlConfig) -> Result<()> {
        let (name, config) = config.to_toml_account_config(self.account.name.as_deref())?;
        let (backend, config) = config.into_account_config(name.clone());

        match backend {
            #[cfg(feature = "imap")]
            BackendConfig::Imap(imap_config) => {
                println!("Checking IMAP integrity…");
                ImapContextBuilder::new(config.clone(), Arc::new(imap_config))
                    .with_prebuilt_credentials()
                    .await?
                    .check()
                    .await
            }
            #[cfg(feature = "maildir")]
            BackendConfig::Maildir(maildir_config) => {
                println!("Checking Maildir integrity…");
                MaildirContextBuilder::new(config.clone(), Arc::new(maildir_config))
                    .check()
                    .await
            }
        }?;

        println!("Account {name} is well configured!");

        Ok(())
    }
}
