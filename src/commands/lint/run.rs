use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

use super::Finding;
use super::all_mechanisms;

pub fn run(path: &str, mechanism_filter: Option<&str>) -> Result<()> {
  let mechanisms = all_mechanisms();
  let mechanisms: Vec<_> = match mechanism_filter {
    Some(name) => {
      let m = mechanisms.into_iter().find(|m| m.name() == name).with_context(|| {
        let names: Vec<_> = all_mechanisms().iter().map(|m| m.name()).collect();
        format!("unknown mechanism: {name}\navailable: {}", names.join(", "))
      })?;
      vec![m]
    },
    None => mechanisms,
  };

  let files = collect_rust_files(path)?;
  if files.is_empty() {
    println!("no .rs files found under {path}");
    return Ok(());
  }

  let mut total_findings = 0;

  for file in &files {
    let contents =
      fs::read_to_string(file).with_context(|| format!("failed to read {}", file.display()))?;

    for mechanism in &mechanisms {
      let findings = mechanism.check(file, &contents)?;
      total_findings += findings.len();
      for f in &findings {
        print_finding(mechanism.name(), f);
      }
    }
  }

  println!("\n{total_findings} finding(s) across {} files", files.len());
  Ok(())
}

fn print_finding(mechanism_name: &str, finding: &Finding) {
  match finding.line {
    Some(line) => println!("[{mechanism_name}] {}:{line}", finding.file),
    None => println!("[{mechanism_name}] {}", finding.file),
  }
  println!("  {}\n", finding.description);
}

fn collect_rust_files(root: &str) -> Result<Vec<PathBuf>> {
  let mut files = vec![];
  collect_recursive(Path::new(root), &mut files)?;
  files.sort();
  Ok(files)
}

fn collect_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
  if path.is_file() {
    if path.extension().is_some_and(|ext| ext == "rs") {
      files.push(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
    }
    return Ok(());
  }

  if path.is_dir() {
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
      && (name == "target" || name.starts_with('.'))
    {
      return Ok(());
    }

    for entry in
      fs::read_dir(path).with_context(|| format!("failed to read directory {}", path.display()))?
    {
      collect_recursive(&entry?.path(), files)?;
    }
  }

  Ok(())
}
