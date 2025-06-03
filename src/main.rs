mod app;
mod client;
mod config;
mod error;
mod handlers;
mod response;
mod state;

use anyhow::Context;
use clap::Parser;

use crate::{app::ApplicationServer, config::AppConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::parse();

    config.log.init();

    ApplicationServer::serve(config)
        .await
        .context("could not initialize application routes")?;

    Ok(())
}
