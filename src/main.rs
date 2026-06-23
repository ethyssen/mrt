use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

mod commands;
pub mod github;
mod name_generator;
pub mod usage;
pub mod utils;
pub mod window;

use commands::BacktestsCommand;
use commands::CheatsheetCommand;
use commands::ClaudeCommand;
use commands::CliHelpCommand;
use commands::ComplaintsCommand;
use commands::DeployCommand;
use commands::FixCommand;
use commands::LintCommand;
use commands::ShipCommand;
use commands::StrategiesCommand;
use commands::TempStratCommand;
use commands::UpdateCommand;
use commands::UsageCommand;
use commands::ViewsCommand;

#[derive(Parser)]
#[command(
  name = "mrt",
  about = "Personal leverage for Ethan and AI as he builds out ecosystem software"
)]
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
  /// Reference information about the trading infrastructure
  Cheatsheet(CheatsheetCommand),
  /// Sequenced code quality checks
  Lint(LintCommand),
  /// Commit, push, and open a PR for the current branch
  Ship(ShipCommand),
  /// Info about trading strategies
  Strategies(StrategiesCommand),
  /// Generate a new temporary strategy crate
  TempStrat(TempStratCommand),
  /// Rebuild and reinstall mrt from source
  Update(UpdateCommand),
  /// Inspect mrt command usage stats
  Usage(UsageCommand),
  /// Fast, lightweight projections of a codebase or CLI that surface bugs
  Views(ViewsCommand),
}

fn main() {
  let start = std::time::Instant::now();
  let cli = Cli::parse();

  let result: Result<()> = match cli.command {
    Commands::CliHelp(cmd) => cmd.execute(),
    Commands::Backtests(cmd) => cmd.execute(),
    Commands::Claude(cmd) => cmd.execute(),
    Commands::Complaints(cmd) => cmd.execute(),
    Commands::Deploy(cmd) => cmd.execute(),
    Commands::Fix(cmd) => cmd.execute(),
    Commands::Cheatsheet(cmd) => cmd.execute(),
    Commands::Lint(cmd) => cmd.execute(),
    Commands::Ship(cmd) => cmd.execute(),
    Commands::Strategies(cmd) => cmd.execute(),
    Commands::TempStrat(cmd) => cmd.execute(),
    Commands::Update(cmd) => cmd.execute(),
    Commands::Usage(cmd) => cmd.execute(),
    Commands::Views(cmd) => cmd.execute(),
  };

  let exit_code = if let Err(ref e) = result {
    eprintln!("Error: {e:?}");
    1
  } else {
    0
  };
  usage::record_invocation(exit_code, start.elapsed().as_millis());
  std::process::exit(exit_code);
}
