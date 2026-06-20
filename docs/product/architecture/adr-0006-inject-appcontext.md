# ADR-0006: Inject an `AppContext` value instead of the `GLOBAL_VAR` `OnceCell`

## Status
Accepted (forward-looking; design recorded for a future behavior-preserving refactor — feature `config-injection`)

Supersedes the **"GLOBAL_VAR OnceCell"** open item deferred in
[ADR-0005](adr-0005.md) ("Refactor away `GLOBAL_VAR` ... — deferred") and the
matching *Open questions / known smells* entry in
[brief.md](brief.md).

## Context
Configuration, the run timestamp (`today`), and the parsed `UserInput` are held
in a process-global `pub static GLOBAL_VAR: OnceCell<GlobalVars>`
(`src/global_conf.rs`), populated once by `set_global_vars` (`src/config_parse.rs`)
and read everywhere through free getters (`get_global_var()`, which *clones* the
whole struct) and `get_variable_from_config_file`.

Because the cell is process-global and set-once:
- Determinism depends on **`cargo-nextest`** (process-per-test): each test gets a
  fresh process and therefore a fresh, unset cell. ADR-0005 adopted nextest
  precisely to paper over this.
- Plain threaded **`cargo test`** is order-dependent and can flake. Three tests
  assert behavior *with the global unset* —
  `test_get_variable_from_config_file_error_if_missing` and
  `test_get_db_configurations_error_if_missing` (`config_parse.rs`),
  `test_insert_cv_panics_without_global` (`cv_insert.rs`) — all expecting
  `panic!("GlobalVar Get didn't work")`. Whichever test sets the cell first
  changes the outcome of these for the rest of the run.

"Tests are only deterministic under a specific runner" is a smell: the
determinism lives in the harness, not in the design. The 85 existing tests
(`cargo nextest run`) are the safety net for changing this.

This is internal infrastructure: no new user-facing typed contract is created, so
the Outcome Collision Check is **N/A** for this change (and the local
`nwave-ai outcomes check-delta` is non-functional in this install — missing
`jsonschema`, no `outcomes/registry.yaml`).

## Decision
Replace `GLOBAL_VAR` with an immutable **`AppContext`** value, constructed once in
`main`, and threaded **by shared borrow (`&AppContext`)** through the use cases.

```text
AppContext { config: Ini, today: DateTime<Local>, user_input: UserInput }
```

- The existing accessors (`get_user_input_vars`, `get_today_str`, `get_job_title`,
  `get_user_input_db_engine`, ...) move verbatim onto `AppContext` as `&self`
  read-only methods. `get_variable_from_config_file` / `get_db_configurations`
  become `AppContext` methods (e.g. `ctx.config_var(section, key)`).
- `AppContext` exposes **read accessors only** — no setters, no interior
  mutability, no global. The bug class "a `&self` method silently mutates shared
  config" becomes non-representable (Principle 12: read-only driving value, no
  write surface on the injected type).
- It composes with — does not replace — the existing DI seams: the
  `CommandRunner` port (ADR-0002) and `DbConnection` `MultiConnection`
  (ADR-0003) remain the effect boundaries; `AppContext` is the *read-only
  configuration* value that flows alongside them.
- The DB factory boundary stays decoupled from the config type:
  `establish_connection` takes the **resolved `(engine, url)`** (or
  `db_path`), not `&AppContext`, so `database.rs` keeps no dependency on the
  config struct (preferred; see Open questions).

## Alternatives considered
- **`Arc<AppContext>` shared (Option B)** — rejected for this codebase: the CLI
  is single-threaded and synchronous; ownership is trivially `main` for the whole
  run, so a borrow suffices. `Arc` adds atomic refcounting and obscures the
  "one owner, read-only borrow" story for no benefit. The one closure that
  captures config (`run_persistence`'s `open_conn`) is an `FnOnce` that can
  borrow `&ctx` directly — it does not require `'static`, so no `Arc` is needed.
  Reconsider only if a future async/threaded path needs `'static` capture.
- **Pass `Ini` + `DateTime` + `UserInput` separately (Option C)** — rejected:
  no umbrella type, but every threaded function grows by up to three parameters
  and the cohesive accessor methods (`get_job_title`, `get_today_str`, ...) have
  nowhere to live, scattering into free functions. More churn, less cohesion than
  Option A.
- **Keep `GLOBAL_VAR`, rely on nextest (status quo, ADR-0005)** — rejected as the
  standing decision: it leaves `cargo test` flaky and keeps determinism in the
  harness instead of the design. This ADR is the deferred follow-up ADR-0005
  pointed to.

## Consequences
- **Positive — test determinism:** the three "without global" tests become
  *local-construct* tests (build a minimal `AppContext`, assert the same
  error/behavior). They no longer depend on a shared unset cell, so plain
  threaded `cargo test` becomes deterministic; nextest stays valid but is no
  longer *required* for correctness.
- **Positive — no shared mutable state:** `OnceCell` set/clone-the-world
  (`get_global_var()` clones the whole struct on every read) is gone; reads
  become borrows.
- **Positive — clearer boundaries:** config is an explicit input to each use
  case, visible in the signature; composes cleanly with ADR-0002/0003 (runner +
  connection already injected — config now matches that pattern).
- **Negative — call-site churn:** every function currently reaching for
  `GLOBAL_VAR`/`get_variable_from_config_file` gains a `&AppContext` parameter
  (~9 modules; see `feature-delta.md` decomposition table). This is mechanical
  and guarded by the existing 85 tests.
- **Negative — `main` wiring grows slightly:** `main` now constructs
  `AppContext` and passes it down (previously a hidden global side effect).
- **Neutral — no new dependencies:** `once_cell` may eventually be droppable from
  `Cargo.toml` once `GLOBAL_VAR` and the inner `OnceCell`s are removed.

## Enforcement (Principle 11)
After the refactor, add an architecture guard (test or `clippy`/grep-based
pre-commit) asserting **no `static ... OnceCell`** and **no `get_global_var`**
symbol exist outside a deleted-module allowlist, so the global cannot silently
return. This keeps the read-only-injection rule from eroding.
