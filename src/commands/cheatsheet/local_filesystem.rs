use anyhow::Result;
use clap::Parser;

/// List important local directories and their purposes
#[derive(Parser)]
pub(super) struct LocalFilesystemCommand;

struct Entry {
  path: &'static str,
  description: &'static str,
}

const ENTRIES: &[Entry] = &[
  Entry {
    path: "~/projects",
    description: "All project repositories",
  },
  Entry {
    path: "~/projects/lots",
    description: "Core LOTS trading framework (write once, run on PDQ/KORE)",
  },
  Entry {
    path: "~/projects/lots-traits",
    description: "Trait definitions for LOTS ecosystem (events, state, history)",
  },
  Entry {
    path: "~/projects/kore-rs",
    description: "FFI harness for running Rust strategies on C++ KORE platform",
  },
  Entry {
    path: "~/projects/pdq-bach",
    description: "Fast Rust multi-day backtester using tick-level data",
  },
  Entry {
    path: "~/projects/replay",
    description: "Market data replay system — data provider for PDQ backtester",
  },
  Entry {
    path: "~/projects/pdq-john",
    description: "John's production trading strategies tested with PDQ (25+)",
  },
  Entry {
    path: "~/projects/kore-rs-john",
    description: "John's strategies for KORE forward/live trading",
  },
  Entry {
    path: "~/projects/pdq-ethan",
    description: "Ethan's experimental strategy development workspace",
  },
  Entry {
    path: "~/projects/pdq-seana",
    description: "Seana's strategies with feature collection capabilities",
  },
  Entry {
    path: "~/projects/lewis-strategies",
    description: "Christian Lewis's quantitative trading strategies",
  },
  Entry {
    path: "~/projects/dbn-pdq-strategies",
    description: "Shared strategies on Databento-sourced tick data",
  },
  Entry {
    path: "~/projects/algos",
    description: "C++ trading strategies for live/forward testing via KORE",
  },
  Entry {
    path: "~/projects/pdq-test",
    description: "~85 test strategies validating PDQ and LOTS features",
  },
  Entry {
    path: "~/projects/golden-test",
    description: "Golden test strategies and comparison code",
  },
  Entry {
    path: "~/projects/price",
    description: "Precise USD price representation using integer math",
  },
  Entry {
    path: "~/projects/fixed-point",
    description: "C++-compatible FixedPoint type for KORE FFI",
  },
  Entry {
    path: "~/projects/share-count",
    description: "Type-safe share holdings wrapper (1/100th share precision)",
  },
  Entry {
    path: "~/projects/datetime-rs",
    description: "Lightweight DateTime using Unix timestamps",
  },
  Entry {
    path: "~/projects/market-cal",
    description: "U.S. stock market calendar (trading days, holidays, bell times)",
  },
  Entry {
    path: "~/projects/bell-time",
    description: "Market-aware time with microsecond precision anchored to bells",
  },
  Entry {
    path: "~/projects/bars",
    description: "Trait-based OHLCV bars and technical analysis indicators",
  },
  Entry {
    path: "~/projects/agg-stats",
    description: "Aggregate market statistics across symbols for strategies",
  },
  Entry {
    path: "~/projects/choices",
    description: "Shared trading enums (Exchange, Side, Venue, etc.)",
  },
  Entry {
    path: "~/projects/chronicles",
    description: "Historical data archive — daily fundamentals from 2015+",
  },
  Entry {
    path: "~/projects/query-data",
    description: "Programmatic access to PostgreSQL-stored market data",
  },
  Entry {
    path: "~/projects/databento",
    description: "CLI to download tick-level data from Databento API",
  },
  Entry {
    path: "~/projects/dbn-downloader",
    description: "Lightweight Databento batch downloader",
  },
  Entry {
    path: "~/projects/options-databento",
    description: "Options market data ingestion from Databento",
  },
  Entry {
    path: "~/projects/dibs",
    description: "Historical tick-level options data (Databento → DuckDB)",
  },
  Entry {
    path: "~/projects/spider-rock",
    description: "SpiderRock API client for options market data",
  },
  Entry {
    path: "~/projects/tickview",
    description: "Fast market data access optimized for visualization",
  },
  Entry {
    path: "~/projects/scribble",
    description: "Efficient symbol-partitioned market event storage",
  },
  Entry {
    path: "~/projects/sequencer",
    description: "Merge market data streams in chronological order",
  },
  Entry {
    path: "~/projects/search",
    description: "Extract fundamental stock data from KTG HDF5 archive",
  },
  Entry {
    path: "~/projects/inspect-hdf5s",
    description: "Python utility to convert KTG HDF5 tick data to CSV",
  },
  Entry {
    path: "~/projects/comp-backtests",
    description: "Compare backtest results across platforms (C++/Rust/KORE)",
  },
  Entry {
    path: "~/projects/kite",
    description: "Rust API client for KTG's KITE backtesting system",
  },
  Entry {
    path: "~/projects/panels",
    description: "Next.js web app for visualizing trading/backtest data",
  },
  Entry {
    path: "~/projects/capn-strat",
    description: "CLI to automate KORE strategy dev and deployment tasks",
  },
  Entry {
    path: "~/projects/auto-scoring",
    description: "Python — identify predictive features and generate decision rules",
  },
  Entry {
    path: "~/projects/auto-sizing",
    description: "Python → Rust codegen for position sizing logic",
  },
  Entry {
    path: "~/projects/kelly_optimizer",
    description: "Compute optimal Kelly fractions for position sizing",
  },
  Entry {
    path: "~/projects/fast_ml",
    description: "Python ML framework for trading performance prediction",
  },
  Entry {
    path: "~/projects/ml-tools",
    description: "Python ML training/management for strategy models",
  },
  Entry {
    path: "~/projects/ti_151",
    description: "Analyze KITE backtests and generate optimized trading rules",
  },
  Entry {
    path: "~/projects/conductor-dev-env",
    description: "Command & control hub for strategy lifecycle management",
  },
  Entry {
    path: "~/projects/orchestration",
    description: "Ansible playbooks for production infrastructure",
  },
  Entry {
    path: "~/projects/rosetta",
    description: "Rust ↔ C++ interop library via cxx bridge",
  },
  Entry {
    path: "~/projects/libksherpa",
    description: "C++20 reusable components for KORE (legacy reference)",
  },
  Entry {
    path: "~/projects/mrt",
    description: "This CLI — leverage tool for accelerating results",
  },
  Entry {
    path: "~/projects/advisor",
    description: "Fact librarian for curated domain knowledge retrieval",
  },
  Entry {
    path: "~/projects/tasks",
    description: "Personal task/project management hub (metadata only, no code)",
  },
  Entry {
    path: "~/projects/cli-template",
    description: "Rust CLI template with clap best practices",
  },
  Entry {
    path: "/mnt/local/historical/replay/tck",
    description: "KTG HDF5 tick data archive (trades, NBBOs, imbalances, fundamentals)",
  },
  Entry {
    path: "/mnt/pdq-results/sherpa",
    description: "Shared drive on krjr80 for long-running PDQ backtest results",
  },
];

impl LocalFilesystemCommand {
  pub fn execute(self) -> Result<()> {
    let max_path = ENTRIES.iter().map(|e| e.path.len()).max().unwrap_or(0);

    for entry in ENTRIES {
      println!("{:<width$}  {}", entry.path, entry.description, width = max_path);
    }

    Ok(())
  }
}
