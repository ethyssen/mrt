mod list;
pub mod mechanisms;
mod registry;
mod run;
mod state;

use std::path::Path;

use anyhow::Result;
use clap::Parser;

/// A single finding from a mechanism.
pub struct Finding {
  pub file: String,
  pub line: Option<usize>,
  pub description: String,
}

/// What a mechanism operates on.
#[allow(dead_code)]
pub enum Scope {
  SingleFile,
}

/// Each lint check implements this trait.
pub trait Mechanism: Send + Sync {
  /// Unique name, used in CLI and state keys.
  fn name(&self) -> &'static str;

  /// Human-readable one-liner.
  fn description(&self) -> &'static str;

  /// Run the check on a single file's contents.
  /// Returns findings (possibly empty).
  fn check(&self, path: &Path, contents: &str) -> Result<Vec<Finding>>;
}

/// Sequenced code quality checks
#[derive(Parser)]
pub struct LintCommand {
  /// List available mechanisms
  #[arg(long)]
  list: bool,

  /// Run only this mechanism
  #[arg(long)]
  mechanism: Option<String>,

  /// Ignore dirty-state cache, check everything
  #[arg(long)]
  force: bool,

  /// Target path (file or directory of Rust files)
  path: Option<String>,
}

impl LintCommand {
  pub fn execute(self) -> Result<()> {
    if self.list {
      return list::run();
    }

    let path = self.path.as_deref().unwrap_or(".");
    run::run(path, self.mechanism.as_deref(), self.force)
  }
}
