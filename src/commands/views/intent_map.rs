use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use proc_macro2::Span;
use serde::Serialize;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::visit::{self};
use walkdir::WalkDir;

/// Build a comprehensive, multi-scale "intent map" of a crate.
///
/// Every meaningful unit of code becomes one entry: a located *selection of
/// lines* plus a *natural-language intent*.
///
/// The *selection* (file + line range + the source text) is fully deterministic.
#[derive(Parser)]
pub struct IntentMapCmd {
  /// Crate root to scan.
  #[arg(long)]
  path: PathBuf,

  /// Write the JSON to this file instead of stdout.
  #[arg(long, short)]
  output: Option<PathBuf>,

  /// Only emit entries at these scales (repeatable). Default: all.
  #[arg(long, value_enum)]
  scale: Vec<Scale>,

  /// Omit the `code` field, leaving only the line range. Useful when you only
  /// want the skeleton and will slice the source yourself.
  #[arg(long = "no-code", action = clap::ArgAction::SetTrue)]
  no_code: bool,

  /// Emit compact (single-line) JSON instead of pretty-printed.
  #[arg(long, action = clap::ArgAction::SetTrue)]
  compact: bool,
}

/// The altitude an entry sits at. Order here is the reading order: broad to fine.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum Scale {
  Crate,
  Module,
  Type,
  Impl,
  Function,
  Block,
}

/// Where the seeded `intent` text came from — so a later enrichment pass knows
/// which entries are still blank.
#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
enum IntentSource {
  /// A `///` / `//!` / `#[doc]` doc comment attached to the item.
  Doc,
  /// A plain `//` line comment immediately above the selection.
  Comment,
  /// Nothing readable — `intent` is null, awaiting synthesis.
  Missing,
}

/// One element of the map: a located selection of code and its intent.
#[derive(Serialize)]
struct Entry {
  scale: Scale,
  /// The syntactic kind: fn / struct / enum / trait / impl / mod / for / match …
  kind: &'static str,
  /// A path-ish identifier, e.g. `crate::foo::Bar::method` or, for a block,
  /// `crate::foo::Bar::method#for@L42`.
  path: String,
  file: String,
  line_start: usize,
  line_end: usize,
  /// The selected source text. Omitted when `--no-code` is passed.
  #[serde(skip_serializing_if = "Option::is_none")]
  code: Option<String>,
  /// Natural-language intent. Seeded from doc/line comments; null when missing.
  intent: Option<String>,
  intent_source: IntentSource,
}

impl IntentMapCmd {
  pub fn execute(self) -> Result<()> {
    if !self.path.join("src").is_dir() {
      println!("No src dir found! Please provide a valid crate root");
      return Ok(());
    }
    let scan_root = self.path.join("src");

    let mut entries = vec![];
    for dir_entry in WalkDir::new(&scan_root).into_iter().filter_map(|e| e.ok()) {
      let file_path = dir_entry.path();
      if file_path.extension().is_none_or(|ext| ext != "rs") {
        continue;
      }
      collect_from_file(file_path, &scan_root, &mut entries)
        .with_context(|| format!("parsing {}", file_path.display()))?;
    }

    // Containers before their contents: same file, earliest start first, and on
    // a tie the *wider* span (later end) first so a fn precedes its inner blocks.
    entries.sort_by(|a, b| {
      (&a.file, a.line_start, std::cmp::Reverse(a.line_end)).cmp(&(
        &b.file,
        b.line_start,
        std::cmp::Reverse(b.line_end),
      ))
    });

    if !self.scale.is_empty() {
      entries.retain(|e| self.scale.contains(&e.scale));
    }
    if self.no_code {
      entries.iter_mut().for_each(|e| e.code = None);
    }

    let json = if self.compact {
      serde_json::to_string(&entries)?
    } else {
      serde_json::to_string_pretty(&entries)?
    };

    match &self.output {
      Some(path) => {
        std::fs::write(path, &json).with_context(|| format!("writing {}", path.display()))?;
        eprintln!("{} entries written to {}", entries.len(), path.display());
      },
      None => println!("{json}"),
    }
    Ok(())
  }
}

/// Parse one file and append every entry it yields.
fn collect_from_file(path: &Path, root: &Path, out: &mut Vec<Entry>) -> Result<()> {
  let source = std::fs::read_to_string(path)?;
  let ast = syn::parse_file(&source)?;
  let display = path.strip_prefix(root).unwrap_or(path).display().to_string();

  // The file *is* a module. Derive its module path and decide whether it is the
  // crate root (lib.rs / main.rs at the top level).
  let (module_path, is_crate_root) = module_path_for(path, root);
  let lines: Vec<&str> = source.lines().collect();

  // The whole-file entry, scaled `crate` for the root and `module` otherwise.
  let last = lines.len().max(1);
  let file_intent = doc_from_attrs(&ast.attrs);
  out.push(Entry {
    scale: if is_crate_root { Scale::Crate } else { Scale::Module },
    kind: "mod",
    path: module_path.clone(),
    file: display.clone(),
    line_start: 1,
    line_end: last,
    code: None, // a whole file is rarely a useful "selection"; keep it light
    intent_source: source_of(&file_intent, IntentSource::Doc),
    intent: file_intent,
  });

  let mut collector = Collector { lines: &lines, file: &display, path: vec![module_path], out };
  collector.visit_file(&ast);
  Ok(())
}

