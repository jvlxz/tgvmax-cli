pub mod station;
pub mod train;

use anyhow::{Context, Result};
use tgvmax_core::client::TgvmaxClient;
use tgvmax_core::models::Station;
use tgvmax_core::opendata::OpenDataClient;

use crate::cache;
use crate::cli::{Cli, Command};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Station { action } => station::run(action, cli.json, cli.refresh).await,
        Command::Train { action } => train::run(action, cli.json, cli.refresh).await,
    }
}

/// Load stations from cache (respecting `refresh`) or fetch from API.
pub async fn fetch_stations(refresh: bool) -> Result<Vec<Station>> {
    let cached = if refresh {
        None
    } else {
        cache::load_stations()?
    };
    match cached {
        Some(stations) => Ok(stations),
        None => {
            let client = OpenDataClient::new()?;
            let stations = client
                .list_stations()
                .await
                .context("Failed to fetch station list")?;
            if let Err(e) = cache::save_stations(&stations) {
                eprintln!("Warning: failed to cache station list: {e}");
            }
            Ok(stations)
        }
    }
}
