# Feature Delta — cv-variant-build

> Feature ID: `cv-variant-build` · Wave: DESIGN (PROPOSE mode, APPLICATION/components scope)
> RETROACTIVE backfill against implemented code (branch `feature/change-generation`, v4.0.2).
> Density: **LEAN** (Tier-1 [REF] sections only).

## Prior Wave Consultation

| Artifact | Wave | Status | Note |
|----------|------|--------|------|
| `docs/product/architecture/brief.md` (System Architecture) | DESIGN | ⊘ absent → bootstrapped here | First architect; greenfield SSOT. |
| `docs/product/architecture/brief.md` (Domain Model) | DESIGN | ⊘ absent | Domain captured inline in brief. |
| DISCUSS requirements / stories / AC | DISCUSS | ⊘ absent | Reverse-engineered from code + 6 merged commits. |
| `docs/product/prd.md` | PRODUCT | ⊘ absent | Greenfield bootstrap. |

**Greenfield bootstrap**: no upstream nWave artifacts existed; this delta and the
brief establish the SSOT retroactively. No upstream changes were required.

---

## Wave: DESIGN / [REF] Design Decisions (DDD)

- **D-1** — Variant-based CV build via Justfile: select 1 of 4 variants and build
  with config-driven `just build <variant>` (tectonic), replacing
  xelatex + `BLANKPOSITION` placeholder. Resolution: `--variant` flag → keyword
  inference (manager first) → `[variant] default`. → ADR-0001.
- **D-2** — `CommandRunner` driven port (`status`/`output`/`spawn`) with
  `SystemRunner`/`FakeRunner`, injected into `compile_cv`/`view_cv_file`/
  `is_tailscale_connected`. → ADR-0002.
- **D-3** — diesel `MultiConnection` `DbConnection` (Postgres prod / SQLite test);
  functions take `&mut DbConnection`; default all-columns selection. → ADR-0003.
- **D-4** — Pre-usage PATH tool checks (`ensure_tools_available`) at orchestration
  layer with devenv hint. → ADR-0004.
- **D-5** — Coverage discipline: test seams raise 54%→84%; `#[coverage(off)]`
  gated on `coverage_nightly`; nextest for determinism. → ADR-0005.

## Wave: DESIGN / [REF] Component Decomposition

| Component | Path | Change type |
|-----------|------|-------------|
| Build subsystem (variant resolve + compile + copy-out) | `src/file_handlers.rs` | EXTEND |
| Orchestrator `prepare_cv` | `src/main.rs` | EXTEND |
| CLI `--variant` flag | `src/cli_structure.rs` | EXTEND |
| Insert use case | `src/cv_insert.rs` | EXTEND |
| `CommandRunner` port + adapters | `src/command_runner.rs` | CREATE NEW |
| `DbConnection` (`MultiConnection`) + CRUD | `src/database.rs` | EXTEND/REPLACE |
| Config accessors (incl. `get_variant`) | `src/global_conf.rs`, `src/config_parse.rs` | EXTEND |
| Tool-availability checks | `src/helpers.rs` | CREATE NEW |
| Domain model / schema | `src/models.rs`, `src/schema.rs` | REUSE |

## Wave: DESIGN / [REF] Driving Ports

- **CLI (clap)** — `UserInput` / `UserAction{Insert,Update,Remove,List}` /
  `FilterArgs` (now carries `--variant`); `match_user_action` dispatches to use
  cases. Sole inbound surface.

## Wave: DESIGN / [REF] Driven Ports + Adapters

| Port | Interface | Adapters | Injected into |
|------|-----------|----------|---------------|
| Subprocess | `CommandRunner` (`status`/`output`/`spawn`) | `SystemRunner` (prod), `FakeRunner` (test) | `compile_cv`, `view_cv_file`, `is_tailscale_connected` |
| Persistence | `DbConnection` (diesel `MultiConnection`) | `PgConnection`, `SqliteConnection` | `save_new_cv_to_db`, `read_cv_from_db`, `check_if_entry_exists`, `remove_cv` |
| Configuration | INI via `GLOBAL_VAR` `OnceCell` accessors | `configparser` | all use cases / build subsystem |
| Filesystem | `copy_dir` + `std::fs` behind `file_handlers` fns | OS filesystem | `create_directory`, `remove_created_dir_from_pro`, `copy_to_destination` |

