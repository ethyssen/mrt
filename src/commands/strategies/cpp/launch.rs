use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use uuid::Uuid;
use zip::ZipArchive;

use super::load_strategies;

#[derive(Parser)]
pub struct LaunchArgs {
  /// Strategy name (e.g. j001-mozart)
  name: String,
  /// Start date (yyyy-mm-dd)
  #[arg(long)]
  start_date: String,
  /// End date (yyyy-mm-dd)
  #[arg(long)]
  end_date: String,
  /// Variant name (e.g. both_release)
  #[arg(long)]
  variant: String,
  /// GitHub Actions run ID to reuse (skips triggering a new build)
  #[arg(long)]
  run_id: Option<u64>,
}

pub fn run(args: LaunchArgs) -> Result<()> {
  let strategies = load_strategies()?;
  let strat = strategies
    .iter()
    .find(|s| s.name == args.name)
    .with_context(|| format!("unknown strategy: {}", args.name))?;

  // Validate variant exists for this strategy.
  if !strat.devel_variants.iter().any(|v| v.name == args.variant) {
    let available: Vec<&str> = strat.devel_variants.iter().map(|v| v.name.as_str()).collect();
    anyhow::bail!(
      "unknown variant '{}' for {}. available: {}",
      args.variant,
      strat.name,
      available.join(", ")
    );
  }

  // Step 1: Get latest devel commit.
  eprintln!("fetching latest devel commit for {} ...", strat.github_repo_name());
  let commit = strat.latest_devel_commit();
  if commit.len() < 32 {
    anyhow::bail!(
      "failed to fetch devel commit for {}: got '{}'",
      strat.github_repo_name(),
      commit
    );
  }
  let short_sha = &commit[..7];
  // Registration name is per-repo so strategies sharing a repo reuse the same SO.
  let registration_name = format!("{}-{short_sha}", strat.repo);
  eprintln!("  commit: {short_sha}");
  eprintln!("  registration name: {registration_name}");

  // Step 2: Check if this repo+commit is already registered in KITE.
  let (user, token) = crate::commands::strategies::kite_credentials()?;
  let client = kite::KiteClient::new(&user, &token)?;
  let existing = client.get_strategy(&registration_name).send()?;

  let strategy_id = if let Some(s) = existing {
    eprintln!("  already registered in KITE (uuid: {}), skipping build", s.id);
    s.id
  } else {
    // Step 3: Build, download, and register.
    let gh_token = github_token()?;
    let gh = crate::github::GitHub::new(&gh_token)?;
    let gh_repo = strat.github_repo_name();

    let run_id = match args.run_id {
      Some(id) => {
        eprintln!("  using existing run ID: {id}");
        id
      },
      None => {
        eprintln!("  triggering build for {gh_repo} (ref=devel)...");
        let id = gh.dispatch_build(&gh_repo, "devel")?;
        gh.wait_for_build(&gh_repo, id)?;
        id
      },
    };
    let zip_bytes = gh.download_artifact(&gh_repo, run_id)?;

    // Extract the .so from the ZIP.
    let tmp = tempfile::tempdir()?;
    let cursor = Cursor::new(&zip_bytes);
    let mut archive = ZipArchive::new(cursor)?;
    let so_filename = archive.file_names().next().context("empty ZIP archive")?.to_string();
    archive.extract(tmp.path())?;
    let so_path = tmp.path().join(&so_filename);

    // Get UUID from the .so filename.
    let uuid = uuid_from_so_filename(&so_filename)?;
    eprintln!("  registering as '{}' (uuid: {})...", registration_name, uuid);

    // Register with KITE.
    client
      .register_strategy(uuid)
      .file_contents(&so_path)
      .display_name(&registration_name)
      .ksim_version("3.14.3-6".into())
      .timeout(std::time::Duration::from_secs(300))
      .send()?;
    eprintln!("  registered successfully");

    uuid
  };

  // Step 4: Load variant TOML.
  let variant_path = variant_toml_path(&strat.repo, &args.variant);
  let variant: kite::options::Variant = if variant_path.exists() {
    let contents = fs::read_to_string(&variant_path)
      .with_context(|| format!("failed to read {}", variant_path.display()))?;
    toml::from_str(&contents)
      .with_context(|| format!("failed to parse variant TOML: {}", variant_path.display()))?
  } else {
    eprintln!("  warning: variant file not found at {}, using defaults", variant_path.display());
    kite::options::Variant::default()
  };

  // Step 5: Launch backtest.
  // Full submission name: logicalname-hash/variant/adjective-noun
  // kite description = "{strategy_name}/{backtest_name}"
  let logical_name = format!("{}-{short_sha}", strat.name);
  let backtest_name = format!("{}/{}", args.variant, crate::name_generator::generate_name());
  eprintln!("launching backtest '{logical_name}/{backtest_name}'...");
  let start_date = date::Date::parse(&args.start_date, "%Y-%m-%d")
    .with_context(|| format!("invalid start date: {}", args.start_date))?;
  let end_date = date::Date::parse(&args.end_date, "%Y-%m-%d")
    .with_context(|| format!("invalid end date: {}", args.end_date))?;

  let mut builder = client
    .launch_backtest(strategy_id)
    .strategy_name(&logical_name)
    .backtest_name(&backtest_name)
    .start_date(start_date)
    .end_date(end_date)
    .variant(variant);

  if !strat.start_time.is_empty() {
    let st: bell_delta::BellDelta<bell_delta::Bell> = strat
      .start_time
      .parse()
      .with_context(|| format!("failed to parse start_time: {}", strat.start_time))?;
    builder = builder.start_time(st);
  }
  if !strat.end_time.is_empty() {
    let et: bell_delta::BellDelta<bell_delta::Bell> = strat
      .end_time
      .parse()
      .with_context(|| format!("failed to parse end_time: {}", strat.end_time))?;
    builder = builder.end_time(et);
  }

  let resp = builder.send()?;
  eprintln!("backtest '{}' launched", resp.name);

  Ok(())
}

/// Read the GitHub token from ~/.capn_strat.toml.
fn github_token() -> Result<String> {
  let home = std::env::var("HOME").context("HOME not set")?;
  let path = format!("{home}/.capn_strat.toml");
  let contents = fs::read_to_string(&path).with_context(|| format!("failed to read {path}"))?;
  let config: toml::Value =
    toml::from_str(&contents).with_context(|| format!("failed to parse {path}"))?;
  config
    .get("github_token")
    .and_then(|v| v.as_str())
    .map(|s| s.to_string())
    .context("github_token not found in ~/.capn_strat.toml")
}

/// Extract the UUID from a compiled strategy .so filename.
///
/// Filenames look like: `libNative_01234567-0123-0123-0123-0123456789ab.so`
/// or: `libNative_01234567-0123-0123-0123-0123456789ab.0.1.so`
fn uuid_from_so_filename(filename: impl AsRef<Path>) -> Result<Uuid> {
  let filename = filename.as_ref().file_name().unwrap().to_str().unwrap();
  let trimmed =
    filename.trim_start_matches("libNative_").trim_end_matches(".0.1").trim_end_matches(".so");
  Uuid::try_from(trimmed)
    .with_context(|| format!("failed to parse UUID from filename: {filename}"))
}

/// Path to variant TOML files for a strategy repo.
fn variant_toml_path(repo: &str, variant: &str) -> PathBuf {
  let home = std::env::var("HOME").unwrap_or_default();
  PathBuf::from(format!("{home}/projects/algos/{repo}/config/variants/{variant}.toml"))
}
