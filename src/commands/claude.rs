use std::process::Command;

use anyhow::Result;
use clap::Parser;

use crate::window;

#[derive(Parser)]
pub struct ClaudeCommand;

impl ClaudeCommand {
    pub fn execute(self) -> Result<()> {
        let home = std::env::var("HOME")?;
        let projects_dir = format!("{home}/projects");

        let _ = window::snap_active_right();

        let mut child = Command::new("claude")
            .current_dir(&projects_dir)
            .spawn()?;

        child.wait()?;

        Ok(())
    }
}
