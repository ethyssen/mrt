use std::collections::HashSet;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;

/// Scrape every (command, arg, description) triple from a CLI's --help output.
///
/// Walks the target CLI's subcommand tree by recursively invoking `--help`,
/// then prints one flat table sorted by arg. Same arg across commands lands in
/// adjacent rows, so a description that drifted for one flag — or two different
/// flags worded identically — jumps out.
#[derive(Parser)]
pub struct CliArgOccurrencesCmd {
  /// How to invoke the target CLI, e.g. `mrt` or `cargo run -p foo --`.
  /// Everything here is run verbatim with a subcommand path and `--help`
  /// appended.
  #[arg(required = true, trailing_var_arg = true)]
  invocation: Vec<String>,

  /// Include the auto-generated --help / --version rows (off by default; they
  /// are identical everywhere and only add noise).
  #[arg(long)]
  include_builtins: bool,
}

/// One scraped (command, arg, description) triple.
struct Occurrence {
  command: String,
  arg: String,
  description: String,
}

impl CliArgOccurrencesCmd {
  pub fn execute(self) -> Result<()> {
    let mut occurrences = vec![];
    let mut visited = HashSet::new();
    self.walk(&[], &mut occurrences, &mut visited)?;

    if !self.include_builtins {
      occurrences.retain(|o| o.arg != "--help" && o.arg != "--version");
    }

    if occurrences.is_empty() {
      println!("no args found — is the invocation a clap-style CLI?");
      return Ok(());
    }

    // The sort key is the bug-finder: arg first clusters every occurrence of a
    // flag together, then description, then command.
    occurrences.sort_by(|a, b| {
      (&a.arg, &a.description, &a.command).cmp(&(&b.arg, &b.description, &b.command))
    });

    print_table(&occurrences);
    Ok(())
  }

  /// Recursively invoke `<invocation> <path...> --help`, scrape its args, and
  /// descend into every subcommand it advertises.
  fn walk(
    &self, path: &[String], occurrences: &mut Vec<Occurrence>, visited: &mut HashSet<Vec<String>>,
  ) -> Result<()> {
    if !visited.insert(path.to_vec()) {
      return Ok(());
    }

    let help = self.run_help(path)?;
    let command = if path.is_empty() { "(root)".to_string() } else { path.join(" ") };

    let parsed = parse_help(&help);
    for (arg, description) in parsed.args {
      occurrences.push(Occurrence { command: command.clone(), arg, description });
    }

    for sub in parsed.subcommands {
      // `help` is clap's built-in pseudo-subcommand; descending into it loops.
      if sub == "help" {
        continue;
      }
      let mut child = path.to_vec();
      child.push(sub);
      self.walk(&child, occurrences, visited)?;
    }

    Ok(())
  }

