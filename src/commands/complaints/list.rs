use std::fs;

use anyhow::Context;
use anyhow::Result;

use super::complaints_path;

pub fn run() -> Result<()> {
  let path = complaints_path();
  if !path.exists() {
    println!("no complaints recorded");
    return Ok(());
  }

  let contents =
    fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

  print!("{contents}");

  Ok(())
}
