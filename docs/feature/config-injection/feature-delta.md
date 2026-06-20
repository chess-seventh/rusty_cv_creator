# Feature Delta — `config-injection`

> DESIGN wave, PROPOSE mode, APPLICATION/components scope. Density: **LEAN**.
> Forward (not-yet-implemented) refactor. **DOCS-ONLY** — no source changed.
> Behavior-preserving: the 85 existing tests (`cargo nextest run`) are the safety net.
> Outcome Collision Check: **N/A** (internal refactor, no new typed user-facing
> contract; `nwave-ai outcomes check-delta` also non-functional in this install —
> missing `jsonschema`, no `outcomes/registry.yaml`).

## Wave: DESIGN / [REF] Goal

Replace the process-global `GLOBAL_VAR: OnceCell<GlobalVars>`
(`src/global_conf.rs`) with a dependency-injected, immutable `AppContext` value
threaded by borrow (`&AppContext`) through the use cases, so that plain threaded
`cargo test` is deterministic (today only `cargo-nextest`'s process-per-test
makes the shared `OnceCell` safe). See [ADR-0006](../../product/architecture/adr-0006-inject-appcontext.md).

## Wave: DESIGN / [REF] Detailed Design Decisions (DDD)

- **D-1** — Introduce an immutable `AppContext { config: Ini, today: DateTime<Local>, user_input: UserInput }`, constructed once in `main`.
- **D-2** — Thread it by shared borrow `&AppContext` (Option A), not `Arc` (B) and not loose params (C). Single-threaded synchronous CLI ⇒ borrow suffices.
- **D-3** — `AppContext` exposes **read accessors only** (move the existing getters verbatim as `&self` methods). No setters / no interior mutability ⇒ "silent shared-config mutation" is non-representable.
- **D-4** — `get_variable_from_config_file` / `get_db_configurations` become `AppContext` methods (`ctx.config_var(section,key)`, `ctx.db_path()`).
- **D-5** — Keep the DB factory decoupled: `establish_connection` takes the resolved `(engine, url)` (or `db_path`), not `&AppContext`, so `database.rs` does not depend on the config type.
- **D-6** — Compose with, do not touch, ADR-0002 (`CommandRunner`) and ADR-0003 (`DbConnection`): config flows *alongside* the already-injected runner + connection.
- **D-7** — Migrate **incrementally**: add `AppContext`, convert leaf consumers first (`BuildConfig::from_config`, `establish_connection` inputs), thread upward, then delete `GLOBAL_VAR`/`set_global_vars`. Optionally keep `set_global_vars` as a thin shim during transition (see Open questions).
- **D-8** — The three "without global" `should_panic` tests become local-construct tests (build a minimal `AppContext`, assert same error). Determinism moves from the harness into the design.
- **D-9** — Add an enforcement guard (Principle 11): no `static … OnceCell` / no `get_global_var` outside an allowlist, so the global cannot silently return.

## Wave: DESIGN / [REF] Component decomposition

| File | Change type | What changes |
|------|-------------|--------------|
| `src/app_context.rs` | **CREATE NEW** | `AppContext` struct + read-only accessor methods (migrated from `GlobalVars`). |
| `src/global_conf.rs` | **DELETE** (end of migration) | `GLOBAL_VAR`, `GlobalVars`, `get_global_var*` removed; accessors moved to `AppContext`. |
| `src/config_parse.rs` | **MODIFY** | `set_global_vars` → `AppContext::from_user_input` constructor (or transitional shim); `get_variable_from_config_file`/`get_db_configurations` → `AppContext` methods. |
| `src/main.rs` | **MODIFY** | `main` builds `AppContext`; pass `&ctx` into `match_user_action`, `prepare_cv`, the view path, and `check_if_db_env…`. |
| `src/cli_structure.rs` | **MODIFY** | `match_user_action(user_input)` → `match_user_action(&ctx)`; dispatch passes `&ctx` to use cases. |
| `src/cv_insert.rs` | **MODIFY** | `insert_cv()` → `insert_cv(&ctx)`; `prepare_cv(&ctx, runner, …)`; `run_persistence` opens conn via resolved `(engine,url)` from `ctx`. |
| `src/file_handlers.rs` | **MODIFY** | `create_directory(&ctx,…)`, `BuildConfig::from_config(&ctx)`, `prepare_path_for_new_cv` (today + paths from `ctx`), `remove_created_dir_from_pro(&ctx,…)`. |
| `src/helpers.rs` | **MODIFY** | `check_if_db_env_is_set_or_set_from_config(&ctx)`; the viewer-name lookup that feeds `view_cv_file` reads `ctx` at the `main` call site (`view_cv_file` signature unchanged — already takes `pdf_viewer: &str`). |
| `src/database.rs` | **MODIFY** | `establish_connection` takes resolved `(engine, url)`/`db_path` (D-5); no `get_global_var`. |
| `src/user_action.rs` | **MODIFY** | `remove_cv(&ctx, filters)` (needs the resolved DB inputs for `establish_connection`); `show_cvs` unchanged (already takes `&mut DbConnection`). |
| `Cargo.toml` | **MODIFY** (optional, end) | `once_cell` becomes droppable once `GLOBAL_VAR`/inner `OnceCell`s are gone. |

## Wave: DESIGN / [REF] Driving ports

**Unchanged.** The only driving port remains the **CLI** (clap `UserInput` →
`match_user_action`). `AppContext` is constructed from the parsed `UserInput` and
flows inward; it is a read-only value, not a new entry surface.

## Wave: DESIGN / [REF] Driven ports + adapters

- **Configuration** — was an implicit global port (`GLOBAL_VAR` + free getters);
  becomes an **injected read-only value** (`&AppContext`). No adapter/trait — it
  is plain data owned by `main`.
