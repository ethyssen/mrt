use std::thread;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::json;

/// A GitHub API client for triggering builds and downloading artifacts.
pub struct GitHub {
  client: Client,
}

impl GitHub {
  pub fn new(token: &str) -> Result<Self> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::ACCEPT, "application/vnd.github+json".parse()?);
    headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);
    headers
      .insert(reqwest::header::USER_AGENT, format!("mrt/{}", env!("CARGO_PKG_VERSION")).parse()?);
    headers.insert(reqwest::header::AUTHORIZATION, format!("Bearer {token}").parse()?);
    let client = ClientBuilder::new().default_headers(headers).build()?;
    Ok(Self { client })
  }

  /// Dispatch a build workflow and return the run ID once it starts.
  pub fn dispatch_build(&self, repo: &str, ref_: &str) -> Result<u64> {
    // Trigger the workflow.
    self
      .client
      .post(format!(
        "https://api.github.com/repos/ss151/{repo}/actions/workflows/build.yaml/dispatches"
      ))
      .json(&json!({
        "ref": ref_,
        "inputs": {"build_type": "Debug"},
      }))
      .send()?
      .error_for_status()
      .context("failed to dispatch build workflow")?;

    // Wait for the run to appear.
    for tick in 0..15 {
      eprint!("\r  build dispatched, waiting for run ID ({} seconds)...  ", 15 - tick);
      thread::sleep(Duration::from_secs(1));
    }
    eprintln!();

    let runs = self
      .client
      .get(format!("https://api.github.com/repos/ss151/{repo}/actions/runs"))
      .query(&[("event", "workflow_dispatch")])
      .send()?
      .error_for_status()?
      .json::<WorkflowRunsResponse>()?;

    let run = runs.workflow_runs.first().context("no workflow runs found")?;
    eprintln!("  run ID: {} (https://github.com/ss151/{repo}/actions/runs/{})", run.id, run.id);
    Ok(run.id)
  }

  /// Poll a workflow run until it completes. Returns the conclusion string.
  pub fn wait_for_build(&self, repo: &str, run_id: u64) -> Result<()> {
    // Initial wait.
    for tick in 0..60 {
      eprint!("\r  waiting for build ({} seconds)...  ", 60 - tick);
      thread::sleep(Duration::from_secs(1));
    }
    eprintln!();

    let mut attempts: u64 = 0;
    loop {
      let result = self
        .client
        .get(format!("https://api.github.com/repos/ss151/{repo}/actions/runs/{run_id}"))
        .send();

      // Retry on transient request errors (connection resets, etc).
      let run: WorkflowRun = match result {
        Ok(resp) => resp.error_for_status()?.json()?,
        Err(e) => {
          eprintln!("  request failed ({}), retrying in 10s...", e);
          thread::sleep(Duration::from_secs(10));
          continue;
        },
      };

      if let Some(conclusion) = &run.conclusion {
        if conclusion == "success" {
          eprintln!("  build completed successfully");
          return Ok(());
        } else {
          anyhow::bail!("build completed with status: {conclusion}");
        }
      }

      attempts += 1;
      let delay = attempts.saturating_sub(10).clamp(1, 6) * 5;
      for tick in 0..delay {
        eprint!("\r  build in progress, checking in {} seconds...  ", delay - tick);
        thread::sleep(Duration::from_secs(1));
      }
      eprintln!();
    }
  }

  /// Download the compiled strategy artifact ZIP.
  pub fn download_artifact(&self, repo: &str, run_id: u64) -> Result<bytes::Bytes> {
    let artifacts = self
      .client
      .get(format!("https://api.github.com/repos/ss151/{repo}/actions/runs/{run_id}/artifacts"))
      .send()?
      .error_for_status()?
      .json::<ArtifactsResponse>()?;

    let artifact = artifacts
      .artifacts
      .into_iter()
      .find(|a| a.name == "compiled-strategy")
      .context("no 'compiled-strategy' artifact found — build may have failed")?;

    eprintln!("  downloading artifact...");
    let zip_bytes =
      self.client.get(&artifact.archive_download_url).send()?.error_for_status()?.bytes()?;
    eprintln!("  downloaded {} bytes", zip_bytes.len());
    Ok(zip_bytes)
  }
}

#[derive(Deserialize)]
struct WorkflowRunsResponse {
  workflow_runs: Vec<WorkflowRun>,
}

#[derive(Deserialize)]
struct WorkflowRun {
  id: u64,
  conclusion: Option<String>,
}

#[derive(Deserialize)]
struct ArtifactsResponse {
  artifacts: Vec<Artifact>,
}

#[derive(Deserialize)]
struct Artifact {
  name: String,
  archive_download_url: String,
}
