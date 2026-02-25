use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;

/// Rebuild and reinstall mrt from source
#[derive(Parser)]
pub struct UpdateCommand;

impl UpdateCommand {
    pub fn execute(self) -> Result<()> {
        let home = std::env::var("HOME")?;
        let mrt_dir = format!("{home}/projects/mrt");

        let status = Command::new("cargo")
            .args(["install", "--path", "."])
            .current_dir(&mrt_dir)
            .status()
            .context("failed to run cargo install")?;

        if !status.success() {
            anyhow::bail!("cargo install failed");
        }

        Ok(())
    }
}
