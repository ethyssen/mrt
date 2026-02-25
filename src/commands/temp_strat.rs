use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;

use crate::name_generator::generate_name;
use crate::window;

const CARGO_TEMPLATE: &str = include_str!("../../templates/Cargo.toml.template");
const MAIN_TEMPLATE: &str = include_str!("../../templates/main.rs.template");
const RUSTFMT_TEMPLATE: &str = include_str!("../../rustfmt.toml");

const SS151_DEPS: &[&[&str]] = &[
  &["pdq", "--registry", "ss151"],
  &["lots", "--registry", "ss151", "--features", "equities"],
  &["agg-stats", "--registry", "ss151"],
  &["feature-data", "--registry", "ss151"],
];

/// Generate a new temporary strategy crate
#[derive(Parser)]
pub struct TempStratCommand;

impl TempStratCommand {
  pub fn execute(self) -> Result<()> {
    let base = temp_strats_dir();
    fs::create_dir_all(&base)?;

    let name = loop {
      let candidate = generate_name();
      if !base.join(&candidate).exists() {
        break candidate;
      }
    };

    let dir = base.join(&name);
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir)?;

    let cargo_toml = CARGO_TEMPLATE.replace("{{name}}", &name);
    fs::write(dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(dir.join(".gitignore"), "/target\n")?;
    fs::write(dir.join("rustfmt.toml"), RUSTFMT_TEMPLATE)?;
    fs::write(src_dir.join("main.rs"), MAIN_TEMPLATE)?;

    println!("{}", dir.display());

    Command::new("code").arg(&dir).arg(src_dir.join("main.rs")).spawn()?;

    let _ = window::snap_window_left(&name);

    for dep in SS151_DEPS {
      let status = Command::new("cargo")
        .arg("add")
        .args(*dep)
        .current_dir(&dir)
        .status()
        .with_context(|| format!("failed to run cargo add {}", dep[0]))?;
      if !status.success() {
        anyhow::bail!("cargo add {} failed", dep[0]);
      }
    }

    let child = Command::new("cargo")
      .args(["build", "--release"])
      .current_dir(&dir)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .stdin(Stdio::null())
      .spawn()?;
    std::mem::forget(child);

    Ok(())
  }
}

fn temp_strats_dir() -> PathBuf {
  let home = std::env::var("HOME").expect("HOME not set");
  PathBuf::from(home).join("projects/temp-strats")
}
