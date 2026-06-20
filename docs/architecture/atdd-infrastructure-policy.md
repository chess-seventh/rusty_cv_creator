# ATDD Infrastructure Policy — rusty_cv_creator

Per `nw-distill` § Project Infrastructure Policy. One file per project. Apply-if-exists;
write-if-absent; rewrite with `--policy=fresh`. Git history is the audit trail.

> Language: **Rust** (`Cargo.toml` present; tests run under `cargo nextest run` /
> `cargo test`). No cucumber-rust harness — acceptance behaviour is documented as
> `.feature` SSOT (`tests/acceptance/cv-variant-build/` for cv-variant-build;
> `tests/features/tui_job_applications/` for tui-job-applications) and mapped to
> in-crate `#[cfg(test)]` + `tests/` integration tests via each feature's
> traceability table. `proptest` is a dev-dependency (added by tui-job-applications);
> cv-variant-build itself introduced no PBT.

## Driving
| Port | Mechanism | Note |
|------|-----------|------|
| CLI (`insert` subcommand, clap `UserInput`/`UserAction`/`FilterArgs`) | subprocess from a tmp working dir (target mechanism) | `tests/cli_smoke.rs` now spawns the built binary (`--help`/`insert --help`); deeper flows still exercised at the orchestration layer (`prepare_cv`, `match_user_action`). |
| TUI (terminal UI, feature `tui-job-applications`) | `std::process::Command` subprocess from `tempfile::TempDir`; binary via `env!("CARGO_BIN_EXE_rusty_cv_creator")`; startup probe exits on non-TTY. | Assertions on exit code, stdout, and unit-level state; raw-mode rendering not directly asserted. |

## Driven internal (real)
| Port | Mechanism | Note |
|------|-----------|------|
| `DbConnection` (diesel `MultiConnection`) | Prod: `PgConnection` (Postgres over the secure network). Tests: in-memory `SqliteConnection` (`:memory:`) with the `cv` table created per test. | Backend-agnostic query code (ADR-0003). `MultiConnection` precludes `as_select`/`as_returning`; default all-columns selection. |
| Filesystem (`create_directory`, `remove_created_dir_from_pro`, copy-out/cleanup) | Real OS filesystem under `tempfile::TempDir`. | Real I/O on an isolated tmp tree; no fake FS. |
| Configuration (INI via injected `AppContext`) | Real `configparser` reading a tmp INI written per test; `build_context` builds an `AppContext`. | Immutable injected config (ADR-0006); deterministic under threaded `cargo test` (superseded the `GLOBAL_VAR` `OnceCell`). |
| DB read for TUI (`tui::db::load_all_applications`) | Real SQLite file via Diesel seeded in `TempDir`; `DATABASE_URL=sqlite://<tmp>/test.db` per test; `serial_test::serial` for env isolation. | Follow-up: migrate onto the `AppContext`/`DbConnection` seam for consistency with the rest of v5. |

## Driven external / non-deterministic (fake)
| Port | Fake | Note |
|------|------|------|
| `CommandRunner` (subprocess: `just`/`tectonic` build) | `testing::FakeRunner` (records invocations, canned success/failure/io-error) | Asserts exact command string, e.g. `"just build senior-devops"` (ADR-0002). Prod adapter: `SystemRunner`. |
| `CommandRunner` (PDF viewer / zathura) | `testing::FakeRunner` (or a tiny custom runner) | `view_cv_file` spawn is faked; asserts `"zathura <pdf>"`. |
| `CommandRunner` (`sudo tailscale status` — secure-network reachability) | `testing::FakeRunner::with_stdout` / `failing` / `io_error` | `is_tailscale_connected` parses canned stdout ("Logged out." vs details). |
| External build contract (CV template repo Justfile: `just build <variant>` → `<prefix>-<variant>.pdf`) | NOT faked at acceptance layer | Highest-risk boundary; covered by a recommended CI template-contract smoke test (architecture brief), not by these specs. |
| OS PDF open (TUI `open_pdf`) | Returns `Result<String, String>`; tests capture the path string without spawning a viewer. | The subprocess walking-skeleton scenario verifies wiring end-to-end. |
| Terminal (crossterm raw mode) | Not exercised in subprocess tests; the startup probe detects a non-TTY and exits before entering raw mode. | TUI rendering is not assertion-tested; assertions are on exit code, stdout, and unit-level state. |
