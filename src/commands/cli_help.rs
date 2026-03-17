use anyhow::Result;
use clap::Parser;

use crate::Cli;

/// Print CLI documentation as markdown
#[derive(Parser)]
pub struct CliHelpCommand;

impl CliHelpCommand {
  pub fn execute(self) -> Result<()> {
    let md = clap_markdown::help_markdown::<Cli>();
    print!("{md}");
    Ok(())
  }
}
