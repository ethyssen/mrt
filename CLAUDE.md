# mr-t

mr-t is a cli tool for simplifying common workflows around trading strategy development.

Most of the time there are a small set of decisions to be made, surrounded by many steps.
This tool automates those steps so that human time and attention is freed up.

## Project Structure

```
src/
├── main.rs              # CLI entry point (clap derive, subcommand dispatch)
├── name_generator.rs    # Random adjective-noun name generation (used by fix, temp-strat)
├── window.rs            # X11 window management (snap left/right via wmctrl/xdotool)
└── commands/
    ├── mod.rs           # Module exports
    ├── claude.rs        # claude command
    ├── deploy.rs        # deploy command (with subcommand targets)
    ├── fix.rs           # fix command
    ├── ship.rs          # ship command
    ├── temp_strat.rs    # temp-strat command
    └── update.rs        # update command
templates/
├── Cargo.toml.template  # Template for temp strategy crates
└── main.rs.template     # Template LOTS strategy boilerplate
```

Dependencies: `anyhow`, `clap` (derive), `rand`

## Commands

### `mrt claude`

Launch Claude CLI with cwd set to `~/projects`. Snaps the active terminal window to the right half of the screen.

### `mrt deploy <target>`

Deploy updates to remote services. Subcommand targets:

- `pdq-studio` — SSH into krjr84, pull latest main, build, and restart the pdq-studio systemd service.

### `mrt fix <repo>`

Start a fix workflow for a repository under `~/projects`. Creates a git worktree on a new `fix/<random-name>` branch off origin/main (or origin/master), opens VS Code at the worktree, and snaps windows side-by-side.

### `mrt ship <message>`

Commit, push, and open a PR for the current branch. Shows `git status` and `git diff` for review, stages all changes, commits with the given message, pushes to origin, creates a PR against main (falls back to master), and enables auto-merge with squash. Prints the PR URL on success.

### `mrt temp-strat`

Scaffold a temporary strategy crate in `~/projects/temp-strats/<random-name>`. Generates Cargo.toml and main.rs from templates, adds pdq/lots/agg-stats/feature-data dependencies, opens in VS Code, and kicks off a background `cargo build --release`.

### `mrt update`

Rebuild and reinstall mrt from source via `cargo install --path .`.

---

## Future: Full release workflow

`mrt ship` handles commit → PR. The remaining steps for a full release workflow are:

- Auto-detect semver bump (major/minor/patch) and update version numbers
- Wait for PR merge, then create a GitHub release with generated notes
- The full sequence: `mrt fix <repo>` → make changes → `mrt ship <msg>` → (future) `mrt release`

---

The "finished" state of a code change is when its merged in, released, and all systems, prod and dev are updated accordingly and everyone who benefits from knowing, knows.

We want to accelerate getting to this state. And we want to accelerate delivering useful updates to the ecosystem.

Therefore we need to define hooks that can be used for these things and we need to be able to toggle on/off certain portions of release.

Single repo changes for now.
