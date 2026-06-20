# Evolution — config-injection

**Shipped**: 2026-06-20 · branch `feature/change-generation` · DES step `01-01` ·
code commit `5214f33` (`refactor(config): inject AppContext, drop global OnceCell`).

## What shipped

Replaced the process-global `pub static GLOBAL_VAR: OnceCell<GlobalVars>` with an
immutable `AppContext { config: Ini, today: DateTime<Local>, user_input: UserInput }`
built once in `main` (`build_context`) and injected by borrow (`&AppContext`)
through the use cases. Removed the global static and the free
`get_global_var` / `get_global_var_config_db_*` accessors; `get_variable_from_config_file`
and `get_db_configurations` now take `&AppContext`. Dropped the `once_cell`
dependency. Behavior-preserving (ADR-0006 Option A).

## Why (decision)

[ADR-0006](../product/architecture/adr-0006-inject-appcontext.md) — chosen over
`Arc<AppContext>` (no concurrency need) and loose-params (signature bloat).
Supersedes the GLOBAL_VAR open item in ADR-0005. Composes with the existing
injected seams: `CommandRunner` (ADR-0002) and `DbConnection` (ADR-0003).

## Outcome measured

- **Determinism fixed**: threaded `cargo test` went from **3 failing → 85/85 green**;
  `cargo nextest run` stays 85/85. Determinism now comes from the design, not from
  the test runner's process-per-test isolation.
- `grep -rn "GLOBAL_VAR\|get_global_var" src/` is clean; `once_cell` removed.
- clippy `-D warnings` + pedantic clean; `treefmt` applied.
- DES integrity: "All 1 steps have complete DES traces" (RED/GREEN/COMMIT for 01-01).
- Blast radius: 11 production files (−199/+194 lines); no new dependencies.

## Process notes

- DELIVER ran as a real wave (not a backfill): roadmap → crafter TDD (RED/GREEN/COMMIT,
  DES-monitored) → integrity gate. DISTILL was intentionally skipped (behavior-preserving
  refactor with full existing coverage; DELIVER derived from DESIGN).
- Phases skipped with rationale: L1-L6 refactor pass (the change *is* a refactor,
  clippy/treefmt clean), adversarial review (well-verified, proportional), mutation
  (`nightly-delta` strategy).

## Follow-ups (unchanged from cv-variant-build, still open)

- `parse_date` dead code; wire real DB filtering or descope `list`/`update`.
- CI template-contract smoke test for the external CV repo (`just build <variant>`).
