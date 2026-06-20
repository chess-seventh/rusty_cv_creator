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

---

## Wave: DISCUSS / [REF] Retroactive Backfill Note

> These DISCUSS sections were **reverse-engineered** from the shipped code
> (branch `feature/change-generation`, v4.0.2) and the DESIGN [REF] sections
> above. The feature is already implemented and merged; requirements, JTBD, user
> stories, and acceptance criteria document the *realized* behavior — they do not
> propose new work and must not contradict the DESIGN [REF] sections or
> ADR-0001..0005. Density: **LEAN** (Tier-1 [REF] only). interaction:
> non-interactive backfill (inferred from code + DESIGN, no live discovery).
> No DISCOVER/DIVERGE artifacts existed (⊘); the job statement was supplied as
> ground truth and recorded in `docs/product/jobs.yaml`.

## Wave: DISCUSS / [REF] Persona

- **Persona ID**: `job-seeker` (SSOT: `docs/product/personas/job-seeker.yaml`).
- **Who**: Francesco — the single user and repo owner; an experienced
  infrastructure engineer applying to senior DevOps / Platform / SRE / Engineering
  Manager roles. Comfortable on the CLI, owns a LaTeX CV template repo, applies to
  many companies and needs to move fast without sending a mis-framed CV.

## Wave: DISCUSS / [REF] JTBD One-liner

> When I apply for a specific role at a company, I want to produce the right CV
> variant for that role quickly and keep a record of the application, so I can
> apply faster and track where I've applied.

SSOT: `docs/product/jobs.yaml` (job id `apply-with-tailored-cv`). Functional /
emotional / social dimensions, four forces, and opportunity score live there.

## Wave: DISCUSS / [REF] Locked Decisions

> DISCUSS-wave requirement decisions (RD-n). Distinct namespace from DESIGN
> D-1..D-5; where a requirement is realized by a DESIGN decision the link is named.

- **RD-1** — Variant selection is **explicit-or-inferred**: a valid `--variant`
  flag wins; otherwise the variant is inferred from job-title keywords; otherwise
  the configured default (`[variant] default`, fallback `senior-devops`) is used.
  (realized by DESIGN D-1 / ADR-0001.)
- **RD-2** — Exactly **4 role variants** are in scope: `senior-devops`,
  `senior-platform-engineer`, `senior-sre`, `engineering-manager`.
- **RD-3** — Manager-family titles (`manager`, `management`, `head of`, `lead`)
  resolve to `engineering-manager` **before** any other keyword (manager titles
  may also contain "DevOps"/"Platform").
- **RD-4** — Persisting an application is **opt-in** via `--save-to-database`;
  the default run produces a PDF without writing to the database.
- **RD-5** — Viewing the produced PDF is **opt-in** via `--view-generated-cv`;
  output naming is deterministic: `<date>-<job>-<company>.pdf` under the
  configured per-year output directory.

## Wave: DISCUSS / [REF] User Stories

Five focused stories, each tracing to job `apply-with-tailored-cv`. Full LeanUX
form + outcome KPIs in `docs/feature/cv-variant-build/discuss/user-stories.md`.

### US-01 — Generate a chosen variant by flag

- **job_id**: `apply-with-tailored-cv`
- **Elevator Pitch** —
  - Before: Francesco hand-picks/edits the right LaTeX driver for a manager role
    and risks building the wrong CV shape.
  - After: run `rusty_cv_creator insert --job-title "Engineering Manager" --company-name "Datadog" --variant engineering-manager`
    → sees log `✅ Using variant from --variant flag: engineering-manager` then
    `CV saved to: <output>/2026/2026-06-20-Engineering-Manager-Datadog.pdf`.
  - Decision enabled: Francesco confirms the manager-framed CV is the one he will
    attach before submitting.
- **ACs**
  - Given `--variant senior-sre` and any job title, When `insert` runs, Then the
    build uses `senior-sre` and the output PDF basename contains `senior-sre`.
  - Given `--variant bogus` (not one of the 4), When `insert` runs, Then the tool
    warns and falls back to inference from the job title (does not abort).

### US-02 — Infer the variant from the job title

