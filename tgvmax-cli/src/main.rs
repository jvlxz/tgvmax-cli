mod cache;
mod cli;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::Parser;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(cli).await
}
