use std::fs;
use std::io;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use serde::Deserialize;

/// Commit, push, and open a PR for the current branch
#[derive(Parser)]
pub struct ShipCommand {
  /// Commit message
  pub message: String,
}

impl ShipCommand {
  pub fn execute(self) -> Result<()> {
    run_pre_ship_checks()?;
    interactive_review()?;
    stage_and_commit(&self.message)?;
    let branch = current_branch()?;
    push(&branch)?;
    let pr_url = create_pr()?;
    enable_auto_merge();
    println!("\n{pr_url}");
    Ok(())
  }
}

fn run_pre_ship_checks() -> Result<()> {
  let Ok(contents) = fs::read_to_string(".mrt.toml") else {
    return Ok(());
  };
  let config: MrtConfig = toml::from_str(&contents).context("failed to parse .mrt.toml")?;

  for check in &config.checks {
    println!("running check: {}", check.name);
    let output = Command::new("sh")
      .args(["-c", &check.command])
      .output()
      .with_context(|| format!("failed to run check '{}'", check.name))?;

    if !output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      let stderr = String::from_utf8_lossy(&output.stderr);
      if !stdout.is_empty() {
        eprintln!("{stdout}");
      }
      if !stderr.is_empty() {
        eprintln!("{stderr}");
      }
      anyhow::bail!("check '{}' failed", check.name);
    }
  }

  Ok(())
}

fn interactive_review() -> Result<()> {
  let mut input = String::new();

  Command::new("git").args(["status"]).status().context("failed to run git status")?;
  println!("Enter to continue...");
  io::stdin().read_line(&mut input)?;

  Command::new("git").args(["diff"]).status().context("failed to run git diff")?;
  println!("Enter to continue...");
  io::stdin().read_line(&mut input)?;

  Ok(())
}

fn stage_and_commit(message: &str) -> Result<()> {
  let status = Command::new("git").args(["add", "."]).status().context("failed to run git add")?;
  if !status.success() {
    anyhow::bail!("git add failed");
  }

  let status = Command::new("git")
    .args(["commit", "-m", message])
    .status()
    .context("failed to run git commit")?;
  if !status.success() {
    anyhow::bail!("git commit failed (pre-commit hook rejected?)");
  }

  Ok(())
}

fn current_branch() -> Result<String> {
  let output = Command::new("git")
    .args(["rev-parse", "--abbrev-ref", "HEAD"])
    .output()
    .context("failed to detect current branch")?;
  if !output.status.success() {
    anyhow::bail!("failed to detect current branch");
  }
  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn push(branch: &str) -> Result<()> {
  let status = Command::new("git")
    .args(["push", "-u", "origin", branch])
    .status()
    .context("failed to run git push")?;
  if !status.success() {
    anyhow::bail!("git push failed");
  }
  Ok(())
}

fn create_pr() -> Result<String> {
  let base = resolve_base()?;
  let output = Command::new("gh")
    .args(["pr", "create", "--fill", "--base", &base])
    .output()
    .context("failed to run gh pr create")?;
  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("gh pr create failed: {stderr}");
  }
  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn enable_auto_merge() {
  let ok = Command::new("gh")
    .args(["pr", "merge", "--auto", "--squash"])
    .status()
    .is_ok_and(|s| s.success());
  if !ok {
    eprintln!("warning: could not enable auto-merge");
  }
}

fn resolve_base() -> Result<String> {
  for candidate in ["main", "master"] {
    let output = Command::new("git")
      .args(["rev-parse", "--verify", &format!("origin/{candidate}")])
      .output()
      .context("failed to run git rev-parse")?;

    if output.status.success() {
      return Ok(candidate.to_string());
    }
  }

  anyhow::bail!("could not find origin/main or origin/master");
}

#[derive(Deserialize)]
struct MrtConfig {
  #[serde(default)]
  checks: Vec<Check>,
}

#[derive(Deserialize)]
struct Check {
  name: String,
  command: String,
}
