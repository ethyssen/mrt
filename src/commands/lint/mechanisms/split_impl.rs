use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::commands::lint::Finding;
use crate::commands::lint::Mechanism;

pub struct SplitImpl;

impl Mechanism for SplitImpl {
  fn name(&self) -> &'static str {
    "split-impl"
  }

  fn description(&self) -> &'static str {
    "Find unnecessarily split impl blocks for the same struct"
  }

  fn check(&self, path: &Path, contents: &str) -> Result<Vec<Finding>> {
    let counts = count_inherent_impls(contents);
    let mut findings = vec![];

    for (struct_name, count) in counts {
      if count > 1 {
        findings.push(Finding {
          file: path.display().to_string(),
          line: None,
          description: format!(
            "`impl {struct_name}` appears {count} times \u{2014} consider merging"
          ),
        });
      }
    }

    Ok(findings)
  }
}

/// Count inherent (non-trait) impl blocks per type name.
/// Looks for `impl TypeName` but not `impl Trait for TypeName`.
fn count_inherent_impls(contents: &str) -> HashMap<String, usize> {
  let mut counts: HashMap<String, usize> = HashMap::new();

  for line in contents.lines() {
    let trimmed = line.trim();

    // Skip comments
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
      continue;
    }

    // Must start with `impl` (possibly with `pub` or other visibility, but inherent impls
    // don't have visibility modifiers — they just start with `impl`)
    if !trimmed.starts_with("impl") {
      continue;
    }

    // Skip trait impls: `impl Trait for Type`
    if trimmed.contains(" for ") {
      continue;
    }

    // Extract the type name after `impl`
    // Pattern: `impl TypeName` or `impl<T> TypeName<T>`
    let after_impl = &trimmed[4..]; // skip "impl"

    // Skip past any generic parameters on impl itself: `impl<T>`
    let after_generics = if after_impl.trim_start().starts_with('<') {
      // Find matching '>'
      skip_generics(after_impl.trim_start())
    } else {
      after_impl
    };

    // The next token is the type name
    let type_name = after_generics
      .trim_start()
      .split(|c: char| !c.is_alphanumeric() && c != '_')
      .next()
      .unwrap_or("");

    if !type_name.is_empty() {
      *counts.entry(type_name.to_string()).or_default() += 1;
    }
  }

  counts
}

/// Skip past a `<...>` generic parameter list, handling nested angle brackets.
fn skip_generics(s: &str) -> &str {
  let mut depth = 0;
  for (i, c) in s.char_indices() {
    match c {
      '<' => depth += 1,
      '>' => {
        depth -= 1;
        if depth == 0 {
          return &s[i + 1..];
        }
      },
      _ => {},
    }
  }
  s
}
