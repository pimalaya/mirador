use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::account::config::TomlAccountConfig;

/// The main configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TomlConfig {
    /// The configuration of all the accounts.
    pub accounts: HashMap<String, TomlAccountConfig>,
}

#[async_trait]
impl pimalaya_tui::terminal::config::TomlConfig for TomlConfig {
    type TomlAccountConfig = TomlAccountConfig;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn get_default_account_config(&self) -> Option<(String, Self::TomlAccountConfig)> {
        self.accounts.iter().find_map(|(name, account)| {
            account
                .default
                .filter(|default| *default)
                .map(|_| (name.to_owned(), account.clone()))
        })
    }

    fn get_account_config(&self, name: &str) -> Option<(String, Self::TomlAccountConfig)> {
        self.accounts
            .get(name)
            .map(|account| (name.to_owned(), account.clone()))
    }

    #[cfg(feature = "wizard")]
    async fn from_wizard(path: &std::path::Path) -> color_eyre::Result<Self> {
        use pimalaya_tui::terminal::{print, wizard::confirm_or_exit};

        use crate::account;

        confirm_or_exit(path)?;

        print::section("Configuring your default account");
        let mut config = TomlConfig::default();
        let (account_name, account_config) = account::wizard::configure().await?;
        config.accounts.insert(account_name, account_config);
        config.write(path)?;

        Ok(config)
    }
}
