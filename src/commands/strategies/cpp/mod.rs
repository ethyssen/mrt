mod launch;
mod list;

use std::fs;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use serde::Deserialize;

#[derive(Parser)]
/// C++ strategy info
pub struct CppCommand {
  #[command(subcommand)]
  cmd: CppSubcommand,
}

#[derive(Subcommand)]
enum CppSubcommand {
  /// List all C++ strategies and their variants
  List,
  /// Register latest devel and launch a KITE backtest
  Launch(launch::LaunchArgs),
}

impl CppCommand {
  pub fn execute(self) -> Result<()> {
    match self.cmd {
      CppSubcommand::List => list::run(),
      CppSubcommand::Launch(args) => launch::run(args),
    }
  }
}

#[derive(Deserialize)]
pub(crate) struct CppStrategiesFile {
  pub strategy: Vec<CppStrategy>,
}

#[derive(Deserialize)]
pub(crate) struct CppStrategy {
  pub name: String,
  /// The repo name. Sometimes multiple strategies share a repo.
  pub repo: String,
  #[serde(default)]
  pub start_time: String,
  #[serde(default)]
  pub end_time: String,
  #[serde(default)]
  pub devel_variants: Vec<DevelVariant>,
}

#[derive(Deserialize)]
pub(crate) struct DevelVariant {
  pub name: String,
  pub description: String,
}

impl CppStrategy {
  /// Read the latest commit hash for the strategy's repo on the devel branch.
  pub fn latest_devel_commit(&self) -> String {
    let output = Command::new("gh")
      .args([
        "api",
        &format!("repos/ss151/{}/commits/devel", self.github_repo_name()),
        "--jq",
        ".sha",
      ])
      .output()
      .expect("failed to run gh");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
  }

  pub fn github_repo_name(&self) -> String {
    format!("ks-{}", self.repo)
  }
}

/// Load C++ strategies from data/cpp_strategies.toml.
pub(crate) fn load_strategies() -> Result<Vec<CppStrategy>> {
  let path = crate::utils::data_path("cpp_strategies.toml")?;
  let contents =
    fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
  let file: CppStrategiesFile =
    toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))?;
  Ok(file.strategy)
}
