use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

mod commands;
pub mod github;
mod name_generator;
pub mod utils;
pub mod window;

use commands::BacktestsCommand;
use commands::ClaudeCommand;
use commands::CliHelpCommand;
use commands::ComplaintsCommand;
use commands::DeployCommand;
use commands::FixCommand;
use commands::ShipCommand;
use commands::StrategiesCommand;
use commands::TempStratCommand;
use commands::UpdateCommand;

#[derive(Parser)]
#[command(name = "mr-t", about = "Leverage")]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  #[command(hide = true)]
  CliHelp(CliHelpCommand),
  /// Read and analyze backtest results
  Backtests(BacktestsCommand),
  /// Launch Claude with cwd set to ~/projects
  Claude(ClaudeCommand),
  /// Manage complaints for later review and tooling improvements
  Complaints(ComplaintsCommand),
  /// Deploy updates to remote services
  Deploy(DeployCommand),
  /// Start a fix workflow for a repository
  Fix(FixCommand),
  /// Commit, push, and open a PR for the current branch
  Ship(ShipCommand),
  /// Info about trading strategies
  Strategies(StrategiesCommand),
  /// Generate a new temporary strategy crate
  TempStrat(TempStratCommand),
  /// Rebuild and reinstall mrt from source
  Update(UpdateCommand),
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::CliHelp(cmd) => cmd.execute(),
    Commands::Backtests(cmd) => cmd.execute(),
    Commands::Claude(cmd) => cmd.execute(),
    Commands::Complaints(cmd) => cmd.execute(),
    Commands::Deploy(cmd) => cmd.execute(),
    Commands::Fix(cmd) => cmd.execute(),
    Commands::Ship(cmd) => cmd.execute(),
    Commands::Strategies(cmd) => cmd.execute(),
    Commands::TempStrat(cmd) => cmd.execute(),
    Commands::Update(cmd) => cmd.execute(),
  }
}
