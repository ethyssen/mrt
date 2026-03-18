use anyhow::Result;
use clap::Parser;

use super::Inventory;

/// List all groups
#[derive(Parser)]
pub struct GroupsCommand;

impl GroupsCommand {
  pub fn execute(self) -> Result<()> {
    let inv = Inventory::load()?;
    let mut groups: Vec<&str> = inv
      .entries
      .iter()
      .filter_map(|e| e.group.as_deref())
      .collect();
    groups.sort();
    groups.dedup();
    if groups.is_empty() {
      println!("no groups");
      return Ok(());
    }
    for g in groups {
      println!("{g}");
    }
    Ok(())
  }
}
