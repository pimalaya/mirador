use color_eyre::Result;
use email::watch::config::{WatchHook, WatchNotifyConfig};
use pimalaya_tui::terminal::prompt;

use crate::backend;

use super::config::TomlAccountConfig;

pub async fn configure() -> Result<(String, TomlAccountConfig)> {
    let name = prompt::text("Account name:", Some("personal"))?;
    let folder = prompt::text("Folder to watch:", Some("INBOX"))?;
    let hook = WatchHook {
        notify: if prompt::bool("Send system notification on new message?", true)? {
            Some(WatchNotifyConfig {
                summary: prompt::text("Notification title:", Some("📫 New message from {sender}"))?,
                body: prompt::text("Notification body:", Some("{subject}"))?,
            })
        } else {
            None
        },
        cmd: if prompt::bool("Execute shell command on new message?", false)? {
            prompt::some_text("Shell command:", None)?.map(Into::into)
        } else {
            None
        },
        callback: None,
    };

    let config = TomlAccountConfig {
        default: Some(true),
        folder: Some(folder),
        on_message_added: Some(hook),
        backend: backend::wizard::configure(&name).await?,
    };

    Ok((name, config))
}
