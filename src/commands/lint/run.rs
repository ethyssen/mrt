use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

use super::Finding;
use super::registry;
use super::state::LintState;
use super::state::sha256_hex;

pub fn run(path: impl AsRef<Path>, mechanism_filter: Option<&str>, force: bool) -> Result<()> {
  let mechanisms = registry::all_mechanisms();
  let mechanisms: Vec<_> = match mechanism_filter {
    Some(name) => {
      let m = mechanisms.into_iter().find(|m| m.name() == name).with_context(|| {
        let names: Vec<_> = registry::all_mechanisms().iter().map(|m| m.name()).collect();
        format!("unknown mechanism: {name}\navailable: {}", names.join(", "))
      })?;
      vec![m]
    },
    None => mechanisms,
  };

  let files = collect_rust_files(&path)?;
  if files.is_empty() {
    println!("no .rs files found under {}", path.as_ref().display());
    return Ok(());
  }

  let mut state = LintState::load()?;
  let mut total_findings = 0;
  let mut total_files = 0;
  let mut cached_skips = 0;

  for file in &files {
    let file_str = file.display().to_string();
    let contents =
      fs::read_to_string(file).with_context(|| format!("failed to read {}", file.display()))?;
    let hash = sha256_hex(&contents);
    total_files += 1;

    for mechanism in &mechanisms {
      if !force
        && let Some(stored) = state.get(mechanism.name(), &file_str)
        && stored == hash
      {
        cached_skips += 1;
        continue;
      }

      let findings = mechanism.check(file, &contents)?;
      if findings.is_empty() {
        state.set(mechanism.name(), &file_str, &hash);
      } else {
        total_findings += findings.len();
        for f in &findings {
          print_finding(mechanism.name(), f);
        }
      }
    }
  }

  state.save()?;
  println!(
    "\n{total_findings} finding(s) across {total_files} files ({cached_skips} cached as clean)"
  );
  Ok(())
}

fn print_finding(mechanism_name: &str, finding: &Finding) {
  match finding.line {
    Some(line) => println!("[{mechanism_name}] {}:{line}", finding.file),
    None => println!("[{mechanism_name}] {}", finding.file),
  }
  println!("  {}\n", finding.description);
}

fn collect_rust_files(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
  let mut files = vec![];
  collect_recursive(root, &mut files)?;
  files.sort();
  Ok(files)
}

fn collect_recursive(path: impl AsRef<Path>, files: &mut Vec<PathBuf>) -> Result<()> {
  let path = path.as_ref();
  if path.is_file() {
    if path.extension().is_some_and(|ext| ext == "rs") {
      files.push(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    }
    return Ok(());
  }

  if path.is_dir() {
    // Skip target directories and hidden directories
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
      && (name == "target" || name.starts_with('.'))
    {
      return Ok(());
    }

    for entry in
      fs::read_dir(path).with_context(|| format!("failed to read directory {}", path.display()))?
    {
      let entry = entry?;
      collect_recursive(entry.path(), files)?;
    }
  }

  Ok(())
}
