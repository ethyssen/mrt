use anyhow::Result;
use clap::Parser;

/// How to handle dates and times in Rust
#[derive(Parser)]
pub(super) struct DatesCommand;

impl DatesCommand {
  pub fn execute(self) -> Result<()> {
    print!(
      "\
DATES & TIMES IN RUST
=====================

Crates:
  date-rs      Dates (crates.io, by Luke)         date-rs = \"1\"
  datetime-rs  DateTimes (crates.io, by Luke)      datetime-rs = \"1\"
  market-cal   Trading calendar (ss151, by Luke)   market-cal = {{ version = \"4\", registry = \
       \"ss151\" }}

DATE BASICS (date-rs)
---------------------
  use date::date;
  use date::Date;

  let d = date! {{ 2024-11-01 }};       // Date literal

DATETIME → DATE
---------------
  let dt: DateTime = ...;
  let d: Date = dt.date();             // datetime-rs provides .date() → date::Date

TRADING CALENDAR (market-cal)
-----------------------------
  use date::date;
  use date::Date;
  use market_cal::MarketCalendar;      // trait augments date::Date

  let start = date! {{ 2016-01-04 }};
  let end   = date! {{ 2016-12-30 }};

  start.trading_days_between(&end)           -> u64   // trading days between two dates
  end.trading_days_between_signed(&start)    -> i64   // signed version
  start.trading_days_from_now(13)            -> Date  // N trading days forward
  date! {{ 2025-12-25 }}.is_trading_day()      -> bool  // false (Christmas)
  start.next_trading_day()                   -> Date
  date! {{ 2025-12-25 }}.prev_trading_day()    -> Date
"
    );
    Ok(())
  }
}
