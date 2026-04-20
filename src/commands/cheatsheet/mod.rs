mod dates;
mod local_filesystem;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

/// Reference information about the trading infrastructure
#[derive(Parser)]
pub struct CheatsheetCommand {
  #[command(subcommand)]
  cmd: CheatsheetSubcommand,
}

#[derive(Subcommand)]
enum CheatsheetSubcommand {
  /// How to handle dates and times in Rust
  Dates(dates::DatesCommand),
  /// List important local directories and their purposes
  LocalFilesystem(local_filesystem::LocalFilesystemCommand),
}

impl CheatsheetCommand {
  pub fn execute(self) -> Result<()> {
    match self.cmd {
      CheatsheetSubcommand::Dates(cmd) => cmd.execute(),
      CheatsheetSubcommand::LocalFilesystem(cmd) => cmd.execute(),
    }
  }
}
