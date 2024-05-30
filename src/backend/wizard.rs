use color_eyre::Result;
use email::autoconfig;
use pimalaya_tui::{prompt, wizard};

use super::{config::BackendConfig, BackendKind};

static BACKENDS: &[BackendKind] = &[
    #[cfg(feature = "imap")]
    BackendKind::Imap,
    #[cfg(feature = "maildir")]
    BackendKind::Maildir,
];

pub async fn configure(account_name: &str) -> Result<BackendConfig> {
    let backend = prompt::item("Backend to configure:", &*BACKENDS, None)?;

    let backend = match backend {
        #[cfg(feature = "imap")]
        BackendKind::Imap => {
            let email = prompt::email("Email address:", None)?;

            println!("Discovering IMAP config…");
            let autoconfig = autoconfig::from_addr(&email).await.ok();

            let config = wizard::imap::start(account_name, &email, autoconfig.as_ref()).await?;

            BackendConfig::Imap(config)
        }
        #[cfg(feature = "maildir")]
        BackendKind::Maildir => {
            let config = wizard::maildir::start(account_name)?;
            BackendConfig::Maildir(config)
        }
        _ => unreachable!(),
    };

    Ok(backend)
}
