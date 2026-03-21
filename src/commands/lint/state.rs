use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

/// Persistent checksum state for dirty detection.
/// Key: (mechanism_name, absolute_file_path) -> sha256 hex of file contents when last clean.
#[derive(Serialize, Deserialize, Default)]
pub struct LintState {
  /// mechanism_name -> { absolute_file_path -> sha256_hex }
  pub checksums: HashMap<String, HashMap<String, String>>,
}

impl LintState {
  fn state_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let dir = PathBuf::from(home).join(".local/share/mrt");
    Ok(dir.join("lint_state.json"))
  }

  pub fn load() -> Result<Self> {
    let path = Self::state_path()?;
    if !path.exists() {
      return Ok(Self::default());
    }
    let contents =
      fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let state: Self = serde_json::from_str(&contents)
      .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(state)
  }

  pub fn save(&self) -> Result<()> {
    let path = Self::state_path()?;
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)
        .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(self)?;
    fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
  }

  pub fn get(&self, mechanism: &str, file: impl AsRef<Path>) -> Option<&str> {
    self
      .checksums
      .get(mechanism)
      .and_then(|files| files.get(file.as_ref().to_str().unwrap()))
      .map(|s| s.as_str())
  }

  pub fn set(&mut self, mechanism: &str, file: impl AsRef<Path>, hash: &str) {
    self
      .checksums
      .entry(mechanism.to_string())
      .or_default()
      .insert(file.as_ref().to_string_lossy().to_string(), hash.to_string());
  }
}

pub fn sha256_hex(contents: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(contents.as_bytes());
  format!("{:x}", hasher.finalize())
}
