// This file is part of Mirador, a CLI to watch mailbox changes.
//
// Copyright (C) 2024-2026 Clément DOUIN <pimalaya.org@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::path::PathBuf;

#[cfg(feature = "imap")]
use anyhow::bail;
use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
#[cfg(feature = "imap")]
use pimalaya_toolbox::{
    config::TomlConfig,
    sasl::{Sasl, SaslAnonymous, SaslLogin, SaslMechanism, SaslPlain},
    stream::{Rustls, RustlsCrypto, Tls, TlsProvider},
};
use pimalaya_toolbox::{
    long_version,
    terminal::{
        clap::{
            args::{AccountFlag, JsonFlag, LogFlags},
            commands::{CompletionCommand, ManualCommand},
            parsers::path_parser,
        },
        printer::Printer,
    },
};

use crate::config::Config;
#[cfg(feature = "imap")]
use crate::{
    config::{RustlsCryptoConfig, SaslMechanismConfig, TlsProviderConfig},
    imap::ImapWatchCommand,
};

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author, version, about)]
#[command(long_version = long_version!())]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct MiradorCli {
    #[command(subcommand)]
    pub command: MiradorCommand,

    /// Override the default configuration file path.
    ///
    /// The given paths are shell-expanded then canonicalized (if
    /// applicable). If the first path does not point to a valid file,
    /// the wizard will propose to assist you in the creation of the
    /// configuration file. Other paths are merged with the first one,
    /// which allows you to separate your public config from your
    /// private(s) one(s).
    #[arg(short, long = "config", global = true, env = "MIRADOR_CONFIG")]
    #[arg(value_name = "PATH", value_parser = path_parser)]
    pub config_paths: Vec<PathBuf>,
    #[command(flatten)]
    pub account: AccountFlag,
    #[command(flatten)]
    pub json: JsonFlag,
    #[command(flatten)]
    pub log: LogFlags,
}

#[derive(Debug, Subcommand)]
pub enum MiradorCommand {
    Manuals(ManualCommand),
    Completions(CompletionCommand),

    #[cfg(feature = "imap")]
    Imap(ImapWatchCommand),
}

impl MiradorCommand {
    pub fn exec(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        match self {
            Self::Manuals(cmd) => cmd.execute(printer, MiradorCli::command()),
            Self::Completions(cmd) => cmd.execute(printer, MiradorCli::command()),

            #[cfg(feature = "imap")]
            Self::Imap(cmd) => {
                let config = Config::from_paths_or_default(config_paths)?;
                let (account_name, mut account_config) = config.get_account(account_name)?;

                let Some(imap_config) = account_config.imap.take() else {
                    bail!("IMAP config is missing for account `{account_name}`")
                };

                let url = imap_config.url;

                let tls = Tls {
                    provider: match imap_config.tls.provider {
                        Some(TlsProviderConfig::Rustls) => Some(TlsProvider::Rustls),
                        Some(TlsProviderConfig::NativeTls) => Some(TlsProvider::NativeTls),
                        None => None,
                    },
                    rustls: Rustls {
                        crypto: match imap_config.tls.rustls.crypto {
                            Some(RustlsCryptoConfig::Aws) => Some(RustlsCrypto::Aws),
                            Some(RustlsCryptoConfig::Ring) => Some(RustlsCrypto::Ring),
                            None => None,
                        },
                    },
                    cert: imap_config.tls.cert,
                };

                let starttls = imap_config.starttls;

                let sasl = Sasl {
                    mechanisms: imap_config
                        .sasl
                        .mechanisms
                        .into_iter()
                        .map(|m| match m {
                            SaslMechanismConfig::Login => SaslMechanism::Login,
                            SaslMechanismConfig::Plain => SaslMechanism::Plain,
                            SaslMechanismConfig::Anonymous => SaslMechanism::Anonymous,
                        })
                        .collect(),
                    anonymous: match imap_config.sasl.anonymous {
                        Some(auth) => Some(SaslAnonymous {
                            message: auth.message,
                        }),
                        None => None,
                    },
                    login: match imap_config.sasl.login {
                        Some(auth) => Some(SaslLogin {
                            username: auth.username,
                            password: auth.password.get()?,
                        }),
                        None => None,
                    },
                    plain: match imap_config.sasl.plain {
                        Some(auth) => Some(SaslPlain {
                            authzid: auth.authzid,
                            authcid: auth.authcid,
                            passwd: auth.passwd.get()?,
                        }),
                        None => None,
                    },
                };

                cmd.execute(url, tls, starttls, sasl)
            }
        }
    }
}
