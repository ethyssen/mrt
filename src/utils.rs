use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

pub fn clear_screen() {
  print!("\x1B[2J\x1B[H");
}

/// Return the mrt repo directory from `MRT_REPO_DIR` env var.
pub fn repo_dir() -> Result<PathBuf> {
  let dir = std::env::var("MRT_REPO_DIR")
    .context("MRT_REPO_DIR is not set — point it at the mrt repo root")?;
  Ok(PathBuf::from(dir))
}

/// Return a path under the mrt data/ directory.
pub fn data_path(filename: &str) -> Result<PathBuf> {
  Ok(repo_dir()?.join("data").join(filename))
}
