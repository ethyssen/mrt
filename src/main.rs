use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

mod commands;
mod name_generator;
pub mod utils;
pub mod window;

use commands::CliHelpCommand;
use commands::ClaudeCommand;
use commands::ComplaintsCommand;
use commands::DateRangeCommand;
use commands::DeployCommand;
use commands::FixCommand;
use commands::PdqCommand;
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
  #[command(hide = true)]
  CliHelp(CliHelpCommand),
  /// Launch Claude with cwd set to ~/projects
  Claude(ClaudeCommand),
  /// Manage complaints for later review and tooling improvements
  Complaints(ComplaintsCommand),
  /// Detect the earliest and latest dates in a CSV file
  DateRange(DateRangeCommand),
  /// Deploy updates to remote services
  Deploy(DeployCommand),
  /// Start a fix workflow for a repository
  Fix(FixCommand),
  /// Read and analyze PDQ backtest results
  Pdq(PdqCommand),
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
    Commands::CliHelp(cmd) => cmd.execute(),
    Commands::Claude(cmd) => cmd.execute(),
    Commands::Complaints(cmd) => cmd.execute(),
    Commands::DateRange(cmd) => cmd.execute(),
    Commands::Deploy(cmd) => cmd.execute(),
    Commands::Fix(cmd) => cmd.execute(),
    Commands::Pdq(cmd) => cmd.execute(),
    Commands::Ship(cmd) => cmd.execute(),
    Commands::TempStrat(cmd) => cmd.execute(),
    Commands::Update(cmd) => cmd.execute(),
  }
}
