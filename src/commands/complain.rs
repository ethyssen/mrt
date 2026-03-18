use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

/// Manage complaints for later review and tooling improvements
#[derive(Parser)]
pub struct ComplaintsCommand {
  #[command(subcommand)]
  subcommand: ComplaintsSubcommand,
}

#[derive(Subcommand)]
enum ComplaintsSubcommand {
  /// Record a new complaint
  Add {
    /// The complaint to record
    text: Vec<String>,
  },
  /// Print all recorded complaints
  List,
  /// Open the complaints file in VS Code
  Open,
}

impl ComplaintsCommand {
  pub fn execute(self) -> Result<()> {
    match self.subcommand {
      ComplaintsSubcommand::Add { text } => add(text),
      ComplaintsSubcommand::List => list(),
      ComplaintsSubcommand::Open => open(),
    }
  }
}

fn add(text: Vec<String>) -> Result<()> {
  let text = text.join(" ");
  if text.is_empty() {
    anyhow::bail!("complaint text cannot be empty");
  }

  let path = complaints_path();
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).context("failed to create complaints directory")?;
  }

  let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&path)
    .with_context(|| format!("failed to open {}", path.display()))?;

  let timestamp = timestamp();
  writeln!(file, "[{timestamp}] {text}").context("failed to write complaint")?;

  println!("logged: {text}");

  Ok(())
}

fn list() -> Result<()> {
  let path = complaints_path();
  if !path.exists() {
    println!("no complaints recorded");
    return Ok(());
  }

  let contents = fs::read_to_string(&path)
    .with_context(|| format!("failed to read {}", path.display()))?;

  print!("{contents}");

  Ok(())
}

fn open() -> Result<()> {
  let path = complaints_path();
  if !path.exists() {
    println!("no complaints recorded");
    return Ok(());
  }

  Command::new("code")
    .arg(&path)
    .spawn()
    .with_context(|| format!("failed to open {} in VS Code", path.display()))?;

  Ok(())
}

fn complaints_path() -> PathBuf {
  let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ethan".to_string());
  PathBuf::from(home).join(".local/share/mrt/complaints.log")
}

fn timestamp() -> String {
  Command::new("date")
    .arg("+%Y-%m-%d %H:%M:%S")
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|| "unknown".to_string())
}
