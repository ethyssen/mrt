use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use walkdir::WalkDir;

/// List every free function in a crate, tabled by name.
///
/// A "free function" is one declared at module scope — `fn foo()` directly in a
/// file or inside an inline `mod` block. We deliberately exclude methods (`fn`s
/// inside an `impl` or `trait`) and functions nested inside other function
/// bodies, since those are not independently callable by bare name.
///
/// Sorting by name first is the whole point: two functions that do almost the
/// same thing under almost the same name, but live in different files, end up
/// adjacent so the redundancy is obvious.
#[derive(Parser)]
pub struct FreeFunctionsArgs {
  /// Crate (or directory) to scan. If it looks like a crate root we scan its
  /// `src/`; otherwise we walk every `.rs` file beneath the path.
  #[arg(default_value = ".")]
  path: PathBuf,
}

/// One free function, located.
struct FreeFunction {
  name: String,
  // Relative to the scan root, so the table stays readable.
  file: String,
  line: usize,
}

impl FreeFunctionsArgs {
  pub fn execute(self) -> Result<()> {
    // A crate root carries a `Cargo.toml`, in which case the code we care about
    // lives under `src/`. Otherwise we treat the path as a plain tree to walk.
    let scan_root = if self.path.join("Cargo.toml").is_file() && self.path.join("src").is_dir() {
      self.path.join("src")
    } else {
      self.path.clone()
    };

    let mut functions = vec![];
    for entry in WalkDir::new(&scan_root).into_iter().filter_map(|e| e.ok()) {
      let file_path = entry.path();
      if file_path.extension().is_none_or(|ext| ext != "rs") {
        continue;
      }
      collect_from_file(file_path, &scan_root, &mut functions)
        .with_context(|| format!("parsing {}", file_path.display()))?;
    }

    if functions.is_empty() {
      println!("no free functions found under {}", scan_root.display());
      return Ok(());
    }

    // Name first clusters the near-duplicates; file then line break ties so the
    // ordering is stable and points you straight at the source.
    functions.sort_by(|a, b| (&a.name, &a.file, a.line).cmp(&(&b.name, &b.file, b.line)));

    print_table(&functions);
    Ok(())
  }
}

/// Parse a single file and append its free functions to `out`.
fn collect_from_file(
  path: impl AsRef<Path>,
  root: impl AsRef<Path>,
  out: &mut Vec<FreeFunction>,
) -> Result<()> {
  let path = path.as_ref();
  let root = root.as_ref();
  let source = std::fs::read_to_string(path)?;
  let ast = syn::parse_file(&source)?;

  // Display the file relative to the scan root when we can; fall back to the
  // full path for anything that somehow sits outside it.
  let display = path.strip_prefix(root).unwrap_or(path).display().to_string();

  collect_from_items(&ast.items, &display, out);
  Ok(())
}

/// Walk a list of items, recording free functions and descending into inline
/// modules. We do *not* descend into `impl`/`trait` blocks — their `fn`s are
/// methods, not free functions — nor into function bodies.
fn collect_from_items(items: &[syn::Item], origin: &str, out: &mut Vec<FreeFunction>) {
  for item in items {
    match item {
      syn::Item::Fn(item_fn) => {
        // We key the line off the name's span rather than the `fn` keyword or a
        // leading attribute, so the table points at the identifier itself.
        out.push(FreeFunction {
          name: item_fn.sig.ident.to_string(),
          file: origin.to_string(),
          line: item_fn.sig.ident.span().start().line,
        });
      },
      // An inline `mod foo { ... }` still holds free functions; a `mod foo;`
      // (no content) points at another file we'll reach via the walk.
      syn::Item::Mod(item_mod) => {
        if let Some((_, nested)) = &item_mod.content {
          collect_from_items(nested, origin, out);
        }
      },
      _ => {},
    }
  }
}

fn print_table(functions: &[FreeFunction]) {
  let name_w = functions.iter().map(|f| f.name.len()).max().unwrap_or(8).max(8);
  let file_w = functions.iter().map(|f| f.file.len()).max().unwrap_or(4).max(4);

  println!("{:<name_w$}  {:<file_w$}  LINE", "FUNCTION", "FILE");
  for f in functions {
    println!("{:<name_w$}  {:<file_w$}  {}", f.name, f.file, f.line);
  }
  println!("\n{} free function(s)", functions.len());
}
