use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

use super::Entry;
use super::Inventory;
use super::Platform;
use super::generate_id;

/// Register a backtest file
#[derive(Parser)]
pub struct RegisterCommand {
  /// Path to the file
  file: PathBuf,
  /// Backtest platform
  #[arg(long)]
  platform: Platform,
  /// Group name
  #[arg(long)]
  group: Option<String>,
  /// Tags (can be repeated)
  #[arg(long)]
  tag: Vec<String>,
}

impl RegisterCommand {
  pub fn execute(self) -> Result<()> {
    let path = self.file.canonicalize().with_context(|| format!("file not found: {}", self.file.display()))?;
    let path_str = path.to_string_lossy().to_string();

    let mut inv = Inventory::load()?;

    if inv.entries.iter().any(|e| e.path == path_str) {
      anyhow::bail!("already registered: {path_str}");
    }

    let id = generate_id();
    let entry = Entry {
      id: id.clone(),
      platform: self.platform,
      path: path_str.clone(),
      group: self.group.clone(),
      tags: self.tag.clone(),
    };
    inv.entries.push(entry);
    inv.save()?;

    let mut parts = vec![format!("{id}  platform={}", self.platform)];
    if let Some(g) = &self.group {
      parts.push(format!("group={g}"));
    }
    if !self.tag.is_empty() {
      parts.push(format!("tags={}", self.tag.join(",")));
    }
    println!("{}", parts.join("  "));
    Ok(())
  }
}
