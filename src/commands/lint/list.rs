use anyhow::Result;

use super::registry;

pub fn run() -> Result<()> {
  let mechanisms = registry::all_mechanisms();
  println!("Available mechanisms (run order):");
  for m in &mechanisms {
    println!("  {:<16} {}", m.name(), m.description());
  }
  Ok(())
}