- **job_id**: `apply-with-tailored-cv`
- **Elevator Pitch** —
  - Before: Francesco must remember and type the exact variant name for every
    application.
  - After: run `rusty_cv_creator insert --job-title "Site Reliability Engineer" --company-name "Cloudflare"`
    → sees log `✅ Inferred variant 'senior-sre' from job title: Site Reliability Engineer`.
  - Decision enabled: Francesco trusts the inferred framing or overrides it with
    `--variant`.
- **ACs**
  - Given no `--variant` and job title "Senior Platform Engineer", When `insert`
    runs, Then variant `senior-platform-engineer` is selected.
  - Given no `--variant` and job title "Engineering Manager - DevOps", When
    `insert` runs, Then variant `engineering-manager` is selected (manager wins).
  - Given no `--variant` and an unmatched title (e.g. "Accountant"), When `insert`
    runs, Then the configured default variant is used.

### US-03 — Land the PDF in an organized, predictable location

- **job_id**: `apply-with-tailored-cv`
- **Elevator Pitch** —
  - Before: built PDFs are scattered in the template working copy and hard to find
    when attaching to an application.
  - After: run `rusty_cv_creator insert --job-title "Senior DevOps" --company-name "ACME"`
    → sees `CV saved to: <output_pdf>/2026/2026-06-20-Senior-DevOps-ACME.pdf`;
    the working directory is cleaned up.
  - Decision enabled: Francesco knows exactly which file to attach and which year's
    applications it belongs to.
- **ACs**
  - Given a successful build, When `insert` completes, Then a PDF named
    `<date>-<job>-<company>.pdf` exists under `<output_pdf>/<year>/`.
  - Given a successful build, When `insert` completes, Then the dated working
    directory is removed and only the PDF copies remain.

### US-04 — Record the application in the database

- **job_id**: `apply-with-tailored-cv`
- **Elevator Pitch** —
  - Before: Francesco cannot tell whether he already applied to a company/role.
  - After: run `rusty_cv_creator insert --job-title "Platform Engineer" --company-name "Stripe" --save-to-database`
    → sees log `Saved CV to database`; job title, company, quote, PDF path and date
    are stored.
  - Decision enabled: Francesco can later check where he has already applied and
    avoid duplicate applications.
- **ACs**
  - Given `--save-to-database`, When `insert` completes, Then a record with job
    title, company, quote, PDF path and application date is persisted.
  - Given the flag omitted, When `insert` completes, Then no DB write occurs and
    the tool logs `CV NOT SAVED TO DATABASE!`.

### US-05 — Preview the generated PDF before sending

- **job_id**: `apply-with-tailored-cv`
- **Elevator Pitch** —
  - Before: Francesco opens the PDF manually to check it rendered correctly.
  - After: run `rusty_cv_creator insert --job-title "DevOps Engineer" --company-name "GitLab" --view-generated-cv`
    → the produced PDF opens in the configured viewer (zathura).
  - Decision enabled: Francesco visually verifies the CV renders correctly before
    attaching it.
- **ACs**
  - Given `--view-generated-cv` and a successful build, When `insert` completes,
    Then the produced PDF is opened in the configured `[optional] pdf_viewer`.
  - Given the viewer tool is not on PATH, When `insert` runs, Then the tool fails
    fast with a devenv hint (per ADR-0004) rather than a cryptic subprocess error.

## Wave: DISCUSS / [REF] Definition of Done

- All five stories' ACs demonstrable via the `insert` subcommand against a
  configured template repo.
- Variant resolution precedence (flag → inference → default) covered by unit tests
  (`resolve_variant`, `infer_variant_from_job_title`).
- Output PDF appears in the configured per-year directory with the deterministic
  name; working directory cleaned up.
- Persistence and view paths are opt-in and behave per RD-4 / RD-5.
- No contradiction with DESIGN [REF] / ADR-0001..0005. (Already merged; v4.0.2.)

## Wave: DISCUSS / [REF] Out of Scope

- Filtering/listing applications by criteria (`list`/`update` are partially
  implemented; `// TODO filters on proper DB`, `limit(50)` only).
- Editing an existing application record (`update` arm is a stub).
- Adding/removing CV variants beyond the 4 (template-repo concern).
- Multi-user, web UI, cloud storage, AI content (README roadmap — future).
- `parse_date` filter wiring (`#[allow(dead_code)]`).

## Wave: DISCUSS / [REF] Walking-Skeleton Strategy

