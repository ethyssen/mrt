use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

const MARKER_START: &str = "<!-- mrt-cli-start -->";
const MARKER_END: &str = "<!-- mrt-cli-end -->";

const INSTRUCTIONS_START: &str = "<!-- mrt-instructions-start -->";
const INSTRUCTIONS_END: &str = "<!-- mrt-instructions-end -->";

const CLAUDE_INSTRUCTIONS: &str = "\
# Claude Instructions

When something is painful, tedious, or missing from your tooling, **complain about it** using:

```
mrt complaints add <description>
```

This creates a log that Ethan reviews to fix things permanently. Complaints are cheap — silence means nothing gets fixed.
";

/// Rebuild and reinstall mrt from source
#[derive(Parser)]
pub struct UpdateCommand;

impl UpdateCommand {
  pub fn execute(self) -> Result<()> {
    let home = std::env::var("HOME")?;
    let mrt_dir = crate::utils::repo_dir()?;

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

    // 4. Build instructions section
    let instructions_section =
      format!("{INSTRUCTIONS_START}\n{CLAUDE_INSTRUCTIONS}\n{INSTRUCTIONS_END}\n");

    // 5. Remove existing instructions section if present, then append both sections
    let stripped2 = remove_section_with_markers(&stripped, INSTRUCTIONS_START, INSTRUCTIONS_END);
    let updated =
      format!("{}\n{instructions_section}\n{new_section}", stripped2.trim_end());
    fs::write(&claude_md_path, updated)
      .with_context(|| format!("failed to write {}", claude_md_path.display()))?;

    println!("CLAUDE.md updated with cli-help and Claude instructions.");

    Ok(())
  }
}

fn remove_section(text: &str) -> String {
  remove_section_with_markers(text, MARKER_START, MARKER_END)
}

fn remove_section_with_markers(text: &str, start_marker: &str, end_marker: &str) -> String {
  let start = text.find(start_marker);
  let end = text.find(end_marker);

  match (start, end) {
    (Some(s), Some(e)) => {
      let after_end = e + end_marker.len();
      let tail_start =
        if text[after_end..].starts_with('\n') { after_end + 1 } else { after_end };
      format!("{}{}", &text[..s], &text[tail_start..])
    }
    _ => text.to_owned(),
  }
}
