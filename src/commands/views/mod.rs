mod cli_arg_occurrences;
mod free_functions;
mod intent_map;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use cli_arg_occurrences::CliArgOccurrencesCmd;
use free_functions::FreeFunctionsCmd;
use intent_map::IntentMapCmd;

/// Fast, lightweight projections of a codebase.
#[derive(Parser)]
pub struct ViewsCommand {
  #[command(subcommand)]
  view: ViewKind,
}

#[derive(Subcommand)]
enum ViewKind {
  /// Scrape every (command, arg, description) triple from a CLI's --help
  CliArgOccurrences(CliArgOccurrencesCmd),

  /// List every free function (not methods) in a crate as a table sorted by
  /// name.
  FreeFunctions(FreeFunctionsCmd),

  /// Build a multi-scale intent map of a crate.
  IntentMap(IntentMapCmd),
}

impl ViewsCommand {
  pub fn execute(self) -> Result<()> {
    match self.view {
      ViewKind::CliArgOccurrences(cmd) => cmd.execute(),
      ViewKind::FreeFunctions(cmd) => cmd.execute(),
      ViewKind::IntentMap(cmd) => cmd.execute(),
    }
  }
}
