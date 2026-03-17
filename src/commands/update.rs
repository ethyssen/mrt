use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

const MARKER_START: &str = "<!-- mrt-cli-start -->";
const MARKER_END: &str = "<!-- mrt-cli-end -->";

/// Rebuild and reinstall mrt from source
#[derive(Parser)]
pub struct UpdateCommand;

impl UpdateCommand {
  pub fn execute(self) -> Result<()> {
    let home = std::env::var("HOME")?;
    let mrt_dir = format!("{home}/projects/mrt");

    // 0. Install self
    let status = Command::new("cargo")
      .args(["install", "--path", "."])
      .current_dir(&mrt_dir)
      .status()
      .context("failed to run cargo install")?;

    if !status.success() {
      anyhow::bail!("cargo install failed");
    }

    // 1. Generate help text from the newly installed binary
    let output = Command::new("mrt")
      .arg("cli-help")
      .output()
      .context("failed to run `mrt cli-help`")?;

    if !output.status.success() {
      anyhow::bail!("`mrt cli-help` failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let help_md = String::from_utf8(output.stdout).context("`mrt cli-help` output was not valid UTF-8")?;
    let new_section = format!("{MARKER_START}\n{help_md}\n{MARKER_END}\n");

    // 2. Read CLAUDE.md
    let claude_md_path = PathBuf::from(&home).join(".claude/CLAUDE.md");
    let existing = fs::read_to_string(&claude_md_path)
      .with_context(|| format!("failed to read {}", claude_md_path.display()))?;

    // 3. Remove existing help portion if present
    let stripped = remove_section(&existing);

    // 4. Append new section at the bottom
    let updated = format!("{}\n{new_section}", stripped.trim_end());
    fs::write(&claude_md_path, updated)
      .with_context(|| format!("failed to write {}", claude_md_path.display()))?;

    println!("CLAUDE.md updated with cli-help.");

    Ok(())
  }
}

fn remove_section(text: &str) -> String {
  let start = text.find(MARKER_START);
  let end = text.find(MARKER_END);

  match (start, end) {
    (Some(s), Some(e)) => {
      let after_end = e + MARKER_END.len();
      // Trim a single trailing newline after the end marker if present
      let tail_start = if text[after_end..].starts_with('\n') { after_end + 1 } else { after_end };
      format!("{}{}", &text[..s], &text[tail_start..])
    }
    _ => text.to_owned(),
  }
}
