use clap::Parser;
use color_eyre::Result;
use mirador::{cli::Cli, config::Config};
use pimalaya_tui::cli::tracing;

#[tokio::main]
async fn main() -> Result<()> {
    let tracing = tracing::install()?;

    #[cfg(feature = "keyring")]
    secret::keyring::set_global_service_name("mirador-cli");

    let cli = Cli::parse();

    let Some(cmd) = cli.command else {
        let config = Config::from_paths_or_default(&cli.config_paths).await?;
        println!("{config:#?}");
        return Ok(());
    };

    let res = cmd.execute(&cli.config_paths).await;

    tracing.with_debug_and_trace_notes(res)
}
