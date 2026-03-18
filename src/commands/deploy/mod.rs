mod pdq_studio;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

/// Deploy updates to remote services
#[derive(Parser)]
pub struct DeployCommand {
  #[command(subcommand)]
  target: DeployTarget,
}

#[derive(Subcommand)]
enum DeployTarget {
  /// Update pdq-studio on krjr84 from main
  PdqStudio,
}

impl DeployCommand {
  pub fn execute(self) -> Result<()> {
    match self.target {
      DeployTarget::PdqStudio => pdq_studio::run(),
    }
  }
}
