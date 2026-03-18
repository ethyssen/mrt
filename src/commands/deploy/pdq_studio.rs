use std::process::Command;

use anyhow::Context;
use anyhow::Result;

const HOST: &str = "krjr84";
const REMOTE_CMD: &str = "sudo -iu lewis bash -lc 'cd ~/pdq-studio && git pull && bun run build' && sudo \
            systemctl restart pdq-studio";

pub fn run() -> Result<()> {
  println!("Deploying pdq-studio on {HOST}...");

  let status = Command::new("ssh")
    .args([HOST, REMOTE_CMD])
    .status()
    .with_context(|| format!("failed to ssh to {HOST}"))?;

  if !status.success() {
    anyhow::bail!("pdq-studio deploy failed");
  }

  println!("pdq-studio deployed successfully.");
  Ok(())
}
