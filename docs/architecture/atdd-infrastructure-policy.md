# ATDD Infrastructure Policy — rusty_cv_creator

Per `nw-distill` § Project Infrastructure Policy. One file per project. Apply-if-exists;
write-if-absent; rewrite with `--policy=fresh`. Git history is the audit trail.

> Language: **Rust** (`Cargo.toml` present; tests run under `cargo nextest run` /
> `cargo test`). No cucumber-rust harness in this project — acceptance behaviour
> is documented as `.feature` SSOT under `tests/acceptance/cv-variant-build/` and
> mapped to in-crate `#[cfg(test)]` and `tests/` integration tests via the
> traceability table in `docs/feature/cv-variant-build/feature-delta.md`.
> `proptest` is NOT a dependency; no PBT machinery is introduced (LEAN backfill).

## Driving
| Port | Mechanism | Note |
|------|-----------|------|
| CLI (`insert` subcommand, clap `UserInput`/`UserAction`/`FilterArgs`) | subprocess from a tmp working dir (target mechanism) | **Current tests invoke at the orchestration layer** (`prepare_cv`, `match_user_action`), NOT as a real subprocess. A true subprocess CLI test (spawn the built binary, assert exit code + stdout) is an Open Question / gap — see feature-delta DISTILL Driving Adapter coverage. |

## Driven internal (real)
| Port | Mechanism | Note |
|------|-----------|------|
| `DbConnection` (diesel `MultiConnection`) | Prod: `PgConnection` (Postgres over the secure network). Tests: in-memory `SqliteConnection` (`:memory:`) with the `cv` table created per test. | Backend-agnostic query code (ADR-0003). `MultiConnection` precludes `as_select`/`as_returning`; default all-columns selection. |
| Filesystem (`create_directory`, `remove_created_dir_from_pro`, copy-out/cleanup) | Real OS filesystem under `tempfile::TempDir`. | Real I/O on an isolated tmp tree; no fake FS. |
| Configuration (INI via `GLOBAL_VAR` `OnceCell`) | Real `configparser` reading a tmp INI written per test; `set_global_vars`. | Process-global `OnceCell` → determinism requires `cargo-nextest` (process-per-test), ADR-0005. |

## Driven external / non-deterministic (fake)
| Port | Fake | Note |
|------|------|------|
| `CommandRunner` (subprocess: `just`/`tectonic` build) | `testing::FakeRunner` (records invocations, canned success/failure/io-error) | Asserts exact command string, e.g. `"just build senior-devops"` (ADR-0002). Prod adapter: `SystemRunner`. |
| `CommandRunner` (PDF viewer / zathura) | `testing::FakeRunner` (or a tiny custom runner) | `view_cv_file` spawn is faked; asserts `"zathura <pdf>"`. |
| `CommandRunner` (`sudo tailscale status` — secure-network reachability) | `testing::FakeRunner::with_stdout` / `failing` / `io_error` | `is_tailscale_connected` parses canned stdout ("Logged out." vs details). |
| External build contract (CV template repo Justfile: `just build <variant>` → `<prefix>-<variant>.pdf`) | NOT faked at acceptance layer | Highest-risk boundary; covered by a recommended CI template-contract smoke test (architecture brief), not by these specs. |
