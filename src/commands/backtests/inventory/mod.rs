mod group;
mod groups;
mod list;
mod register;
mod tagged;

use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use serde::Serialize;

use group::GroupCommand;
use groups::GroupsCommand;
use list::ListCommand;
use register::RegisterCommand;
use tagged::TaggedCommand;

pub const INVENTORY_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/inventory.json");

#[derive(Serialize, Deserialize, Default)]
pub struct Inventory {
  pub entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
  Pdq,
  Kite,
}

impl std::fmt::Display for Platform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Platform::Pdq => write!(f, "pdq"),
      Platform::Kite => write!(f, "kite"),
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
  pub id: String,
  pub platform: Platform,
  pub path: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub group: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub tags: Vec<String>,
}

pub fn generate_id() -> String {
  uuid::Uuid::new_v4().to_string()[..8].to_string()
}

impl Inventory {
  pub fn load() -> Result<Self> {
    let path = PathBuf::from(INVENTORY_PATH);
    if !path.exists() {
      return Ok(Self::default());
    }
    let contents = std::fs::read_to_string(&path).context("failed to read inventory")?;
    serde_json::from_str(&contents).context("failed to parse inventory")
  }

  pub fn save(&self) -> Result<()> {
    let contents = serde_json::to_string_pretty(self)?;
    std::fs::write(INVENTORY_PATH, contents).context("failed to write inventory")?;
    Ok(())
  }
}

/// Manage a backtest file inventory
#[derive(Parser)]
pub struct InventoryCommand {
  #[command(subcommand)]
  cmd: InventorySubcommand,
}

#[derive(clap::Subcommand)]
enum InventorySubcommand {
  /// Register a backtest file
  Register(RegisterCommand),
  /// List all registered files
  List(ListCommand),
  /// List all groups
  Groups(GroupsCommand),
  /// Show files in a group
  Group(GroupCommand),
  /// List files with a given tag
  Tagged(TaggedCommand),
}

impl InventoryCommand {
  pub fn execute(self) -> Result<()> {
    match self.cmd {
      InventorySubcommand::Register(cmd) => cmd.execute(),
      InventorySubcommand::List(cmd) => cmd.execute(),
      InventorySubcommand::Groups(cmd) => cmd.execute(),
      InventorySubcommand::Group(cmd) => cmd.execute(),
      InventorySubcommand::Tagged(cmd) => cmd.execute(),
    }
  }
}
