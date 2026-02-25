use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

use crate::name_generator::generate_name;
use crate::window;

/// Start a fix workflow for a repository
#[derive(Parser)]
pub struct FixCommand {
  /// Name of the repository under ~/projects/
  pub repo: String,
}

impl FixCommand {
  pub fn execute(self) -> Result<()> {
    let repo_dir = repo_dir(&self.repo);

    if !repo_dir.exists() {
      anyhow::bail!("repository not found: {}", repo_dir.display());
    }

    let base_ref = fetch_and_resolve_base(&repo_dir)?;

    let branch = loop {
      let candidate = format!("fix/{}", generate_name());
      if !branch_exists(&repo_dir, &candidate)? {
        break candidate;
      }
    };

    let worktree_dir = repo_dir.join(".worktrees").join(&branch);

    let status = Command::new("git")
      .args(["worktree", "add", "-b", &branch])
      .arg(&worktree_dir)
      .arg(&base_ref)
      .current_dir(&repo_dir)
      .status()
      .context("failed to run git worktree add")?;

    if !status.success() {
      anyhow::bail!("git worktree add failed");
    }

    println!("branch: {branch}");
    println!("worktree: {}", worktree_dir.display());

    // Snap the terminal to the right half
    let _ = window::snap_active_right();

    Command::new("code").arg(&worktree_dir).spawn()?;

    // The VS Code window title will contain the worktree folder name.
    // snap_window_left waits for the window to appear before positioning.
    let window_title = worktree_dir.file_name().unwrap().to_str().unwrap();
    let _ = window::snap_window_left(window_title);

    Ok(())
  }
}

fn repo_dir(name: &str) -> PathBuf {
  let home = std::env::var("HOME").expect("HOME not set");
  PathBuf::from(home).join("projects").join(name)
}

/// Fetch from origin and return the remote ref for main/master.
fn fetch_and_resolve_base(repo_dir: &PathBuf) -> Result<String> {
  let status = Command::new("git")
    .args(["fetch", "origin"])
    .current_dir(repo_dir)
    .status()
    .context("failed to run git fetch")?;

  if !status.success() {
    anyhow::bail!("git fetch origin failed");
  }

  // Try origin/main first, fall back to origin/master
  for candidate in ["origin/main", "origin/master"] {
    let output = Command::new("git")
      .args(["rev-parse", "--verify", candidate])
      .current_dir(repo_dir)
      .output()
      .context("failed to run git rev-parse")?;

    if output.status.success() {
      return Ok(candidate.to_string());
    }
  }

  anyhow::bail!("could not find origin/main or origin/master");
}

fn branch_exists(repo_dir: &PathBuf, branch: &str) -> Result<bool> {
  let output = Command::new("git")
    .args(["branch", "--list", branch])
    .current_dir(repo_dir)
    .output()
    .context("failed to run git branch --list")?;

  if !output.status.success() {
    anyhow::bail!("git branch --list failed");
  }

  Ok(!output.stdout.is_empty())
}
