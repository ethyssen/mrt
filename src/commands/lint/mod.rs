pub mod mechanisms;
mod run;

use std::path::Path;

use anyhow::Result;
use clap::Parser;

/// A single finding from a mechanism.
pub struct Finding {
  pub file: String,
  pub line: Option<usize>,
  pub description: String,
}

/// Each lint check implements this trait.
pub trait Mechanism: Send + Sync {
  fn name(&self) -> &'static str;
  fn description(&self) -> &'static str;
  fn check(&self, path: &Path, contents: &str) -> Result<Vec<Finding>>;
}

fn all_mechanisms() -> Vec<Box<dyn Mechanism>> {
  vec![
    Box::new(mechanisms::AsRefPath),
    Box::new(mechanisms::SplitImpl),
  ]
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

  /// Target path (file or directory of Rust files)
  path: Option<String>,
}

impl LintCommand {
  pub fn execute(self) -> Result<()> {
    if self.list {
      println!("Available mechanisms (run order):");
      for m in &all_mechanisms() {
        println!("  {:<16} {}", m.name(), m.description());
      }
      return Ok(());
    }

    let path = self.path.as_deref().unwrap_or(".");
    run::run(path, self.mechanism.as_deref())
  }
}
