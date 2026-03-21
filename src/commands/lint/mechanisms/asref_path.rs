use std::path::Path;

use anyhow::Result;

use crate::commands::lint::Finding;
use crate::commands::lint::Mechanism;

pub struct AsRefPath;

/// Parameter name fragments that suggest a path.
const PATH_NAMES: &[&str] = &["path", "file", "dir", "folder", "filename"];

/// Concrete types that should be `impl AsRef<Path>` when the param name suggests a path.
/// Note: `&Path` is excluded — it's already the right borrowed form and is required in trait
/// method signatures where `impl AsRef<Path>` isn't allowed.
const CONCRETE_PATH_TYPES: &[&str] =
  &[": &str", ": &String", ": String", ": PathBuf", ": &PathBuf"];

impl Mechanism for AsRefPath {
  fn name(&self) -> &'static str {
    "asref-path"
  }

  fn description(&self) -> &'static str {
    "Functions accepting file paths should use impl AsRef<Path>"
  }

  fn check(&self, path: &Path, contents: &str) -> Result<Vec<Finding>> {
    let mut findings = vec![];

    for (i, line) in contents.lines().enumerate() {
      let trimmed = line.trim();

      // Skip comments
      if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        continue;
      }

      // Must be a function signature
      if !trimmed.contains("fn ") || !trimmed.contains('(') {
        continue;
      }

      // Already using AsRef<Path>
      if trimmed.contains("AsRef<Path>") {
        continue;
      }

      // Extract just the parameter list between parens
      let Some(paren_start) = trimmed.find('(') else {
        continue;
      };
      let Some(paren_end) = trimmed[paren_start..].find(')') else {
        continue;
      };
      let params = &trimmed[paren_start..paren_start + paren_end + 1];

      // Check each parameter: does it have a path-like name AND a concrete path type?
      if has_flaggable_param(params) {
        findings.push(Finding {
          file: path.display().to_string(),
          line: Some(i + 1),
          description: "consider using `impl AsRef<Path>` instead of concrete path type"
            .to_string(),
        });
      }
    }

    Ok(findings)
  }
}

/// Check if a parameter list contains a param with a path-like name and a concrete path type.
fn has_flaggable_param(params: &str) -> bool {
  // Split on commas to get rough parameter segments
  for param in params.split(',') {
    let param_lower = param.to_lowercase();

    // Does this param segment have a path-like name?
    let has_path_name = PATH_NAMES.iter().any(|name| param_lower.contains(name));
    if !has_path_name {
      continue;
    }

    // Does it use a concrete path type?
    let has_concrete_type = CONCRETE_PATH_TYPES.iter().any(|t| param.contains(t));
    if has_concrete_type {
      return true;
    }
  }
  false
}
