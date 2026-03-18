mod daily_trades;
mod date_range;

use anyhow::Result;
use clap::Parser;

use daily_trades::DailyTradesCommand;
use date_range::DateRangeCommand;

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
}

impl BacktestsCommand {
  pub fn execute(self) -> Result<()> {
    match self.cmd {
      BacktestsSubcommand::DailyTrades(cmd) => cmd.execute(),
      BacktestsSubcommand::DateRange(cmd) => cmd.execute(),
    }
  }
}