**N/A — brownfield / already shipped.** The end-to-end `insert` flow (resolve →
build → copy-out → optional persist/view) is implemented and merged (v4.0.2, 6
commits). No skeleton or elephant-carpaccio slicing is proposed; slices are
recorded as N/A in `discuss/wave-decisions.md` rather than fabricated.

## Wave: DISCUSS / [REF] Driving Ports

- **CLI (clap)** — sole inbound surface. Subcommand `insert` carries `FilterArgs`:
  `--job-title`, `--company-name`, `--quote`, `--variant` (and `--date`, unused for
  insert). Global flags on `UserInput`: `--save-to-database`, `--view-generated-cv`,
  `--dry-run`, `--config-ini`, `--engine`. Other subcommands `list` / `update` /
  `remove` exist but are partial/out of scope here.

## Wave: DISCUSS / [REF] Pre-requisites

- External **CV template repository** (`git@github.com:chess-seventh/cv.git`)
  exposing `just build <variant>` → `<prefix>-<variant>.pdf` for the 4 variants.
- Tools on PATH (devenv-provisioned, pre-usage checked per ADR-0004): `just`,
  `tectonic`; `zathura` (only when `--view-generated-cv`); `sudo`+`tailscale`
  (Postgres reachability).
- INI config (`~/.config/rusty-cv-creator/rusty-cv-config.ini`): `[cv]
  cv_template_path`/`cv_file_prefix`, `[destination] cv_path`/`output_pdf`,
  `[variant] default`, optional `[build] builder`/`recipe`, `[optional] pdf_viewer`,
  `[db] engine`.
- **Postgres** (prod, over Tailscale) or **SQLite** (local/tests) for
  `--save-to-database`.

---

## Wave: DISTILL / [REF] Retroactive Backfill Note

> These DISTILL sections were authored against **already-shipped, GREEN** code
> (branch `feature/change-generation`, v4.0.2): 80 tests pass under
> `cargo nextest run`, line coverage ~84%. The `.feature` files under
> `tests/acceptance/cv-variant-build/` are a **documentation SSOT** — the project
> has **no cucumber-rust harness**, so scenarios are mapped to the concrete
> existing Rust tests via the Traceability table below. No source code was
> modified, no dependencies added (notably `proptest` remains absent), and no RED
> scaffolds were created. Mandate-7 RED-scaffolding is **N/A** (implementation
> already exists and is green). Density: **LEAN**. interaction: non-interactive.

## Wave: DISTILL / [REF] Inherited commitments

| Origin | Commitment | DDD | Impact |
|--------|------------|-----|--------|
| DISCUSS#RD-1 | Variant selection is explicit-or-inferred (flag → inference → default). | D-1 | Drives the 7 variant-selection scenarios; pure decision function, no I/O. |
| DISCUSS#RD-2 | Exactly 4 role variants in scope. | n/a | Equivalence classes for inference scenarios (one per variant + none). |
| DISCUSS#RD-3 | Manager-family titles win over other keywords. | n/a | Dedicated precedence edge scenario maps to `test_infer_variant_manager_wins_over_devops`. |
| DISCUSS#RD-4 | Persistence is opt-in via `--save-to-database`. | D-3 | Persistence scenarios run against in-memory SQLite; the no-write path is a documented gap. |
| DISCUSS#RD-5 | View is opt-in; deterministic output name under per-year dir. | D-1 | cv-build filing scenarios assert per-year placement + cleanup. |

## Wave: DISTILL / [REF] Scenario list with tags

32 scenarios across 4 documentation feature files. 19 error/edge (**59%**).

