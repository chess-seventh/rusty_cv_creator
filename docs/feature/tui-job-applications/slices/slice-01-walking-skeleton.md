# Slice 01 — Walking Skeleton: TUI Application Table

**Goal**: `rusty-cv list` launches a ratatui table showing all applications from the DB, exits cleanly with 'q'.

## IN scope
- Add `ratatui` and `crossterm` to `Cargo.toml`
- New `tui/` module wired to `UserAction::List`
- Table columns: #, Date Applied, Company, Job Title, PDF Path (truncated to 40 chars)
- Status bar: "N applications total"
- Empty-state row when DB has no records
- DB error message in footer + clean exit on keypress
- Terminal size guard: warn if < 80×20
- 'q' / Esc exits TUI, restores terminal

## OUT scope
- Row highlight / navigation (slice 02)
- Filter bar (slice 03)
- PDF open action (slice 04)
- Status tracking (not in this feature)
- Colour themes / config

## Learning hypothesis
Disproves "ratatui integration with the existing Clap/Diesel CLI is non-trivial" if the skeleton takes more than 1 day to build end-to-end with basic error handling.

Confirms "the existing `read_cv_from_db` + `establish_connection_postgres` functions are sufficient to feed the TUI without refactoring" if slice ships cleanly.

## Acceptance criteria
- `cargo test` passes
- Running `rusty-cv list` with a populated DB renders a table with correct row count
- Running `rusty-cv list` with empty DB shows the empty-state message
- Running with a bad DATABASE_URL shows the error footer and exits on any keypress
- 'q' and Esc both exit and restore the terminal (no raw mode residue)

## Dependencies
- Add to `Cargo.toml`: `ratatui = "0.29"`, `crossterm = "0.28"`
- `read_cv_from_db` must return `Vec<Cv>` not `Vec<String>` — refactor needed (currently returns pdf paths only)

## Effort estimate
~4 hours | Reference class: "wire new CLI subcommand to existing data layer + new widget library"

## Pre-slice SPIKE needed?
No — ratatui hello-world is well-documented. The refactor of `read_cv_from_db` return type is the only uncertainty; it's a small change.