## Wave: DESIGN / [REF] Technology Choices

Pinned versions in [brief.md](../../product/architecture/brief.md#technology-stack-pinned-from-cargotoml).
Key: clap 4.6.1, diesel 2.3.9 (sqlite+postgres), configparser 3.1.0,
copy_dir 0.1.3, skim 4.6.2, once_cell 1.21.4; Rust edition 2021 (nightly
channel). External tools via devenv: just, tectonic, zathura, tailscale.
All OSS (MIT/Apache-2.0). Paradigm: struct + trait (OOP-leaning), pure
resolution core.

## Wave: DESIGN / [REF] Decisions Table

| DDD | ADR |
|-----|-----|
| D-1 | [ADR-0001](../../product/architecture/adr-0001.md) |
| D-2 | [ADR-0002](../../product/architecture/adr-0002.md) |
| D-3 | [ADR-0003](../../product/architecture/adr-0003.md) |
| D-4 | [ADR-0004](../../product/architecture/adr-0004.md) |
| D-5 | [ADR-0005](../../product/architecture/adr-0005.md) |

## Wave: DESIGN / [REF] Reuse Analysis

| Overlapping component | Decision | Rationale |
|-----------------------|----------|-----------|
| `compile_cv` | EXTEND | Reworked to run `<builder> <recipe> <variant>` via injected runner instead of xelatex; same role. |
| `resolve_variant` / `infer_variant_from_job_title` | CREATE NEW | No prior variant concept existed (placeholder-only before). |
| `_ConnectionType` → `DbConnection` | EXTEND/REPLACE | Single-backend connection replaced by `MultiConnection` enum; CRUD signatures changed to `&mut DbConnection`. |
| `view_cv_file` | EXTEND | Now takes `&dyn CommandRunner` (was direct `Command`); accepts `.tex`/`.pdf`. |
| `is_tailscale_connected` | EXTEND | Now takes `&dyn CommandRunner` for testability. |
| `create_directory` / `remove_created_dir_from_pro` | EXTEND | Output naming/cleanup adapted to `<prefix>-<variant>.pdf`; reused dir/copy plumbing. |
| `models.rs` / `schema.rs` (`Cv`/`NewCv`/`cv`) | REUSE | Domain unchanged; default all-columns selection relies on existing field order. |
| `ensure_tools_available` / `tool_on_path` | CREATE NEW | New pre-usage probe layer (ADR-0004). |
| `GLOBAL_VAR` / `GlobalVars` | EXTEND | Added `get_variant`; otherwise reused config accessor surface. |

**Effect / contract-shape notes (principle 12):**
`resolve_variant`, `infer_variant_from_job_title`, `sanitize_for_path` are
**pure** (return-only). `compile_cv` / `view_cv_file` / `is_tailscale_connected`
are **bounded-change** — all subprocess effects flow through the `CommandRunner`
port (capability injection), so "side-effect-free function silently shells out"
is non-representable at these call sites. Persistence effects are bounded to
`&mut DbConnection`. Filesystem effects are confined to the `file_handlers`
copy/cleanup functions.

## Wave: DESIGN / [REF] Open Questions

- **`GLOBAL_VAR` `OnceCell` refactor** — process-global mutable config; flaky
  under threaded `cargo test`, deterministic under nextest. Candidate to thread
  an injected `Config` value through use cases.
- **`parse_date` dead code** (`cli_structure.rs`, `#[allow(dead_code)]`) — built
  for filter parsing, not yet wired.
- **Filters-on-DB TODO** — `read_cv_from_db` / `show_cvs` accept `FilterArgs`
  but only `limit(50)`; real filtering (`// TODO filters on proper DB`) pending.