  fn run_help(&self, path: &[String]) -> Result<String> {
    let (program, leading) = self.invocation.split_first().context("empty invocation")?;

    let output = Command::new(program)
      .args(leading)
      .args(path)
      .arg("--help")
      // Pin a wide width so clap doesn't wrap descriptions across lines.
      .env("COLUMNS", "10000")
      .output()
      .with_context(|| format!("failed to run `{program}`"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // `--help` prints to stdout; if a command errored with no help, surface it.
    if stdout.trim().is_empty() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      if path.is_empty() {
        bail!("`{program} --help` produced no output:\n{stderr}");
      }
    }
    Ok(stdout.into_owned())
  }
}

/// Args and subcommands scraped from a single `--help` page.
struct ParsedHelp {
  /// (arg, description) — arg is the canonical long flag, else short, else the
  /// positional value name.
  args: Vec<(String, String)>,
  subcommands: Vec<String>,
}

/// Which section of the help page we're currently reading.
enum Section {
  Commands,
  Args,
  Other,
}

fn parse_help(help: &str) -> ParsedHelp {
  let mut args: Vec<(String, String)> = vec![];
  let mut subcommands = vec![];
  let mut section = Section::Other;

  for line in help.lines() {
    // Section headers are flush-left and end in a colon: "Commands:",
    // "Options:", "Arguments:", "Options (global):" etc.
    if !line.starts_with(' ') && line.trim_end().ends_with(':') {
      let header = line.to_ascii_lowercase();
      section = if header.contains("command") {
        Section::Commands
      } else if header.contains("option") || header.contains("argument") {
        Section::Args
      } else {
        Section::Other
      };
      continue;
    }

    let trimmed = line.trim_start();
    if trimmed.is_empty() {
      continue;
    }

    match section {
      Section::Commands => {
        // A subcommand entry starts with its name (alphanumeric); anything else
        // indented is a wrapped continuation of the previous description.
        if trimmed.chars().next().is_some_and(|c| c.is_alphanumeric())
          && let Some(name) = trimmed.split_whitespace().next()
        {
          subcommands.push(name.to_string());
        }
      },
      Section::Args => {
        // An arg entry starts with a flag (`-`) or a positional marker (`<`/`[`).
        // clap aligns long-only flags with extra leading spaces, so we key off
        // the marker, not the indent depth. Other indented lines are wrapped
        // continuations of the prior description.
        let is_entry = matches!(trimmed.chars().next(), Some('-' | '<' | '['));
        if is_entry {
          let (signature, description) = split_entry(trimmed);
          args.push((canonical_arg(&signature), description));
        } else if let Some(last) = args.last_mut() {
          if !last.1.is_empty() {
            last.1.push(' ');
          }
          last.1.push_str(trimmed.trim_end());
        }
      },
      Section::Other => {},
    }
  }

  ParsedHelp { args, subcommands }
}

/// Split an indented entry line into (signature, description). clap separates
/// them by a run of two-or-more spaces.
fn split_entry(line: &str) -> (String, String) {
  let trimmed = line.trim_start();
  let bytes = trimmed.as_bytes();
  for i in 0..bytes.len().saturating_sub(1) {
    if bytes[i] == b' ' && bytes[i + 1] == b' ' {
      let signature = trimmed[..i].trim_end().to_string();
      let description = trimmed[i..].trim().to_string();
      return (signature, description);
    }
  }
  (trimmed.trim_end().to_string(), String::new())
}

/// Reduce an option/argument signature to the single name we key on:
/// the long flag if present, else the short flag, else the positional name.
fn canonical_arg(signature: &str) -> String {
  // Signature looks like "-o, --output-path <PATH>" or "[TEXT]..." or
  // "<NAME>" or "--flag". Tokens are comma/space separated.
  let tokens: Vec<&str> = signature.split([',', ' ']).filter(|t| !t.is_empty()).collect();

  if let Some(long) = tokens.iter().find(|t| t.starts_with("--")) {
    return long.to_string();
  }
  if let Some(short) = tokens.iter().find(|t| t.starts_with('-') && t.len() > 1) {
    return short.to_string();
  }
  // Positional: first token, stripped of value-placeholder decoration like
  // `[NAME]...` or `<NAME>` (drop the trailing `...` before the brackets).
  tokens
    .first()
    .map(|t| t.trim_end_matches('.').trim_matches(['[', ']', '<', '>']).to_string())
    .unwrap_or_default()
}

fn print_table(occurrences: &[Occurrence]) {
  let arg_w = occurrences.iter().map(|o| o.arg.len()).max().unwrap_or(3).max(3);
  let cmd_w = occurrences.iter().map(|o| o.command.len()).max().unwrap_or(7).max(7);

  println!("{:<arg_w$}  {:<cmd_w$}  DESCRIPTION", "ARG", "COMMAND");
  for o in occurrences {
    println!("{:<arg_w$}  {:<cmd_w$}  {}", o.arg, o.command, o.description);
  }
  println!("\n{} occurrence(s)", occurrences.len());
}
