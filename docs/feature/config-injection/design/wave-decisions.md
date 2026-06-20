# Wave Decisions — `config-injection` (DESIGN)

> PROPOSE mode, APPLICATION/components scope, LEAN. Forward refactor, DOCS-ONLY.
> Safety net: 85 existing tests (`cargo nextest run`). Behavior-preserving.

## Key Decisions

| # | Decision | Rationale | Ref |
|---|----------|-----------|-----|
| 1 | Replace `GLOBAL_VAR` `OnceCell` with an immutable `AppContext` value. | Move test determinism out of the harness (`nextest` process isolation) into the design. | ADR-0006 |
| 2 | Thread it by borrow `&AppContext` (Option A) — not `Arc` (B), not loose params (C). | Single-threaded synchronous CLI; `main` owns it for the whole run; borrow is sufficient and cheapest. | ADR-0006 |
| 3 | Read-only accessors only; no setters / no interior mutability. | Makes "silent shared-config mutation" non-representable (Principle 12). | ADR-0006 |
| 4 | Keep `establish_connection` config-free (resolved `(engine,url)`). | Preserve the ADR-0003 driven-port boundary; `database.rs` stays decoupled. | ADR-0006 |
| 5 | Incremental migration with optional `set_global_vars` shim, then delete the global. | Small codebase (~9 files), strong test net; reduces blast-radius risk per step. | ADR-0006 |
| 6 | Add enforcement guard against any returning `static OnceCell`/`get_global_var`. | Principle 11 — rules without enforcement erode. | ADR-0006 |

## Architecture Summary

Unchanged style: **modular monolith, ports-and-adapters** (single Rust binary).
This feature converts the **configuration port** from an implicit process-global
(`GLOBAL_VAR` + free getters, cloned on every read) into an explicit, immutable
`AppContext { config: Ini, today: DateTime<Local>, user_input: UserInput }`
constructed once in `main` and passed inward as `&AppContext`. It composes with
the two existing injected seams — `CommandRunner` (ADR-0002) and `DbConnection`
`MultiConnection` (ADR-0003) — config now travels next to the runner and the
connection instead of hiding in a static. The blast radius is ~9 modules gaining
a `&AppContext` parameter; logic is unchanged.

## Reuse Analysis

See the full table (with Principle-12 contract shapes) in
[`../feature-delta.md`](../feature-delta.md#wave-design--ref-reuse-analysis).
Summary: `AppContext` is the only **CREATE NEW** (justified — no existing
borrowable config value; `GlobalVars` exists only as a global). `global_conf.rs`
is **DELETE** at end of migration. All call-site modules are **EXTEND/MODIFY**.
`view_cv_file`, `show_cvs`, and the DB CRUD functions are **REUSE as-is** (already
take their dependencies as parameters).

## Technology Stack

No new dependencies. Reuses `configparser` (Ini), `chrono` (DateTime), clap
(`UserInput`). `once_cell` (MIT/Apache-2.0) becomes removable once the
`OnceCell`s are gone. Paradigm unchanged: Rust struct + trait (OOP-ish); no
`CLAUDE.md` paradigm change.

## Constraints

- Behavior-preserving: no change to CV-build output, persistence, or CLI surface.
- Determinism target: plain threaded `cargo test` must pass deterministically
  after the refactor (the three "without global" `should_panic` tests become
  local-construct tests).
- Single-user, single-threaded synchronous CLI — no concurrency requirement that
  would justify `Arc`/locking.
- DOCS-ONLY in this wave; implementation deferred to DELIVER.

## Upstream Changes

**None.** No DISCUSS artifacts, no PRD change, no domain-model change, no new
external integration. System Context (L1) and Container (L2) C4 diagrams are
unchanged; only an L3 component view is added.
