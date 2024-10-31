use clap::Parser;
use color_eyre::Result;
use mirador::{cli::Cli, config::TomlConfig};
use pimalaya_tui::terminal::{cli::tracing, config::TomlConfig as _};

#[tokio::main]
async fn main() -> Result<()> {
    let tracing = tracing::install()?;

    #[cfg(feature = "keyring")]
    keyring::set_global_service_name("mirador-cli");

    let cli = Cli::parse();

    let Some(cmd) = cli.command else {
        let config = TomlConfig::from_paths_or_default(&cli.config_paths).await?;
        println!("{config:#?}");
        return Ok(());
    };

    let res = cmd.execute(&cli.config_paths).await;

    tracing.with_debug_and_trace_notes(res)
}
