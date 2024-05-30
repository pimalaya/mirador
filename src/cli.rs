use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use pimalaya_tui::cli::arg::path_parser;

use crate::{
    account::command::{
        configure::ConfigureAccountCommand, doctor::DoctorAccountCommand, watch::WatchCommand,
    },
    completion::command::GenerateCompletionCommand,
    config::Config,
    manual::command::GenerateManualCommand,
};

#[derive(Parser, Debug)]
#[command(name = "mirador", author, version, about)]
#[command(propagate_version = true, infer_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<MiradorCommand>,

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

    /// Enable logs with spantrace.
    ///
    /// This is the same as running the command with `RUST_LOG=debug`
    /// environment variable.
    #[arg(long, global = true, conflicts_with = "trace")]
    pub debug: bool,

    /// Enable verbose logs with backtrace.
    ///
    /// This is the same as running the command with `RUST_LOG=trace`
    /// and `RUST_BACKTRACE=1` environment variables.
    #[arg(long, global = true, conflicts_with = "debug")]
    pub trace: bool,
}

#[derive(Subcommand, Debug)]
pub enum MiradorCommand {
    #[command(alias = "check-up", alias = "checkup", visible_alias = "check")]
    Doctor(DoctorAccountCommand),

    #[command(alias = "cfg")]
    Configure(ConfigureAccountCommand),

    #[command()]
    Watch(WatchCommand),

    #[command(arg_required_else_help = true)]
    #[command(alias = "manuals", alias = "mans")]
    Manual(GenerateManualCommand),

    #[command(arg_required_else_help = true)]
    #[command(alias = "completions")]
    Completion(GenerateCompletionCommand),
}

impl MiradorCommand {
    pub async fn execute(self, config_paths: &[PathBuf]) -> Result<()> {
        match self {
            Self::Doctor(cmd) => {
                let config = Config::from_paths_or_default(config_paths).await?;
                cmd.execute(&config).await
            }
            Self::Configure(cmd) => {
                let config = Config::from_paths_or_default(config_paths).await?;
                cmd.execute(&config).await
            }
            Self::Watch(cmd) => {
                let config = Config::from_paths_or_default(config_paths).await?;
                cmd.execute(&config).await
            }
            Self::Manual(cmd) => cmd.execute().await,
            Self::Completion(cmd) => cmd.execute().await,
        }
    }
}