| # | Feature file | Scenario | Tags |
|---|--------------|----------|------|
| 1 | variant-selection | Explicit variant choice honoured | @US-01 @contract-shape:pure-function |
| 2 | variant-selection | Unrecognised choice falls back to inference | @US-01 @error @contract-shape:pure-function |
| 3 | variant-selection | Platform title selects platform variant | @US-02 @contract-shape:pure-function |
| 4 | variant-selection | Site-reliability title selects reliability variant | @US-02 @contract-shape:pure-function |
| 5 | variant-selection | Devops title selects devops variant | @US-02 @contract-shape:pure-function |
| 6 | variant-selection | Manager title wins over other keywords | @US-02 @edge @contract-shape:pure-function |
| 7 | variant-selection | Unrecognised title falls back to default | @US-02 @error @contract-shape:pure-function |
| 8 | cv-build | Francesco generates a tailored CV end to end | @walking_skeleton @driving_port @US-03 @in-memory @contract-shape:bounded-change |
| 9 | cv-build | Built CV filed under per-year dir + working copy cleaned up | @US-03 @in-memory @contract-shape:bounded-change |
| 10 | cv-build | Spaces in job/company become dashes in filed name | @US-03 @edge @contract-shape:pure-function |
| 11 | cv-build | Building a chosen variant invokes the builder recipe | @US-01 @in-memory @contract-shape:bounded-change |
| 12 | cv-build | Build refused when driver file missing | @US-01 @error @in-memory @contract-shape:unbounded-preservation |
| 13 | cv-build | Build refused when working directory missing | @US-03 @error @in-memory @contract-shape:unbounded-preservation |
| 14 | cv-build | Build reports failure when builder fails | @US-01 @error @in-memory @contract-shape:bounded-change |
| 15 | cv-build | Filing refused when expected PDF missing | @US-03 @error @contract-shape:unbounded-preservation |
| 16 | cv-build | Dated working directory removed after filing | @US-03 @contract-shape:unbounded-preservation |
| 17 | persistence | Application recorded when saving opted in | @US-04 @in-memory @contract-shape:bounded-change |
| 18 | persistence | Re-saving same application keeps single record | @US-04 @edge @in-memory @contract-shape:bounded-change |
| 19 | persistence | Motivational quote stored with application | @US-04 @in-memory @contract-shape:bounded-change |
| 20 | persistence | Recorded applications listed back | @US-04 @in-memory @contract-shape:pure-function |
| 21 | persistence | Listing with no applications returns nothing | @US-04 @edge @in-memory @contract-shape:pure-function |
| 22 | persistence | Connectivity confirmed when secure network up | @US-04 @error @in-memory @contract-shape:pure-function |
| 23 | persistence | Connectivity down when logged out | @US-04 @error @in-memory @contract-shape:pure-function |
| 24 | persistence | Reachability errors when status command fails | @US-04 @error @in-memory @contract-shape:pure-function |
| 25 | persistence | Reachability errors when status cannot run | @US-04 @error @in-memory @contract-shape:pure-function |
| 26 | preview | Generated PDF opens in configured viewer | @US-05 @in-memory @contract-shape:bounded-change |
| 27 | preview | Source path previewed as its PDF counterpart | @US-05 @edge @in-memory @contract-shape:bounded-change |
| 28 | preview | Preview reports error when viewer cannot launch | @US-05 @error @in-memory @contract-shape:bounded-change |
| 29 | preview | Missing required tools fail fast with devenv hint | @US-05 @error @contract-shape:pure-function |
| 30 | preview | Tool check passes when nothing required | @US-05 @edge @contract-shape:pure-function |
| 31 | preview | Tool absent from path reported unavailable | @US-05 @error @contract-shape:pure-function |
| 32 | preview | Tool present on path reported available | @US-05 @contract-shape:pure-function |

## Wave: DISTILL / [REF] Walking-Skeleton Strategy

