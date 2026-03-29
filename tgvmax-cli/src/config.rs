use std::path::PathBuf;

use anyhow::{Context, Result};

const APP_NAME: &str = "tgvmax";

pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join(APP_NAME);
    Ok(dir)
}