/// Walks the AST, emitting one [`Entry`] per unit at every scale.
struct Collector<'a> {
  lines: &'a [&'a str],
  file: &'a str,
  /// The enclosing path components (module / type / impl / fn), joined with `::`.
  path: Vec<String>,
  out: &'a mut Vec<Entry>,
}

impl<'a> Collector<'a> {
  /// Push a non-block entry whose intent comes from doc attributes.
  fn push_item(
    &mut self, scale: Scale, kind: &'static str, name: &str, attrs: &[syn::Attribute], span: Span,
  ) {
    let intent = doc_from_attrs(attrs);
    let path = join(&self.path, name);
    self.push(scale, kind, path, span, source_of(&intent, IntentSource::Doc), intent);
  }

  /// Push a block-scale entry, seeding intent from a leading `//` line comment.
  fn push_block(&mut self, kind: &'static str, span: Span) {
    let start = span.start().line;
    let intent = leading_line_comment(self.lines, start);
    let here = self.path.last().map(String::as_str).unwrap_or("");
    let path = format!("{here}#{kind}@L{start}");
    self.push(Scale::Block, kind, path, span, source_of(&intent, IntentSource::Comment), intent);
  }

  fn push(
    &mut self, scale: Scale, kind: &'static str, path: String, span: Span,
    intent_source: IntentSource, intent: Option<String>,
  ) {
    let line_start = span.start().line;
    let line_end = span.end().line.max(line_start);
    self.out.push(Entry {
      scale,
      kind,
      path,
      file: self.file.to_string(),
      line_start,
      line_end,
      code: Some(slice(self.lines, line_start, line_end)),
      intent,
      intent_source,
    });
  }

  /// Visit a function body and the path component for it, then restore.
  fn descend_fn(&mut self, name: &str, block: &'a syn::Block) {
    self.path.push(name.to_string());
    visit::visit_block(self, block);
    self.path.pop();
  }
}

impl<'a> Visit<'a> for Collector<'a> {
  fn visit_item_mod(&mut self, node: &'a syn::ItemMod) {
    // `mod foo;` (no body) points at a separate file we reach via the walk.
    let Some((_, items)) = &node.content else { return };
    let name = node.ident.to_string();
    self.push_item(Scale::Module, "mod", &name, &node.attrs, node.span());
    self.path.push(name);
    for item in items {
      self.visit_item(item);
    }
    self.path.pop();
  }

  fn visit_item_struct(&mut self, node: &'a syn::ItemStruct) {
    self.push_item(Scale::Type, "struct", &node.ident.to_string(), &node.attrs, node.span());
  }

  fn visit_item_enum(&mut self, node: &'a syn::ItemEnum) {
    self.push_item(Scale::Type, "enum", &node.ident.to_string(), &node.attrs, node.span());
  }

  fn visit_item_union(&mut self, node: &'a syn::ItemUnion) {
    self.push_item(Scale::Type, "union", &node.ident.to_string(), &node.attrs, node.span());
  }

  fn visit_item_type(&mut self, node: &'a syn::ItemType) {
    self.push_item(Scale::Type, "type", &node.ident.to_string(), &node.attrs, node.span());
  }

  fn visit_item_trait(&mut self, node: &'a syn::ItemTrait) {
    let name = node.ident.to_string();
    self.push_item(Scale::Type, "trait", &name, &node.attrs, node.span());
    self.path.push(name);
    for item in &node.items {
      if let syn::TraitItem::Fn(f) = item {
        self.push_item(Scale::Function, "fn", &f.sig.ident.to_string(), &f.attrs, f.span());
        if let Some(block) = &f.default {
          self.descend_fn(&f.sig.ident.to_string(), block);
        }
      }
    }
    self.path.pop();
  }

  fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
    let name = impl_name(node);
    self.push_item(Scale::Impl, "impl", &name, &node.attrs, node.span());
    self.path.push(name);
    for item in &node.items {
      if let syn::ImplItem::Fn(f) = item {
        self.push_item(Scale::Function, "fn", &f.sig.ident.to_string(), &f.attrs, f.span());
        self.descend_fn(&f.sig.ident.to_string(), &f.block);
      }
    }
    self.path.pop();
  }

  fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
    self.push_item(Scale::Function, "fn", &node.sig.ident.to_string(), &node.attrs, node.span());
    self.descend_fn(&node.sig.ident.to_string(), &node.block);
  }

  // --- block scale: control flow inside function bodies --------------------
  // Each captures the construct, then recurses so nested control flow is caught.

  fn visit_expr_for_loop(&mut self, node: &'a syn::ExprForLoop) {
    self.push_block("for", node.span());
    visit::visit_expr_for_loop(self, node);
  }

  fn visit_expr_while(&mut self, node: &'a syn::ExprWhile) {
    self.push_block("while", node.span());
    visit::visit_expr_while(self, node);
  }

  fn visit_expr_loop(&mut self, node: &'a syn::ExprLoop) {
    self.push_block("loop", node.span());
    visit::visit_expr_loop(self, node);
  }

  fn visit_expr_match(&mut self, node: &'a syn::ExprMatch) {
    self.push_block("match", node.span());
    visit::visit_expr_match(self, node);
  }

  fn visit_expr_if(&mut self, node: &'a syn::ExprIf) {
    self.push_block("if", node.span());
    visit::visit_expr_if(self, node);
  }
}