- **`CommandRunner`** (ADR-0002) — **unchanged**; still injected as
  `&dyn CommandRunner`. Config now travels next to it.
- **`DbConnection`** (ADR-0003) — **unchanged** enum; `establish_connection`
  inputs change from "read the global" to "resolved `(engine, url)`" (D-5).
- **Filesystem** — unchanged; `file_handlers` functions now take `&ctx` for
  paths/today instead of reaching the global.

## Wave: DESIGN / [REF] Technology choices

**No new dependencies.** Uses existing `configparser::Ini`, `chrono::DateTime`,
clap `UserInput`. `once_cell` (MIT/Apache-2.0) becomes removable at the end of
the migration. Paradigm unchanged: Rust struct + trait (OOP-ish).

## Wave: DESIGN / [REF] Decisions table

| DDD | Decision | ADR |
|-----|----------|-----|
| D-1..D-9 | Inject `&AppContext` (Option A); remove `GLOBAL_VAR`; read-only value; decoupled DB factory; incremental migration; enforcement guard. | [ADR-0006](../../product/architecture/adr-0006-inject-appcontext.md) |

ADR-0006 **supersedes** the "GLOBAL_VAR OnceCell" open item in ADR-0005 and in
`brief.md`.

## Wave: DESIGN / [REF] Reuse Analysis

Contract-shape legend (Principle 12): **return-only** = pure read, no mutation;
**bounded-change** = declared, aggregate-bounded effects.

| Component | Reuse decision | Contract shape | Universe (mutation set) | Assertion mechanism (crafter) |
|-----------|----------------|----------------|-------------------------|-------------------------------|
| `AppContext` (config + today + user_input) | **CREATE NEW** — no existing injectable value; `GlobalVars` exists only as a global cell, not a passable value. Justification: replaces the global with a borrowable, read-only value. | return-only | none (immutable; no setters/interior mutability) | construct-and-read unit tests; type has no write surface |
| `AppContext::config_var` (was `get_variable_from_config_file`) | **EXTEND/REPLACE** — move logic onto method | return-only | none | unit test on a locally-built `AppContext` |
| `AppContext` getters (`get_job_title`, `get_today_str`, `get_user_input_db_engine`, …) | **EXTEND** — migrated verbatim from `GlobalVars` | return-only | none | existing assertions, retargeted to local `AppContext` |
| `establish_connection` (`database.rs`) | **EXTEND** — signature takes resolved `(engine,url)` | bounded-change (opens 1 DB connection) | DB connection handle only | existing `MultiConnection`/in-memory SQLite tests (ADR-0003) |
| `BuildConfig::from_config` (`file_handlers.rs`) | **EXTEND** — read via `&ctx` | return-only | none | existing `from_config` tests with local `AppContext` |
| `create_directory` / `prepare_path_for_new_cv` / `remove_created_dir_from_pro` | **EXTEND** — `&ctx` for paths + today | bounded-change (creates dated dir, copies template, copies PDFs, removes workdir) | filesystem under configured `cv_path`/`output_pdf` only | existing `tempfile` fs tests with local `AppContext` |
| `insert_cv` / `prepare_cv` | **EXTEND** — `&ctx` threaded | bounded-change (delegates fs + optional DB write) | as delegated below | existing end-to-end fake-builder test with local `AppContext` |
| `match_user_action` (`cli_structure.rs`) | **EXTEND** — `&ctx` param | bounded-change (dispatch) | delegated | existing list/update arm tests |
| `check_if_db_env_is_set_or_set_from_config` (`helpers.rs`) | **EXTEND** — `&ctx` | bounded-change (sets `DATABASE_URL` env) | process env var `DATABASE_URL` only (pre-existing; flagged Open question) | existing behavior preserved; covered indirectly |
| `view_cv_file` (`helpers.rs`) | **REUSE as-is** — already takes `pdf_viewer: &str`; only the *caller* reads `ctx` | bounded-change (spawn viewer via runner) | subprocess via `CommandRunner` | existing `FakeRunner` tests unchanged |
| `show_cvs` / `read_cv_from_db` / `save_new_cv_to_db` | **REUSE as-is** — already take `&mut DbConnection` | bounded-change (DB read/write) | `cv` table rows | existing SQLite tests unchanged |
| `GlobalVars` / `GLOBAL_VAR` / `get_global_var*` (`global_conf.rs`) | **DELETE** at end of migration | n/a | n/a | absence enforced by guard (D-9) |

## Wave: DESIGN / [REF] Open questions

- **Transitional shim?** Keep `set_global_vars` as a thin shim that also builds an
  `AppContext` during incremental migration, or big-bang the swap? Recommend
  incremental with a temporary shim, removed once all call sites take `&ctx`.
- **`establish_connection` input shape:** resolved `(engine, url)` (recommended,
  D-5, keeps `database.rs` config-free) vs `&AppContext` (fewer call-site args
  but couples DB to config type). Decide at crafter time.
- **`DATABASE_URL` env mutation:** `check_if_db_env_is_set_or_set_from_config`
  writes a process env var as a side channel into `establish_connection`'s
  SQLite branch. Out of scope here, but injecting the resolved URL via `ctx`
  would let this env write be removed later (separate refactor).
- **Drop `once_cell`?** Remove the dependency once no `OnceCell` remains.

## Wave: DESIGN / [REF] C4 — config as an injected value

System Context (L1) and Container (L2) are **unchanged** — see
[`c4-diagrams.md`](../../product/architecture/c4-diagrams.md). The only delta is
that the `Config` container stops being a global cell and becomes a value flowing
from `main` into the use cases. The new component view lives in
`c4-diagrams.md` under "L3 — Config injection (feature `config-injection`)".
