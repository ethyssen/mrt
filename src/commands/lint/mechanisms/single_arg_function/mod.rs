use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::commands::lint::Finding;
use crate::commands::lint::Mechanism;

pub struct SingleArgFunction;

impl Mechanism for SingleArgFunction {
  fn name(&self) -> &'static str {
    "single-arg-function"
  }

  fn description(&self) -> &'static str {
    "Single-arg functions should be converted to methods on the argument type"
  }

  fn check(&self, path: &Path, contents: &str) -> Result<Vec<Finding>> {
    // Load the prompt
    let prompt = std::fs::read_to_string(
      Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/commands/lint/mechanisms/single_arg_function/prompt.txt"),
    )?;

    // Prepare the analysis request for Claude
    let analysis_prompt = format!(
      "Analyze the following Rust file and apply this rule:\n\n{}\n\nFile: {}\n\nCode:\n```rust\n{}\n```\n\nFor each function that should be converted to a method:\n1. Identify the line number, parameter name, and type name\n2. Generate the corrected code (the method definition on the type)\n\nReturn findings as a JSON array with objects having these fields:\n- line: number (line where function is defined)\n- param_name: string (name of the parameter)\n- type_name: string (name of the custom type)\n- fixed_code: string (the generated method code)\n\nReturn ONLY the JSON array, no other text.",
      prompt, path.display(), contents
    );

    // Run claude in non-interactive mode with the analysis prompt
    let output = Command::new("claude")
      .arg("-p")
      .arg(&analysis_prompt)
      .output()?;

    let response = String::from_utf8(output.stdout)?;

    // Parse the JSON findings from Claude's response
    parse_findings(&response, path)
  }
}

fn parse_findings(response: &str, path: &Path) -> Result<Vec<Finding>> {
  // Extract JSON array from response (Claude may have other text around it)
  let json_start = response.find('[');
  let json_end = response.rfind(']');

  if let (Some(start), Some(end)) = (json_start, json_end) {
    let json_str = &response[start..=end];
    if let Ok(findings_json) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
      let mut findings = vec![];
      for item in findings_json {
        if let (Some(line), Some(param_name), Some(type_name)) = (
          item.get("line").and_then(|v| v.as_u64()),
          item.get("param_name").and_then(|v| v.as_str()),
          item.get("type_name").and_then(|v| v.as_str()),
        ) {
          let fixed_code = item
            .get("fixed_code")
            .and_then(|v| v.as_str())
            .map(|s| format!("\n\nSuggested fix:\n```rust\n{}\n```", s))
            .unwrap_or_default();

          findings.push(Finding {
            file: path.display().to_string(),
            line: Some(line as usize),
            description: format!(
              "function accepts only `{}` of type `{}` — should be a method on `{}`{}",
              param_name, type_name, type_name, fixed_code
            ),
          });
        }
      }
      return Ok(findings);
    }
  }

  Ok(vec![])
}
