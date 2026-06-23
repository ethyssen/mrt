use std::process::Command;

use anyhow::Context;
use anyhow::Result;

const HOST: &str = "krjr84";

/// Steps run as the `lewis` user (needs his bun/node env), inside ~/pdq-studio.
const LEWIS_STEPS: &[&str] = &["cd ~/pdq-studio", "git pull", "bun run build"];

/// Steps run as root after the build succeeds.
const ROOT_STEPS: &[&str] = &["sudo systemctl restart pdq-studio"];

pub fn run() -> Result<()> {
  println!("Deploying pdq-studio on {HOST}...");

  let remote_cmd = build_remote_cmd();
  let status = Command::new("ssh")
    .args([HOST, &remote_cmd])
    .status()
    .with_context(|| format!("failed to ssh to {HOST}"))?;

  if !status.success() {
    anyhow::bail!("pdq-studio deploy failed");
  }

  println!("pdq-studio deployed successfully.");
  Ok(())
}

fn build_remote_cmd() -> String {
  let lewis = format!("sudo -iu lewis bash -lc '{}'", LEWIS_STEPS.join(" && "));
  let root = ROOT_STEPS.join(" && ");
  format!("{lewis} && {root}")
}
