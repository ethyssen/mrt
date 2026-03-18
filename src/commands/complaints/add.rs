use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;

use super::complaints_path;

pub fn run(text: Vec<String>) -> Result<()> {
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

fn timestamp() -> String {
  Command::new("date")
    .arg("+%Y-%m-%d %H:%M:%S")
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok())
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|| "unknown".to_string())
}
