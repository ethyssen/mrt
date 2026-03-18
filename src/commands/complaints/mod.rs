mod add;
mod list;
mod open;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

/// Manage complaints for later review and tooling improvements
#[derive(Parser)]
pub struct ComplaintsCommand {
  #[command(subcommand)]
  subcommand: ComplaintsSubcommand,
}

#[derive(Subcommand)]
enum ComplaintsSubcommand {
  /// Record a new complaint
  Add {
    /// The complaint to record
    text: Vec<String>,
  },
  /// Print all recorded complaints
  List,
  /// Open the complaints file in VS Code
  Open,
}

impl ComplaintsCommand {
  pub fn execute(self) -> Result<()> {
    match self.subcommand {
      ComplaintsSubcommand::Add { text } => add::run(text),
      ComplaintsSubcommand::List => list::run(),
      ComplaintsSubcommand::Open => open::run(),
    }
  }
}

pub fn complaints_path() -> PathBuf {
  let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ethan".to_string());
  PathBuf::from(home).join(".local/share/mrt/complaints.log")
}
