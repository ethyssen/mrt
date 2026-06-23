use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use serde::Deserialize;

use crate::usage::usage_log_path;

/// Inspect mrt command usage stats
#[derive(Parser)]
pub struct UsageCommand {
  #[command(subcommand)]
  subcommand: Option<UsageSubcommand>,
}

#[derive(Subcommand)]
enum UsageSubcommand {
  /// Show counts per subcommand path (default)
  Summary {
    /// Depth of subcommand path to aggregate at (1 = top-level, 2 = include nested, etc.)
    #[arg(long, default_value_t = 2)]
    depth: usize,
  },
  /// Print raw log path
  Path,
  /// Print full log
  Dump,
}

#[derive(Deserialize)]
struct Record {
  args: Vec<String>,
  #[serde(default)]
  exit_code: i32,
}

impl UsageCommand {
  pub fn execute(self) -> Result<()> {
    match self.subcommand {
      None => summary(2),
      Some(UsageSubcommand::Summary { depth }) => summary(depth),
      Some(UsageSubcommand::Path) => {
        println!("{}", usage_log_path().display());
        Ok(())
      },
      Some(UsageSubcommand::Dump) => dump(),
    }
  }
}

fn read_records() -> Result<Vec<Record>> {
  let path = usage_log_path();
  if !path.exists() {
    return Ok(Vec::new());
  }
  let f = File::open(&path)?;
  let mut out = Vec::new();
  for line in BufReader::new(f).lines() {
    let line = line?;
    if line.trim().is_empty() {
      continue;
    }
    if let Ok(r) = serde_json::from_str::<Record>(&line) {
      out.push(r);
    }
  }
  Ok(out)
}

fn summary(depth: usize) -> Result<()> {
  let records = read_records()?;
  if records.is_empty() {
    println!("no usage recorded yet at {}", usage_log_path().display());
    return Ok(());
  }
  let mut counts: BTreeMap<String, (u64, u64)> = BTreeMap::new();
  for r in &records {
    let key: Vec<&str> =
      r.args.iter().take_while(|a| !a.starts_with('-')).take(depth).map(|s| s.as_str()).collect();
    let key = if key.is_empty() { "<none>".to_string() } else { key.join(" ") };
    let entry = counts.entry(key).or_default();
    entry.0 += 1;
    if r.exit_code != 0 {
      entry.1 += 1;
    }
  }
  let mut rows: Vec<_> = counts.into_iter().collect();
  rows.sort_by_key(|b| std::cmp::Reverse(b.1.0));
  let total: u64 = rows.iter().map(|(_, (n, _))| n).sum();
  println!("{:>6}  {:>6}  command", "count", "fails");
  for (k, (n, fails)) in &rows {
    println!("{:>6}  {:>6}  {}", n, fails, k);
  }
  println!("total invocations: {}", total);
  Ok(())
}

fn dump() -> Result<()> {
  let path = usage_log_path();
  if !path.exists() {
    println!("no usage log at {}", path.display());
    return Ok(());
  }
  let contents = std::fs::read_to_string(&path)?;
  print!("{}", contents);
  Ok(())
}
