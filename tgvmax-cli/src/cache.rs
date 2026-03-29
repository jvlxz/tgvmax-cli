use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tgvmax_core::models::Station;

use crate::config;

const CACHE_FILENAME: &str = "stations_cache.json";
const CACHE_TTL_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
struct StationCache {
    updated_at: DateTime<Utc>,
    stations: Vec<Station>,
}

fn cache_path() -> Result<PathBuf> {
    Ok(config::config_dir()?.join(CACHE_FILENAME))
}

/// Load stations from cache if the file exists and the TTL has not expired.
pub fn load_stations() -> Result<Option<Vec<Station>>> {
    let path = cache_path()?;
    let contents = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read cache at {}", path.display()));
        }
    };
    let cache: StationCache = match serde_json::from_str(&contents) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    let age = Utc::now() - cache.updated_at;
    if age.num_hours() >= CACHE_TTL_HOURS {
        return Ok(None);
    }
    Ok(Some(cache.stations))
}

/// Save stations to the cache file.
pub fn save_stations(stations: &[Station]) -> Result<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create cache dir {}", parent.display()))?;
    }
    let cache = StationCache {
        updated_at: Utc::now(),
        stations: stations.to_vec(),
    };
    let json = serde_json::to_string(&cache)?;
    fs::write(&path, json)
        .with_context(|| format!("Failed to write cache to {}", path.display()))?;
    Ok(())
}
