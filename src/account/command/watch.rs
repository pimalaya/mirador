//! # Watch mailbox command
//!
//! This module contains the [`clap`] command for watching mailbox
//! changes of a given account.

use std::sync::Arc;

use async_ctrlc::CtrlC;
use clap::Parser;
use color_eyre::{Report, Result};
use email::backend::context::BackendContextBuilder;
#[cfg(feature = "imap")]
use email::{envelope::watch::imap::WatchImapEnvelopes, imap::ImapContextBuilder};
#[cfg(feature = "maildir")]
use email::{envelope::watch::maildir::WatchMaildirEnvelopes, maildir::MaildirContextBuilder};
use pimalaya_tui::terminal::config::TomlConfig as _;
use tokio::sync::oneshot;
use tracing::instrument;

use crate::{
    account::arg::name::OptionalAccountNameArg, backend::config::BackendConfig, config::TomlConfig,
};

/// Watch changes of the given mailbox.
#[derive(Debug, Parser)]
pub struct WatchCommand {
    #[command(flatten)]
    pub account: OptionalAccountNameArg,

    #[arg(default_value = "INBOX")]
    pub folder: String,
}

impl WatchCommand {
    #[instrument(skip_all)]
    pub async fn execute(self, config: &TomlConfig) -> Result<()> {
        let (request_shutdown, wait_for_shutdown_request) = oneshot::channel();
        let (shutdown, wait_for_shutdown) = oneshot::channel();

        let watch = async {
            let (name, config) = config.to_toml_account_config(self.account.name.as_deref())?;
            let (backend, config) = config.into_account_config(name.clone());

            let feature = match backend {
                #[cfg(feature = "imap")]
                BackendConfig::Imap(imap_config) => {
                    let ctx = ImapContextBuilder::new(config.clone(), Arc::new(imap_config))
                        .with_prebuilt_credentials()
                        .await?
                        .build()
                        .await?;

                    WatchImapEnvelopes::new_boxed(&ctx)
                }
                #[cfg(feature = "maildir")]
                BackendConfig::Maildir(maildir_config) => {
                    let ctx = MaildirContextBuilder::new(config.clone(), Arc::new(maildir_config))
                        .build()
                        .await?;

                    WatchMaildirEnvelopes::new_boxed(&ctx)
                }
            };

            feature
                .watch_envelopes(self.folder.as_str(), wait_for_shutdown_request, shutdown)
                .await?;

            Result::<(), Report>::Ok(())
        };

        let interrupt = async {
            println!("Watching folder {}, press CTRL+C to exit…", self.folder);
            CtrlC::new().expect("cannot create Ctrl+C handler").await;
            println!("Received interruption signal, stop watching…");
            request_shutdown.send(()).unwrap();
            wait_for_shutdown.await.unwrap();
            Result::<(), Report>::Ok(())
        };

        tokio::select! {
            res = interrupt => res,
            res = watch => res,
        }
    }
}
