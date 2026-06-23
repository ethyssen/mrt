use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn usage_log_path() -> PathBuf {
  let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ethan".to_string());
  PathBuf::from(home).join(".local/state/mrt/usage.log")
}

/// Append a JSONL record of this invocation. Best-effort; never fails the command.
pub fn record_invocation(exit_code: i32, duration_ms: u128) {
  let path = usage_log_path();
  if let Some(parent) = path.parent() {
    let _ = std::fs::create_dir_all(parent);
  }
  let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
  let args: Vec<String> = std::env::args().skip(1).collect();
  let cwd = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
  let record = serde_json::json!({
    "ts": ts,
    "args": args,
    "cwd": cwd,
    "exit_code": exit_code,
    "duration_ms": duration_ms as u64,
  });
  if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
    let _ = writeln!(f, "{}", record);
  }
}
