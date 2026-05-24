use anyhow::{Context, Result};
use clap::Parser;
use qregistry_core::AppConfig;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "qregistry", version, about = "Container registry appliance — admin UI")]
struct Cli {
    /// Path to qregistry.toml
    #[arg(short, long, default_value = "/etc/qregistry/qregistry.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let cli = Cli::parse();
    let cfg = AppConfig::load(&cli.config)
        .with_context(|| format!("loading {}", cli.config.display()))?;

    let bind = cfg.server.bind.clone();
    let router = qregistry_ui::build_router(cfg);

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("bind {bind}"))?;
    tracing::info!(bind = %bind, "qregistry UI listening");

    axum::serve(listener, router).await?;
    Ok(())
}
