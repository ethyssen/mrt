use anyhow::Result;
use clap::Parser;

use super::Inventory;

/// List all registered files
#[derive(Parser)]
pub struct ListCommand;

impl ListCommand {
  pub fn execute(self) -> Result<()> {
    let inv = Inventory::load()?;
    if inv.entries.is_empty() {
      println!("no entries");
      return Ok(());
    }
    for e in &inv.entries {
      let mut line = format!("{}  {}  {}", e.id, e.platform, e.path);
      if let Some(g) = &e.group {
        line.push_str(&format!("  group={g}"));
      }
      if !e.tags.is_empty() {
        line.push_str(&format!("  tags={}", e.tags.join(",")));
      }
      println!("{line}");
    }
    Ok(())
  }
}
