use anyhow::Result;
use clap::Parser;

use super::Entry;
use super::Inventory;

/// Show files in a group
#[derive(Parser)]
pub struct GroupCommand {
  /// Group name
  name: String,
}

impl GroupCommand {
  pub fn execute(self) -> Result<()> {
    let inv = Inventory::load()?;
    let matches: Vec<&Entry> = inv
      .entries
      .iter()
      .filter(|e| e.group.as_deref() == Some(&self.name))
      .collect();
    if matches.is_empty() {
      println!("no entries in group '{}'", self.name);
      return Ok(());
    }
    for e in matches {
      let mut line = format!("{}  {}  {}", e.id, e.platform, e.path);
      if !e.tags.is_empty() {
        line.push_str(&format!("  tags={}", e.tags.join(",")));
      }
      println!("{line}");
    }
    Ok(())
  }
}
