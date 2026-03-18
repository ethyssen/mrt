use std::process::Command;

use anyhow::Context;
use anyhow::Result;

use super::complaints_path;

pub fn run() -> Result<()> {
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
