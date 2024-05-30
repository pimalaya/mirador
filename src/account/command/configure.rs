use clap::Parser;
use color_eyre::{Report, Result};
#[cfg(feature = "imap")]
use email::imap::config::ImapAuthConfig;
#[cfg(feature = "imap")]
use pimalaya_tui::prompt;
use tracing::{debug, info, instrument, warn};

use crate::{
    account::arg::name::OptionalAccountNameArg, backend::config::BackendConfig, config::Config,
};

/// Configure an account.
///
/// This command is mostly used to define or reset passwords managed
/// by your global keyring. If you do not use the keyring system, you
/// can skip this command.
#[derive(Debug, Parser)]
pub struct ConfigureAccountCommand {
    #[command(flatten)]
    pub account: OptionalAccountNameArg,

    /// Reset keyring passwords.
    ///
    /// This argument will force passwords to be prompted again, then
    /// saved to your global keyring.
    #[arg(long, short)]
    pub reset: bool,
}

impl ConfigureAccountCommand {
    #[instrument(skip_all)]
    pub async fn execute(self, config: &Config) -> Result<()> {
        info!("executing configure account command");

        let (name, config) = config.into_account_config(self.account.name.as_deref())?;

        if self.reset {
            let reset = match &config.backend {
                #[cfg(feature = "imap")]
                BackendConfig::Imap(config) => Result::<_, Report>::Ok(config.auth.reset().await?),
                #[cfg(feature = "maildir")]
                BackendConfig::Maildir(_) => Result::<_, Report>::Ok(()),
            };

            if let Err(err) = reset {
                warn!("cannot reset left imap secrets: {err}");
                debug!("{err:?}");
            }
        }

        match &config.backend {
            #[cfg(feature = "imap")]
            BackendConfig::Imap(config) => match &config.auth {
                ImapAuthConfig::Passwd(config) => {
                    config
                        .configure(|| prompt::password("Left IMAP password").map_err(Into::into))
                        .await?;
                }
                #[cfg(feature = "oauth2")]
                ImapAuthConfig::OAuth2(config) => {
                    config
                        .configure(|| {
                            prompt::secret("Left IMAP OAuth 2.0 client secret").map_err(Into::into)
                        })
                        .await?;
                }
            },
            #[cfg(feature = "maildir")]
            BackendConfig::Maildir(_) => {
                //
            }
        };

        let re = if self.reset { "re" } else { "" };
        println!("Account {name} successfully {re}configured!");

        Ok(())
    }
}
