use std::path::Path;

use anyhow::{Context, Result, bail};
use semver::Version;

/// Read the crate/workspace version from a Cargo.toml at the given repo root.
///
/// Checks `[workspace.package].version` first (workspace root),
/// then falls back to `[package].version` (single crate).
pub fn read_version(repo_root: &Path) -> Result<Version> {
    let cargo_toml_path = repo_root.join("Cargo.toml");
    let contents = std::fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("failed to read {}", cargo_toml_path.display()))?;
    let doc: toml::Table =
        toml::from_str(&contents).with_context(|| "failed to parse Cargo.toml")?;

    // Try workspace.package.version first.
    if let Some(version_str) = doc
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        return Version::parse(version_str)
            .with_context(|| format!("invalid semver in workspace.package.version: {version_str}"));
    }

    // Fall back to package.version.
    if let Some(version_val) = doc.get("package").and_then(|p| p.get("version")) {
        match version_val.as_str() {
            Some(version_str) => {
                return Version::parse(version_str).with_context(|| {
                    format!("invalid semver in package.version: {version_str}")
                });
            }
            None => {
                // version = { workspace = true } — not a string, means this is a
                // workspace member but not the root. Caller should point at the
                // workspace root instead.
                bail!(
                    "package.version inherits from workspace; \
                     point at the workspace root instead"
                );
            }
        }
    }

    bail!(
        "no version found in {}",
        cargo_toml_path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn single_crate() {
        let dir = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(dir.path().join("Cargo.toml")).unwrap();
        write!(
            f,
            r#"
[package]
name = "foo"
version = "1.2.3"
"#
        )
        .unwrap();

        let v = read_version(dir.path()).unwrap();
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn workspace_version() {
        let dir = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(dir.path().join("Cargo.toml")).unwrap();
        write!(
            f,
            r#"
[workspace]
members = ["a", "b"]

[workspace.package]
version = "3.0.14"
"#
        )
        .unwrap();

        let v = read_version(dir.path()).unwrap();
        assert_eq!(v, Version::new(3, 0, 14));
    }

    #[test]
    fn inherited_version_errors() {
        let dir = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(dir.path().join("Cargo.toml")).unwrap();
        write!(
            f,
            r#"
[package]
name = "foo"
version.workspace = true
"#
        )
        .unwrap();

        let err = read_version(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("inherits from workspace"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn missing_cargo_toml_errors() {
        let dir = tempfile::tempdir().unwrap();
        let err = read_version(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("failed to read"),
            "unexpected error: {err}"
        );
    }
}
