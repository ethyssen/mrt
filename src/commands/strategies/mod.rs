mod cpp;

use std::fs;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use serde::Deserialize;

/// Info about trading strategies
#[derive(Parser)]
pub struct StrategiesCommand {
  #[command(subcommand)]
  cmd: StrategiesSubcommand,
}

#[derive(Subcommand)]
enum StrategiesSubcommand {
  /// C++ strategy info
  Cpp(cpp::CppCommand),
}

impl StrategiesCommand {
  pub fn execute(self) -> Result<()> {
    match self.cmd {
      StrategiesSubcommand::Cpp(cmd) => cmd.execute(),
    }
  }
}

#[derive(Deserialize)]
struct CapnStratConfig {
  #[serde(default)]
  kite_user: String,
  #[serde(default)]
  kite_token: String,
}

pub(crate) fn kite_credentials() -> Result<(String, String)> {
  let home = std::env::var("HOME").context("HOME not set")?;
  let path = format!("{home}/.capn_strat.toml");
  let contents = fs::read_to_string(&path).with_context(|| {
    format!("failed to read {path} — configure kite credentials via capn-strat")
  })?;
  let config: CapnStratConfig =
    toml::from_str(&contents).with_context(|| format!("failed to parse {path}"))?;
  if config.kite_user.is_empty() || config.kite_token.is_empty() {
    anyhow::bail!("kite_user and kite_token must be set in {path}");
  }
  Ok((config.kite_user, config.kite_token))
}
