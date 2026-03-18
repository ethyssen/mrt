use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

/// Show trade counts per date from PDQ backtest results
#[derive(Parser)]
pub struct DailyTradesCommand {
  /// Path to the PDQ results directory
  #[arg(long)]
  results_dir: PathBuf,
}

impl DailyTradesCommand {
  pub fn execute(self) -> Result<()> {
    let path = self.results_dir.join("trade_matches.csv");
    let mut rdr =
      csv::Reader::from_path(&path).with_context(|| format!("failed to open {}", path.display()))?;

    let headers = rdr.headers()?.clone();
    let entry_time_idx =
      headers.iter().position(|h| h == "entry_time").context("missing entry_time column")?;

    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for record in rdr.records() {
      let record = record?;
      let entry_time = &record[entry_time_idx];
      // entry_time format: 2024-11-01T09:28:44.950000-0400
      let date = &entry_time[..10];
      *counts.entry(date.to_string()).or_default() += 1;
    }

    for (date, count) in &counts {
      println!("{date}\t{count}");
    }

    Ok(())
  }
}
