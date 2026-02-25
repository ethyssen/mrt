use std::fs;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use serde::Deserialize;

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

/// Commit, push, and open a PR for the current branch
#[derive(Parser)]
pub struct ShipCommand {
  /// Commit message
  pub message: String,
}

impl ShipCommand {
  pub fn execute(self) -> Result<()> {
    // Run pre-ship checks from .mrt.toml if present
    if let Ok(contents) = fs::read_to_string(".mrt.toml") {
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
    }

    // Show diff for review
    Command::new("git").args(["status"]).status().context("failed to run git status")?;

    // wait for enter key to move on:
    println!("Enter to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Command::new("git").args(["diff"]).status().context("failed to run git diff")?;
    println!("Enter to continue...");
    std::io::stdin().read_line(&mut input)?;

    // Stage all changes
    let status =
      Command::new("git").args(["add", "."]).status().context("failed to run git add")?;

    if !status.success() {
      anyhow::bail!("git add failed");
    }

    // Commit
    let status = Command::new("git")
      .args(["commit", "-m", &self.message])
      .status()
      .context("failed to run git commit")?;

    if !status.success() {
      anyhow::bail!("git commit failed (pre-commit hook rejected?)");
    }

    // Detect current branch
    let branch_output = Command::new("git")
      .args(["rev-parse", "--abbrev-ref", "HEAD"])
      .output()
      .context("failed to detect current branch")?;

    if !branch_output.status.success() {
      anyhow::bail!("failed to detect current branch");
    }

    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

    // Push
    let status = Command::new("git")
      .args(["push", "-u", "origin", &branch])
      .status()
      .context("failed to run git push")?;

    if !status.success() {
      anyhow::bail!("git push failed");
    }

    // Detect base branch (main or master)
    let base = resolve_base()?;

    // Create PR
    let pr_output = Command::new("gh")
      .args(["pr", "create", "--fill", "--base", &base])
      .output()
      .context("failed to run gh pr create")?;

    if !pr_output.status.success() {
      let stderr = String::from_utf8_lossy(&pr_output.stderr);
      anyhow::bail!("gh pr create failed: {stderr}");
    }

    let pr_url = String::from_utf8_lossy(&pr_output.stdout).trim().to_string();

    // Enable auto-merge
    let status = Command::new("gh")
      .args(["pr", "merge", "--auto", "--squash"])
      .status()
      .context("failed to run gh pr merge --auto")?;

    if !status.success() {
      eprintln!("warning: could not enable auto-merge");
    }

    println!("\n{pr_url}");

    Ok(())
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
