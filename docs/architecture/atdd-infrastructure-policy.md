# ATDD Infrastructure Policy

Per `nw-distill` § Project Infrastructure Policy. One file per project. Apply-if-exists (default); write-if-absent (first DISTILL); rewrite with `--policy=fresh`. Git history is the audit trail.

First written: 2026-06-06 (feature `tui-job-applications`, DISTILL wave)

## Driving

| Port | Mechanism | Note |
|---|---|---|
| CLI (`rusty-cv <subcommand>`) | `std::process::Command` subprocess from `tempfile::TempDir` | Binary path via `env!("CARGO_BIN_EXE_rusty_cv_creator")` |

## Driven internal (real)

| Port | Mechanism | Note |
|---|---|---|
| DB / `load_all_applications()` | Real SQLite file via Diesel, seeded in `TempDir` per test | `DATABASE_URL=sqlite://<tmp_path>/test.db` set per test; `serial_test::serial` for env-var isolation |

## Driven external / non-deterministic (fake)

| Port | Fake | Note |
|---|---|---|
| OS PDF open (`open_pdf`) | Returns `Result<String, String>` — in tests, capture the path string without actually spawning | No real process spawn in unit tests; subprocess smoke test in WS scenario verifies wiring |
| Terminal (crossterm raw mode) | Not exercised in subprocess tests — startup probe detects non-TTY and exits before raw mode | TUI rendering is not directly assertion-tested; assertions are on exit code, stdout, and unit-level state |
