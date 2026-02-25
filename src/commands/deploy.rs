use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;

/// Deploy updates to remote services
#[derive(Parser)]
pub struct DeployCommand {
    #[command(subcommand)]
    target: DeployTarget,
}

#[derive(clap::Subcommand)]
enum DeployTarget {
    /// Update pdq-studio on krjr84 from main
    PdqStudio,
}

impl DeployCommand {
    pub fn execute(self) -> Result<()> {
        match self.target {
            DeployTarget::PdqStudio => deploy_pdq_studio(),
        }
    }
}

fn deploy_pdq_studio() -> Result<()> {
    println!("Deploying pdq-studio on krjr84...");

    let status = Command::new("ssh")
        .args([
            "krjr84",
            "sudo -iu lewis bash -lc 'cd ~/pdq-studio && git pull && bun run build' && sudo systemctl restart pdq-studio",
        ])
        .status()
        .context("failed to ssh to krjr84")?;

    if !status.success() {
        anyhow::bail!("pdq-studio deploy failed");
    }

    println!("pdq-studio deployed successfully.");
    Ok(())
}
