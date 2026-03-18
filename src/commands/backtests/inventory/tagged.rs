use anyhow::Result;
use clap::Parser;

use super::Entry;
use super::Inventory;

/// List files with a given tag
#[derive(Parser)]
pub struct TaggedCommand {
  /// Tag to filter by
  tag: String,
}

impl TaggedCommand {
  pub fn execute(self) -> Result<()> {
    let inv = Inventory::load()?;
    let matches: Vec<&Entry> = inv
      .entries
      .iter()
      .filter(|e| e.tags.iter().any(|t| t == &self.tag))
      .collect();
    if matches.is_empty() {
      println!("no entries tagged '{}'", self.tag);
      return Ok(());
    }
    for e in matches {
      let mut line = format!("{}  {}  {}", e.id, e.platform, e.path);
      if let Some(g) = &e.group {
        line.push_str(&format!("  group={g}"));
      }
      println!("{line}");
    }
    Ok(())
  }
}
