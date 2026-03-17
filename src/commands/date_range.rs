use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use date::Date;
use datetime::DateTime;

/// Detect the earliest and latest dates in a CSV file
#[derive(Parser)]
pub struct DateRangeCommand {
  /// Path to the CSV file
  csv_path: String,
}

const DATE_FORMATS: &[&str] = &["%Y-%m-%d", "%Y/%m/%d", "%m/%d/%Y", "%m-%d-%Y"];

fn parse_date(s: &str) -> Option<Date> {
  let s = s.trim();
  // Try plain date formats first, then fall back to datetime (taking the date portion).
  DATE_FORMATS
    .iter()
    .find_map(|fmt| Date::parse(s, fmt).ok())
    .or_else(|| s.parse::<DateTime>().ok().map(|dt| dt.date()))
}

impl DateRangeCommand {
  pub fn execute(self) -> Result<()> {
    let mut rdr = csv::Reader::from_path(&self.csv_path)
      .with_context(|| format!("could not open '{}'", self.csv_path))?;

    let headers: Vec<String> =
      rdr.headers().context("could not read CSV headers")?.iter().map(str::to_owned).collect();

    let ncols = headers.len();
    let mut col_min: Vec<Option<Date>> = vec![None; ncols];
    let mut col_max: Vec<Option<Date>> = vec![None; ncols];
    let mut total_date_values: usize = 0;

    for result in rdr.records() {
      let record = result.context("error reading CSV record")?;
      for (i, field) in record.iter().enumerate() {
        if i >= ncols {
          break;
        }
        if let Some(date) = parse_date(field) {
          total_date_values += 1;
          col_min[i] = Some(col_min[i].map_or(date, |prev: Date| prev.min(date)));
          col_max[i] = Some(col_max[i].map_or(date, |prev: Date| prev.max(date)));
        }
      }
    }

    if total_date_values == 0 {
      println!("No date values detected in '{}'.", self.csv_path);
      return Ok(());
    }

    let mut overall_min: Option<Date> = None;
    let mut overall_max: Option<Date> = None;

    println!("Date columns detected:");
    for (i, header) in headers.iter().enumerate() {
      if let (Some(mn), Some(mx)) = (col_min[i], col_max[i]) {
        println!("  {header}: {mn} → {mx}");
        overall_min = Some(overall_min.map_or(mn, |prev| prev.min(mn)));
        overall_max = Some(overall_max.map_or(mx, |prev| prev.max(mx)));
      }
    }

    if let (Some(mn), Some(mx)) = (overall_min, overall_max) {
      println!("\nOverall range: {mn} → {mx}");
    }

    Ok(())
  }
}
