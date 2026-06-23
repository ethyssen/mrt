mod cli_arg_occurrences;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use cli_arg_occurrences::CliArgOccurrencesArgs;

/// A library of fast, lightweight projections of a codebase or CLI.
///
/// Each view keeps one dimension and drops the rest, then tables the
/// occurrences so the odd-one-out is obvious.
#[derive(Parser)]
pub struct ViewsCommand {
  #[command(subcommand)]
  view: ViewKind,
}

#[derive(Subcommand)]
enum ViewKind {
  /// Scrape every (command, arg, description) triple from a CLI's --help and
  /// table them sorted by arg so descriptions that drift for the same flag pop
  CliArgOccurrences(CliArgOccurrencesArgs),
}

impl ViewsCommand {
  pub fn execute(self) -> Result<()> {
    match self.view {
      ViewKind::CliArgOccurrences(args) => args.execute(),
    }
  }
}
