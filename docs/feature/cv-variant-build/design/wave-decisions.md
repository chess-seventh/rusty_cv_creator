# DESIGN Wave Decisions — cv-variant-build

> Retroactive backfill (PROPOSE mode, APPLICATION/components scope) against the
> implemented code on `feature/change-generation` (v4.0.2). LEAN density.

## Key Decisions

| ID | Decision | ADR |
|----|----------|-----|
| D-1 | Variant-based CV build via Justfile (`just build <variant>`, tectonic); resolution flag → infer (manager first) → default. | ADR-0001 |
| D-2 | `CommandRunner` driven port (`SystemRunner`/`FakeRunner`) for all subprocess effects. | ADR-0002 |
| D-3 | diesel `MultiConnection` `DbConnection` (Postgres prod / SQLite test). | ADR-0003 |
| D-4 | Pre-usage PATH tool checks with devenv hint at orchestration layer. | ADR-0004 |
| D-5 | Coverage discipline: test seams 54%→84%, `#[coverage(off)]` on `coverage_nightly`, nextest. | ADR-0005 |

## Architecture Summary

Modular monolith, ports-and-adapters, single Rust binary. Thin imperative shell
(clap CLI + `main`/`prepare_cv` orchestration) around a small pure core (variant
resolution) and effect-isolating driven ports. Driving port: CLI. Driven ports:
`CommandRunner` (subprocess), `DbConnection` (persistence), INI config,
filesystem. External integration of record: the CV template repo's Justfile
recipe contract (`just build <variant>` → `<prefix>-<variant>.pdf`). C4 L1/L2/L3
in `docs/product/architecture/c4-diagrams.md`.

## Reuse Analysis

| Component | Decision |
|-----------|----------|
| `compile_cv` | EXTEND (runner-injected, builder/recipe/variant) |
| `_ConnectionType` → `DbConnection` | EXTEND/REPLACE (`MultiConnection`) |
| `view_cv_file`, `is_tailscale_connected` | EXTEND (runner-injected) |
| `create_directory`, `remove_created_dir_from_pro` | EXTEND (variant PDF naming) |
| `resolve_variant`, `CommandRunner`, `ensure_tools_available` | CREATE NEW |
| `models.rs` / `schema.rs` (`Cv`/`cv`) | REUSE |

Full table: `docs/feature/cv-variant-build/feature-delta.md`.

## Technology Stack

Rust edition 2021 (nightly channel, for `coverage_attribute`). Crates (pinned):
clap 4.6.1, diesel 2.3.9 (`sqlite`,`postgres`,`returning_clauses_for_sqlite_3_35`),
configparser 3.1.0, chrono 0.4.44, copy_dir 0.1.3, dirs 6.0.0, dotenvy 0.15.7,
env_logger 0.11.10, log 0.4.30, once_cell 1.21.4, skim 4.6.2; dev: tempfile
3.27.0, serial_test 3.5.0. External tools (devenv): just, tectonic, zathura,
tailscale, sqlite/postgresql, diesel-cli. All OSS (MIT/Apache-2.0).

## Constraints

- Single-user local CLI; Postgres reachable only over Tailscale.
- Toolchain pinned to **nightly** (coverage attribute).
- Test determinism requires `cargo-nextest` (process isolation) because of the
  process-global `GLOBAL_VAR` `OnceCell`.
- Build correctness depends on the template repo's Justfile recipe/output
  contract (external, versioned outside this repo).
- `MultiConnection` precludes `as_select`/`as_returning`: `Cv` field order must
  stay aligned with the `cv` schema.

## Upstream Changes

None — greenfield bootstrap. No prior nWave SSOT/DISCUSS artifacts existed; this
wave establishes the SSOT retroactively and required no upstream modifications.
