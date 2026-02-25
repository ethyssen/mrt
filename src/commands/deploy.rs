use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

// ── Deploy targets ──────────────────────────────────────────────────
// Each entry defines a deploy target: (variant name, help text, host, remote command)

struct Target {
  host: &'static str,
  command: &'static str,
}

const PDQ_STUDIO: Target = Target {
  host: "krjr84",
  command: "sudo -iu lewis bash -lc 'cd ~/pdq-studio && git pull && bun run build' && sudo \
            systemctl restart pdq-studio",
};

// ─────────────────────────────────────────────────────────────────────

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
      DeployTarget::PdqStudio => deploy(&PDQ_STUDIO, "pdq-studio"),
    }
  }
}

fn deploy(target: &Target, name: &str) -> Result<()> {
  println!("Deploying {name} on {}...", target.host);

  let status = Command::new("ssh")
    .args([target.host, target.command])
    .status()
    .with_context(|| format!("failed to ssh to {}", target.host))?;

  if !status.success() {
    anyhow::bail!("{name} deploy failed");
  }

  println!("{name} deployed successfully.");
  Ok(())
}