/// Best-effort name for an `impl` block: `Trait for Type`, or just `Type`.
fn impl_name(node: &syn::ItemImpl) -> String {
  let ty = type_name(&node.self_ty);
  match &node.trait_ {
    Some((_, path, _)) => format!("{} for {ty}", path_tail(path)),
    None => ty,
  }
}

/// The last identifier of a type path (`a::b::Foo<T>` -> `Foo`), else `_`.
fn type_name(ty: &syn::Type) -> String {
  match ty {
    syn::Type::Path(p) => path_tail(&p.path),
    syn::Type::Reference(r) => type_name(&r.elem),
    _ => "_".to_string(),
  }
}

fn path_tail(path: &syn::Path) -> String {
  path.segments.last().map(|s| s.ident.to_string()).unwrap_or_else(|| "_".to_string())
}

/// Join the enclosing path with a trailing component.
fn join(path: &[String], name: &str) -> String {
  if path.is_empty() { name.to_string() } else { format!("{}::{name}", path.join("::")) }
}

/// Derive a `crate::a::b` module path and whether the file is the crate root.
fn module_path_for(path: &Path, root: &Path) -> (String, bool) {
  let rel = path.strip_prefix(root).unwrap_or(path);
  let mut comps: Vec<String> =
    rel.components().filter_map(|c| c.as_os_str().to_str()).map(|s| s.to_string()).collect();
  if let Some(last) = comps.last_mut() {
    *last = last.trim_end_matches(".rs").to_string();
  }
  // `mod.rs` / `lib.rs` / `main.rs` name the directory, not a child module.
  let root_file = matches!(comps.last().map(String::as_str), Some("lib") | Some("main"));
  let is_crate_root = root_file && comps.len() == 1;
  if matches!(comps.last().map(String::as_str), Some("mod") | Some("lib") | Some("main")) {
    comps.pop();
  }
  let path = std::iter::once("crate".to_string()).chain(comps).collect::<Vec<_>>().join("::");
  (path, is_crate_root)
}

/// Concatenate the text of `///` / `//!` / `#[doc = "…"]` attributes.
fn doc_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
  let mut lines = vec![];
  for attr in attrs {
    if !attr.path().is_ident("doc") {
      continue;
    }
    if let syn::Meta::NameValue(nv) = &attr.meta
      && let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value
    {
      lines.push(s.value().trim().to_string());
    }
  }
  let text = lines.join("\n").trim().to_string();
  (!text.is_empty()).then_some(text)
}

/// Grab a contiguous run of plain `//` comment lines sitting immediately above
/// `line` (1-based). Doc comments (`///`, `//!`) are left to [`doc_from_attrs`].
fn leading_line_comment(lines: &[&str], line: usize) -> Option<String> {
  let mut collected = vec![];
  let mut idx = line.checked_sub(1)?; // index of the construct's own line
  while idx > 0 {
    let candidate = lines[idx - 1].trim();
    let is_plain = candidate.starts_with("//")
      && !candidate.starts_with("///")
      && !candidate.starts_with("//!");
    if !is_plain {
      break;
    }
    collected.push(candidate.trim_start_matches("//").trim().to_string());
    idx -= 1;
  }
  if collected.is_empty() {
    return None;
  }
  collected.reverse();
  Some(collected.join("\n"))
}

/// Pick the source label for a seeded intent, or `Missing` when blank.
fn source_of(intent: &Option<String>, present: IntentSource) -> IntentSource {
  if intent.is_some() { present } else { IntentSource::Missing }
}

/// Slice inclusive 1-based line range `[start, end]` from the file.
fn slice(lines: &[&str], start: usize, end: usize) -> String {
  let lo = start.saturating_sub(1);
  let hi = end.min(lines.len());
  if lo >= hi {
    return String::new();
  }
  lines[lo..hi].join("\n")
}
