mod daily_trades;
mod date_range;
mod inventory;

use anyhow::Result;
use clap::Parser;

use daily_trades::DailyTradesCommand;
use date_range::DateRangeCommand;
use inventory::InventoryCommand;

/// Read and analyze backtest results
#[derive(Parser)]
pub struct BacktestsCommand {
  #[command(subcommand)]
  cmd: BacktestsSubcommand,
}

#[derive(clap::Subcommand)]
enum BacktestsSubcommand {
  /// Show trade counts per date from PDQ backtest results
  DailyTrades(DailyTradesCommand),
  /// Detect the earliest and latest dates in a CSV file
  DateRange(DateRangeCommand),
  /// Manage a backtest file inventory
  Inventory(InventoryCommand),
}

impl BacktestsCommand {
  pub fn execute(self) -> Result<()> {
    match self.cmd {
      BacktestsSubcommand::DailyTrades(cmd) => cmd.execute(),
      BacktestsSubcommand::DateRange(cmd) => cmd.execute(),
      BacktestsSubcommand::Inventory(cmd) => cmd.execute(),
    }
  }
}
