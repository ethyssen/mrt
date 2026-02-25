use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

mod commands;
mod name_generator;
pub mod utils;
pub mod window;

use commands::ClaudeCommand;
use commands::DeployCommand;
use commands::FixCommand;
use commands::ShipCommand;
use commands::TempStratCommand;
use commands::UpdateCommand;

#[derive(Parser)]
#[command(name = "mr-t", about = "Trading strategy development utilities")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Launch Claude with cwd set to ~/projects
  Claude(ClaudeCommand),
  /// Deploy updates to remote services
  Deploy(DeployCommand),
  /// Start a fix workflow for a repository
  Fix(FixCommand),
  /// Commit, push, and open a PR for the current branch
  Ship(ShipCommand),
  /// Generate a new temporary strategy crate
  TempStrat(TempStratCommand),
  /// Rebuild and reinstall mrt from source
  Update(UpdateCommand),
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Claude(cmd) => cmd.execute(),
    Commands::Deploy(cmd) => cmd.execute(),
    Commands::Fix(cmd) => cmd.execute(),
    Commands::Ship(cmd) => cmd.execute(),
    Commands::TempStrat(cmd) => cmd.execute(),
    Commands::Update(cmd) => cmd.execute(),
  }
}
