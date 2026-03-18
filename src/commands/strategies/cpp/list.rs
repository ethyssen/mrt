use anyhow::Result;

use super::load_strategies;

pub fn run() -> Result<()> {
  let strategies = load_strategies()?;
  for s in &strategies {
    println!("{}", s.name);
    for v in &s.devel_variants {
      println!("  {:<30} {}", v.name, v.description);
    }
  }
  Ok(())
}