**Brownfield / already shipped.** Per the Architecture of Reference, the driving
port is the CLI; the single `@walking_skeleton @driving_port` scenario (#8) maps
to the **existing** `test_prepare_cv_end_to_end_with_fake_builder` in `main.rs`,
which exercises the full resolve → build → file path with a real tmp filesystem,
a real INI config, and a fake build runner. It is exercised at the
**orchestration layer** (`prepare_cv`), not as a real subprocess — a true
subprocess CLI driving-adapter test is an Open Question / gap (below).

## Wave: DISTILL / [REF] Adapter coverage table

| Adapter (driven) | Treatment | Covered by scenario(s) | Real-IO boundary note |
|------------------|-----------|------------------------|-----------------------|
| `CommandRunner` — build (`just`/`tectonic`) | `FakeRunner` @in-memory | 11, 12, 14 (+WS 8) | Prod `SystemRunner` shells out; real boundary via recommended CI template-contract smoke test (not in these specs). |
| `CommandRunner` — PDF viewer (zathura) | `FakeRunner` @in-memory | 26, 27, 28 | Prod `SystemRunner.spawn`; real viewer launch not exercised. |
| `CommandRunner` — secure-network status (`sudo tailscale`) | `FakeRunner` @in-memory | 22, 23, 24, 25 | Real `tailscale status` not run in tests. |
| `DbConnection` (diesel `MultiConnection`) | in-memory SQLite (`:memory:`) | 17, 18, 19, 20, 21 | Prod Postgres over secure network; not exercised by specs. |
| Filesystem (`create_directory`/copy-out/cleanup) | **real I/O** via `tempfile::TempDir` | 8, 9, 16 (+10 sanitize pure) | Genuine real-IO adapter — `@real-io` equivalent on an isolated tmp tree. |
| Configuration (INI via `GLOBAL_VAR`) | real `configparser`, tmp INI | 8, 9 | Process-global `OnceCell` → determinism via `cargo-nextest` (ADR-0005). |

## Wave: DISTILL / [REF] Test placement

Rust in-crate `#[cfg(test)]` modules beside each source file (precedent: every
existing unit test lives in a `mod tests` next to its code) plus the existing
`tests/integration-tests.rs` harness. The `.feature` documentation SSOT lives
under `tests/acceptance/cv-variant-build/` (new, doc-only — no runner wired).

## Wave: DISTILL / [REF] Driving Adapter coverage

The sole driving adapter is the **`insert` CLI subcommand** (clap). It is covered
at the orchestration level: dispatch via `test_match_user_action_list_arm` /
`_update_arm` (`cli_structure.rs`) and the end-to-end flow via
`test_prepare_cv_end_to_end_with_fake_builder` (`main.rs`). **Gap / Open
Question**: there is no subprocess-level CLI test that spawns the built binary
and asserts exit code + stdout (the architecture-of-reference target mechanism).
Recorded in `docs/architecture/atdd-infrastructure-policy.md` (Driving table).

## Wave: DISTILL / [REF] Pre-requisites

- DESIGN driving port (CLI) + driven ports (`CommandRunner`, `DbConnection`,
  config, filesystem) per `feature-delta.md` DESIGN [REF] and ADR-0001..0005.
- No DEVOPS environment matrix exists; sensible defaults applied (single-user
  local CLI; in-memory SQLite + tmp FS for tests; `cargo-nextest` for
  determinism). Not a blocker.

## Wave: DISTILL / [REF] Self-Completeness Audit (Phase 2.5)

`nw-at-completeness-check` 15-item checklist over the 32 scenarios. Verdict:
**ACCEPTABLE_WITH_DOCUMENTED_GAPS (11/15)**. All gaps are
`AT_GAP_IN_DELIVERY_SCOPE` (no upstream `SPECIFICATION_AMBIGUITY` blockers).

| Item | Status | Note |
|------|--------|------|
| C1a empty/zero input | PASS | Empty-list scenario (#21). |
| C1b partition boundaries | PASS | One representative per variant equivalence class + `None`. |
| C2a state machine documented | PASS (N/A) | Feature is config/decision-shaped, not a state machine. |
| C2b illegal-event-per-state | PASS (N/A) | No state machine. |
| C3 cardinality 0/1/N | PASS | 0 (#21), 1 (#17), many (#20). |
| C4a apply-twice idempotency | PASS | Re-save duplicate (#18). |
| C4b inverse op without prerequisite | **GAP** | No "uninstall/remove without prior insert" test. |
| C5a mode-flag combinations | **GAP** | `--variant` valid/invalid/absent covered; `--save`/`--view` not combinatorially tested. |
| C5b flag orthogonality | **GAP** | Flag-independence not asserted. |
| C6a malformed value per param | PASS | Invalid variant "bogus" (#2), unmatched title (#7). |
| C6b each declared error triggered | PASS | Missing driver/dir/builder-fail/copy/viewer/network errors (#12-15, 24, 25, 28). |
| C6c closed error set asserted | **GAP** | Error set not asserted as closed. |
| C7a degraded-resource condition | PASS | Missing tool on PATH (#29, 31). |
| C7b interruption mid-operation | PASS (N/A) | Single-shot CLI; no mid-flow interruption contract. |
| C7c concurrent actors | PASS (N/A) | Single-user tool by claim. |

Additional documented gaps beyond the checklist: (a) no subprocess CLI
driving-adapter test; (b) no test for the `--save-to-database` omitted →
no-DB-write path (`CV NOT SAVED TO DATABASE!`).

## Wave: DISTILL / [REF] Traceability table: Scenario → existing Rust test(s)

| # | Scenario | US | Existing Rust test(s) | File |
|---|----------|----|------------------------|------|
| 1 | Explicit variant choice honoured | US-01 | `test_resolve_variant_flag_wins` | `src/file_handlers.rs` |
| 2 | Unrecognised choice → inference | US-01 | `test_resolve_variant_invalid_flag_falls_back_to_inference` | `src/file_handlers.rs` |
| 3 | Platform title → platform variant | US-02 | `test_infer_variant_keywords` | `src/file_handlers.rs` |
| 4 | Site-reliability title → reliability variant | US-02 | `test_infer_variant_keywords` | `src/file_handlers.rs` |
| 5 | Devops title → devops variant | US-02 | `test_infer_variant_keywords` | `src/file_handlers.rs` |
| 6 | Manager title wins | US-02 | `test_infer_variant_manager_wins_over_devops` | `src/file_handlers.rs` |
| 7 | Unrecognised title → default | US-02 | `test_resolve_variant_uses_default_when_nothing_matches`, `test_infer_variant_keywords` (Accountant→None) | `src/file_handlers.rs` |
| 8 | CV generated end to end (WS) | US-03 | `test_prepare_cv_end_to_end_with_fake_builder` | `src/main.rs` |
| 9 | Filed under per-year dir + cleanup | US-03 | `test_create_directory_and_remove_flow` | `src/file_handlers.rs` |
| 10 | Spaces → dashes in filed name | US-03 | `test_sanitize_for_path_replaces_spaces` | `src/file_handlers.rs` |
| 11 | Builder recipe invoked | US-01 | `test_compile_cv_success_invokes_builder` | `src/file_handlers.rs` |
| 12 | Build refused — missing driver | US-01 | `test_compile_cv_missing_driver_errors` | `src/file_handlers.rs` |
| 13 | Build refused — missing dir | US-03 | `test_compile_cv_missing_dir_errors` | `src/file_handlers.rs` |
| 14 | Build reports builder failure | US-01 | `test_compile_cv_builder_failure_errors` | `src/file_handlers.rs` |
| 15 | Filing refused — missing PDF | US-03 | `test_copy_to_destination_errors_for_missing_source` (copy-error proxy; the `remove_created_dir_from_pro` missing-PDF guard is exercised indirectly via the flow test — see gap note) | `src/file_handlers.rs` |
| 16 | Working dir removed after filing | US-03 | `test_remove_cv_dir_removes_directory`, `test_create_directory_and_remove_flow` | `src/file_handlers.rs` |
| 17 | Application recorded when opted in | US-04 | `test_save_new_cv_inserts_row` | `src/database.rs` |
| 18 | Re-save keeps single record | US-04 | `test_save_new_cv_is_idempotent_on_duplicate` | `src/database.rs` |
| 19 | Quote stored with application | US-04 | `test_save_new_cv_stores_quote` | `src/database.rs` |
| 20 | Recorded applications listed back | US-04 | `test_read_cv_from_db_returns_paths` | `src/database.rs` |
| 21 | Empty list returns nothing | US-04 | `test_read_cv_from_db_empty` | `src/database.rs` |
| 22 | Connectivity confirmed (network up) | US-04 | `test_is_tailscale_connected_true_when_details` | `src/main.rs` |
| 23 | Connectivity down (logged out) | US-04 | `test_is_tailscale_connected_false_when_logged_out` | `src/main.rs` |
| 24 | Reachability errors — status fails | US-04 | `test_is_tailscale_connected_err_on_command_failure` | `src/main.rs` |
| 25 | Reachability errors — cannot run | US-04 | `test_is_tailscale_connected_err_on_io_error` | `src/main.rs` |
| 26 | PDF opens in configured viewer | US-05 | `test_view_cv_file_spawns_viewer_ok` | `src/helpers.rs` |
| 27 | Source path previewed as PDF | US-05 | `test_view_cv_file_converts_tex_to_pdf` | `src/helpers.rs` |
| 28 | Preview reports viewer error | US-05 | `test_view_cv_file_errors_when_spawn_fails` | `src/helpers.rs` |
| 29 | Missing tools fail fast + hint | US-05 | `test_ensure_tools_available_errors_and_hints_devenv` | `src/helpers.rs` |
| 30 | Tool check passes when none required | US-05 | `test_ensure_tools_available_ok_for_empty` | `src/helpers.rs` |
| 31 | Tool absent reported unavailable | US-05 | `test_tool_on_path_false_for_missing_tool` | `src/helpers.rs` |
| 32 | Tool present reported available | US-05 | `test_tool_on_path_true_for_installed_tool` | `src/helpers.rs` |

**Coverage of user stories**: US-01 (#1,2,11,12,14) ✓ · US-02 (#3-7) ✓ ·
US-03 (#8,9,10,13,15,16) ✓ · US-04 (#17-25) ✓ · US-05 (#26-32) ✓ — all 5 covered.

**Supporting tests not mapped to a user-story scenario** (internal helpers /
out-of-scope / test-infra): `test_check_dir_exists_*`, `test_check_file_exists_*`,
`test_copy_to_destination_copies_file` (filing helper happy-path),
`test_fake_runner_records_and_returns`, `test_fake_runner_io_error` (port self-tests),
`test_match_user_action_list_arm` / `_update_arm` (out-of-scope subcommands),
`test_parse_date_*` (dead code, `#[allow(dead_code)]`), `test_filter_args_default`,
`test_insert_cv_panics_without_global` (config-guard), and the
`clean_string_from_quotes` / `fix_home_directory_path` / `check_config_file_exists`
utility tests.

## Wave: DISTILL / [REF] Self-Review Checklist

- WS strategy declared (brownfield; maps to existing e2e test) — ✓
- WS scenario tagged `@walking_skeleton @driving_port` — ✓ (#8)
- Every driven adapter has coverage; real-IO boundary documented per adapter — ✓
- In-memory doubles documented for what they cannot model (no real `just`,
  no real Postgres, no real viewer, no real `tailscale`) — ✓
- Mandate-7 RED scaffolding — N/A (shipped GREEN), documented — ✓
- Business language only in scenario text (no `just`/`diesel`/`SQLite`/HTTP in
  Given/When/Then) — ✓
- Error/edge ratio ≥40% — ✓ (59%)
- Every scenario carries a `@contract-shape:` tag (Mandate 14) — ✓
- Reconciliation HARD GATE passed (0 contradictions) — ✓
- AT-completeness verdict ≥ ACCEPTABLE_WITH_DOCUMENTED_GAPS — ✓ (11/15)
- 4-reviewer consolidated gate — intentionally skipped by parent for this
  shipped-code backfill (noted).

---

## Wave: DELIVER / [REF] Retroactive Backfill Note

> **Retroactive backfill** (DES-EXEMPT). These DELIVER sections document the
> *already-shipped* feature (branch `feature/change-generation`, v4.0.2): 6
> merged commits, 80/80 tests GREEN under `cargo nextest run`, ~84% line
> coverage. **No DES execution-log, no DES markers, no TDD cycle, no crafter
> dispatch, no source changes** were produced for this backfill — only the
> DELIVER paper trail. Roadmap → commit mapping lives in `../roadmap.json`.
> Density: **LEAN**. Reconciled with DISCUSS/DESIGN/DISTILL [REF] above; no
> contradictions. Date: 2026-06-20.

## Wave: DELIVER / [REF] Implementation Summary

The variant-aware CV build flow shipped as four delivered steps: a diesel
`MultiConnection` `DbConnection` for backend-agnostic persistence (`6472189`); a
`CommandRunner` seam isolating all subprocess effects behind an injectable port
(`23fde25`); the variant build subsystem itself — resolution precedence
(flag → inference → default), config-driven `just build <variant>` (tectonic),
deterministic per-year PDF filing with working-dir cleanup, and pre-usage PATH
tool checks with a devenv hint (`beb5034`); and coverage discipline via
`llvm-cov coverage(off)` exclusions plus schema exclusion that raised line
coverage 54%→84% (`d34990e`). The work realizes DISCUSS RD-1..RD-5 and DESIGN
D-1..D-5 / ADR-0001..0005 with no contradictions.

## Wave: DELIVER / [REF] Files Modified

**Production (`src/*.rs`)** — from `git show --stat`:
- `src/database.rs` — `DbConnection` `MultiConnection` + CRUD on `&mut DbConnection` (6472189).
- `src/user_action.rs` — CRUD call-sites adapted to `&mut DbConnection` (6472189).
- `src/command_runner.rs` — new `CommandRunner` port + `SystemRunner`/`FakeRunner` (23fde25).
- `src/file_handlers.rs` — variant resolve + `compile_cv` + copy-out/cleanup (beb5034).
- `src/main.rs` — `prepare_cv` orchestration + `is_tailscale_connected` via runner (beb5034).
- `src/cli_structure.rs` — `--variant` flag on `FilterArgs` (beb5034).
- `src/cv_insert.rs` — wires variant resolution into the insert use case (beb5034).
- `src/global_conf.rs` — `get_variant` accessor (beb5034).
- `src/helpers.rs` — `ensure_tools_available`/`tool_on_path`, `view_cv_file` via runner (beb5034).
- `src/lib.rs` — `coverage_nightly` attribute gate (d34990e).

**Tests** — in-crate `#[cfg(test)] mod tests` beside each source file (database,
command_runner, file_handlers, helpers, main, cli_structure) + `tests/integration-tests.rs`.

**Config / build** — `rusty-cv-config-example.ini` (variant/build keys, beb5034);
`Cargo.toml` (coverage profile, d34990e); `.github/workflows/rust-tests.yml`
(llvm-cov exclusions, d34990e); `devenv.nix` (toolchain, 865780d).

**Docs** — DESIGN brief, ADR-0001..0005, c4-diagrams (c3807a7); this delta,
roadmap.json, evolution record (this DELIVER backfill).

## Wave: DELIVER / [REF] Scenarios Green

- **32 of 32** DISTILL acceptance scenarios mapped to GREEN Rust tests (see DISTILL
  Traceability table above; the `.feature` files are a documentation SSOT — no
  cucumber harness).
- **80 of 80** unit/integration tests pass under `cargo nextest run`.
- Date verified: **2026-06-20**.

## Wave: DELIVER / [REF] DoD Check

Against the DISCUSS Definition of Done:
- [x] All five stories' ACs demonstrable via `insert` — mapped to GREEN tests (#1-32).
- [x] Variant precedence (flag → inference → default) unit-covered —
  `test_resolve_variant_*`, `test_infer_variant_*`.
- [x] Output PDF under per-year dir, deterministic name, working dir cleaned up —
  `test_create_directory_and_remove_flow`, `test_sanitize_for_path_replaces_spaces`.
- [x] Persistence & view opt-in per RD-4/RD-5 — `test_save_new_cv_*`,
  `test_view_cv_file_*`. **Documented gap**: no test for the `--save-to-database`
  *omitted* (no-write) path.
- [x] No contradiction with DESIGN [REF] / ADR-0001..0005 — merged, v4.0.2.

## Wave: DELIVER / [REF] Demo Evidence

Non-destructive CLI driving-port evidence (binary `./target/debug/rusty_cv_creator`):
- `rusty_cv_creator --help` → exit **0**; lists `insert/update/remove/list` and
  global flags (`--save-to-database`, `--view-generated-cv`, `--dry-run`,
  `--config-ini`, `--engine`).
- `rusty_cv_creator insert --help` → exit **0**; shows `--variant` (with the 4
  valid variants + inference/default behavior) plus `--job-title`,
  `--company-name`, `--quote`, `--date`.

A real `insert` was **not** run: it is side-effectful and requires the external
CV template repo (`git@github.com:chess-seventh/cv.git`), `just`+`tectonic`, and
Postgres at runtime. Full end-to-end build is covered by the
`test_prepare_cv_end_to_end_with_fake_builder` test (orchestration layer, real
tmp FS + fake build runner).

## Wave: DELIVER / [REF] Quality Gates

- **clippy** — `-D warnings` clean + pedantic clean.
- **tests** — `cargo nextest run` 80/80 GREEN.
- **coverage** — 54% → **84%** line coverage (`cargo-llvm-cov`, ADR-0005).
- **refactor / mutation / review** — **N/A (retroactive)**; no TDD cycle was run
  for this backfill (DES-EXEMPT). Mutation strategy is a forward DEVOPS Decision-9
  concern, not applicable to shipped code.

## Wave: DELIVER / [REF] Pre-requisites

Inherited from DISCUSS/DISTILL Pre-requisites (unchanged): external CV template
repo exposing `just build <variant>` → `<prefix>-<variant>.pdf`; PATH tools
`just`/`tectonic` (+`zathura` for view, `sudo`+`tailscale` for Postgres
reachability), all devenv-provisioned and pre-usage checked (ADR-0004); INI
config; Postgres (prod) or SQLite (local/tests).
