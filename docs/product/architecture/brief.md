# Architecture Brief — rusty_cv_creator

> SSOT bootstrap (greenfield documentation backfill). This brief is the single
> source of truth for the system architecture. It was authored **retroactively**
> against the implemented code on branch `feature/change-generation` (v4.0.2).
> Density: **LEAN** (Tier-1 [REF] sections only).

## Prior Wave Consultation

| Artifact | Wave | Status | Note |
| --- | --- | --- | --- |
| `docs/product/architecture/brief.md` (System Architecture) | DESIGN (Titan) | ⊘ absent | This file bootstraps the SSOT; no prior `## System Architecture` section. |
| `docs/product/architecture/brief.md` (Domain Model) | DESIGN (Hera) | ⊘ absent | No prior `## Domain Model`. Domain captured inline below. |
| DISCUSS requirements / user stories / AC | DISCUSS | ⊘ absent | LEAN retroactive backfill; requirements reverse-engineered from code + commits. |
| `docs/product/prd.md` | PRODUCT | ⊘ absent | Greenfield bootstrap. |

**Mode**: PROPOSE. **Scope**: APPLICATION / components. This is a RETROACTIVE
backfill — the implementation already exists and is merged; these artifacts
document the realized architecture and decisions, they do not propose new work.

---

## System Context and Capabilities

`rusty_cv_creator` is a single-user CLI that generates tailored, per-application
CV PDFs from a LaTeX template repository and records them in a database.

Capabilities:

- Select one of **4 CV variants** (`senior-devops`, `senior-platform-engineer`,
  `senior-sre`, `engineering-manager`) per job application.
- Build the selected variant via a config-driven `just build <variant>`
  (Tectonic under the hood) inside a dated working copy of the template.
- Copy the produced PDF to a per-year output location and to a sibling of the
  working directory, then clean up the working directory.
- Persist CV metadata (job title, company, quote, path, date) to Postgres (prod,
  reached over Tailscale) or SQLite (tests / local).
- Optionally open the produced PDF in a configured viewer (zathura).

---

## Application Architecture

### Architectural style

**Modular monolith, ports-and-adapters (hexagonal)**. A single Rust binary with
a thin imperative shell (CLI parsing, orchestration, side effects) around
testable core logic. Dependency-inversion is realized through two explicit
runtime-injected ports plus two implicit configuration/filesystem ports.

This is the simplest style that satisfies the actual quality drivers
(testability, single-user operability). Microservices / async messaging /
layered frameworks were unnecessary and are not used. See ADR-0002, ADR-0003.

### Component decomposition

| Component | Path | Responsibility |
| --- | --- | --- |
| Entrypoint / Orchestrator | `src/main.rs` | `main`, `prepare_cv` (build orchestration), `is_tailscale_connected`. |
| CLI (driving adapter) | `src/cli_structure.rs` | clap `UserInput`/`UserAction`/`FilterArgs` (incl. `--variant`); `match_user_action` dispatch. |
| Insert use case | `src/cv_insert.rs` | `insert_cv` — wires variant resolution → build → optional persist. |
| Remove/List use case | `src/user_action.rs` | `remove_cv`, `show_cvs` (interactive selection via skim). |
| Build subsystem | `src/file_handlers.rs` | `resolve_variant`, `BuildConfig`, `compile_cv`, directory prep, PDF copy-out + cleanup. |
| Command port | `src/command_runner.rs` | `CommandRunner` trait + `SystemRunner` (prod) + `testing::FakeRunner`. |
| Persistence | `src/database.rs` | `DbConnection` (diesel `MultiConnection`), connection + CRUD functions. |
| Domain model | `src/models.rs`, `src/schema.rs` | diesel `Cv`/`NewCv`, `cv` table. |
| Config | `src/config_parse.rs`, `src/global_conf.rs` | INI load + immutable injected `AppContext` (ADR-0006), typed accessors. |
| Template sourcing | `src/template_source.rs` | `TemplateSource` port + `LocalDirectory`/`GitHubRepository` adapters, `TemplateCache`, `detect_template_source` (ADR-0008). |
| Helpers | `src/helpers.rs` | `ensure_tools_available`/`tool_on_path`, `view_cv_file`, `my_fzf`, path utils. |
| Library facade | `src/lib.rs` | exposes `models` + `schema` + `tui`; enables `coverage_nightly` attribute. |
| TUI | `src/tui/*.rs` | ratatui terminal UI (feature `tui-job-applications`) to browse/filter applications + open PDFs — `app`/`state`/`events`/`ui`/`db`/`terminal_guard`. |

### Ports and adapters

**Driving ports (inbound)**

- **CLI** — clap parses `UserInput`; `match_user_action` dispatches to use cases.
- **TUI** — a ratatui terminal UI (feature `tui-job-applications`) launched from the
  CLI; reads applications from the DB and opens PDFs. ADRs under
  `docs/feature/tui-job-applications/architecture/` (adr-001 ratatui, adr-002 module structure).

**Driven ports (outbound)**

- **`CommandRunner`** (subprocess effects) — `status`/`output`/`spawn` (plus a
  stderr-capturing run added for template sourcing, see below).
  Adapters: `SystemRunner` (real `std::process::Command`), `FakeRunner` (tests).
  Injected into `compile_cv`, `view_cv_file`, `is_tailscale_connected`, and the
  `TemplateSource` resolver. See ADR-0002.
- **`TemplateSource`** (template resolution) — resolves the configured
  `cv_template_path` to a local template directory: `LocalDirectory` (passthrough,
  backward compat) or `GitHubRepository` (git clone/fetch/checkout via
  `CommandRunner`). See ADR-0008 and the `### Template sourcing` subsection below.
- **`DbConnection`** (persistence) — diesel `MultiConnection` enum.
  Adapters: `PgConnection` (prod), `SqliteConnection` (tests / local). Functions
  take `&mut DbConnection`. See ADR-0003.
- **Configuration** (INI) — `configparser` read into an immutable `AppContext`
  built in `main` and injected by borrow (ADR-0006; no process-global cell).
  Typed accessors in `global_conf.rs`. Output dir, prefix, builder, recipe,
  default variant, DB engine all config-driven.
- **Filesystem** — `copy_dir` (template copy) + `std::fs` (copy/cleanup) behind
  the `file_handlers` functions.

External tools (`just`, `tectonic`, `pdf_viewer`/zathura, `sudo`+`tailscale`)
are gated by **pre-usage PATH checks** (`ensure_tools_available`) at the
orchestration layer before any subprocess runs. See ADR-0004.

### Template sourcing

> Feature `template-source` (branch `feature/template-source-skeleton`). Lets
> `[cv] cv_template_path` be a GitHub git URL as well as a local directory, with
> optional ref pinning, auth, and an offline cache. See ADR-0008 and
> `docs/feature/template-source/`.

The `TemplateSource` driven port resolves the configured template value to a
local directory that the existing `copy_dir` flow consumes unchanged (D5). A
factory `detect_template_source` auto-detects (D1): an existing readable
directory → `LocalDirectory` (passthrough, the backward-compat path); a git URL
(`git@…`, `https://….git`, or `file://…`) → `GitHubRepository`.

`GitHubRepository` shells out to the system `git` binary through the existing
`CommandRunner` port (no Rust git crate; ADR-0008), gated by the ADR-0004
pre-usage PATH check. It caches per `repo@ref` under `[cv] cv_template_cache`
(default `~/.cache/rusty-cv-creator/templates`), reusing an entry via
`fetch`+`checkout` rather than re-cloning. A `TemplateCache` collaborator owns the
cache-key derivation and a **pure** reuse-vs-fetch-vs-abort decision
(`CacheAction`); only the executor touches disk, and its write universe is bounded
to the cache dir. Ref pinning (`[cv] cv_template_ref`) checks out the
branch/tag/SHA and logs the resolved SHA, aborting (never silently falling back)
on an unresolvable ref. Auth (`[cv] cv_template_auth = auto|ssh|token`) is
inferred from the URL scheme by default; `token` reads `GITHUB_TOKEN` from the
environment and feeds git via `core.askpass`, never via the INI, the argv, or the
cache repo's `.git/config`. Failures are classified by a typed
`TemplateSourceError` enum (auth vs network/offline vs bad-ref vs no-cache) so
each emits a distinct actionable hint; classification reads git's stderr, for
which the `CommandRunner` port gains an additive stderr-capturing run.

### Technology stack (pinned, from `Cargo.toml`)

| Dependency | Version | Role | License |
| --- | --- | --- | --- |
| Rust | edition 2021, **nightly** channel | language/toolchain (nightly for `coverage_attribute`) | — |
| clap | 4.6.1 (`derive`) | CLI parsing (driving adapter) | MIT/Apache-2.0 |
| diesel | 2.3.9 (`sqlite`,`postgres`,`returning_clauses_for_sqlite_3_35`) | ORM / persistence port | MIT/Apache-2.0 |
| configparser | 3.1.0 | INI config | MIT/Apache-2.0 |
| chrono | 0.4.44 | dates (dir layout, application date) | MIT/Apache-2.0 |
| copy_dir | 0.1.3 | recursive template copy | MIT |
| dirs | 6.0.0 | home-dir expansion | MIT/Apache-2.0 |
| dotenvy | 0.15.7 | `.env` loading | MIT |
| env_logger / log | 0.11.10 / 0.4.30 | logging | MIT/Apache-2.0 |
| skim | 4.6.2 | interactive selection (`my_fzf`) | MIT |
| tempfile (dev) | 3.27.0 | test fixtures | MIT/Apache-2.0 |
| serial_test (dev) | 3.5.0 | serialize PATH-mutating tests | MIT |

External build/runtime tools (not crates): `just`, `tectonic` (devenv),
`zathura`, `tailscale`+`sudo`; toolchain provided by **devenv**.
Package: `rusty_cv_creator` v4.0.2, **MIT**. All dependencies are OSS.

### Paradigm

Rust, multi-paradigm. Realized as **struct + trait (OOP-leaning)**: ports are
traits, adapters are structs, use cases are free functions. Recommended to keep
this paradigm for future work. Pure helpers (`resolve_variant`,
`infer_variant_from_job_title`, `sanitize_for_path`) form a small functional
core; effects (subprocess, fs, db) live in the shell and behind ports.

### Quality attributes (ISO 25010, realized)

- **Maintainability / Testability** — primary driver. Ports + `MultiConnection`
  enable fakes / in-memory SQLite; line coverage 54% → 84% (ADR-0005).
- **Portability** — backend-agnostic persistence; devenv-provisioned toolchain.
- **Reliability** — pre-usage tool checks fail fast with a `devenv` hint
  (ADR-0004) instead of cryptic subprocess errors.
- **Functional suitability** — variant resolution precedence
  (flag → inference → default) covers explicit and implicit selection.
- **Security** — single-user local tool; Postgres reached only over Tailscale;
  no secrets in repo (pre-commit hooks for keys/AWS creds in devenv).

### Decisions table

| ID | Decision | ADR |
| --- | --- | --- |
| D-1 | Variant-based CV build via Justfile (replaces xelatex + placeholder). | [ADR-0001](adr-0001.md) |
| D-2 | `CommandRunner` port for all subprocess side-effects. | [ADR-0002](adr-0002.md) |
| D-3 | diesel `MultiConnection` for backend-agnostic persistence. | [ADR-0003](adr-0003.md) |
| D-4 | Pre-usage tool-availability checks at orchestration layer. | [ADR-0004](adr-0004.md) |
| D-5 | Coverage discipline via test seams + `coverage_nightly` gating. | [ADR-0005](adr-0005.md) |
| D-6 | Inject immutable `AppContext` (`&AppContext`) instead of `GLOBAL_VAR` `OnceCell`. | [ADR-0006](adr-0006-inject-appcontext.md) |
| D-7 | CI quality gates made blocking (clippy `-D warnings`, rustfmt, threaded `cargo test`) + single release mechanism (`release.yml`; dormant `.releaserc*` removed). | [ADR-0007](adr-0007-ci-quality-gates-single-release.md) |
| D-8 | Template sourcing via system `git` shell-out through `CommandRunner` (cache by `repo@ref`, ref pinning, env-only token). | [ADR-0008](adr-0008-template-source.md) |

### Component Inventory — delivery status

> Marked **delivered** by the DELIVER wave (feature `cv-variant-build`, v4.0.2).
> See `docs/feature/cv-variant-build/roadmap.json` and
> `docs/evolution/cv-variant-build-evolution.md`.

| Component | Delivered by | Status |
| --- | --- | --- |
| `DbConnection` (diesel `MultiConnection`) | `6472189` | delivered |
| `CommandRunner` port + `SystemRunner`/`FakeRunner` | `23fde25` | delivered |
| Variant build flow (resolve → `compile_cv` → per-year filing + cleanup) | `beb5034` | delivered |
| Pre-usage tool checks (`ensure_tools_available`/`tool_on_path`) | `beb5034` | delivered |
| Coverage discipline (`coverage(off)` exclusions, 54%→84%) | `d34990e` | delivered |

`list`/`update`/DB-filtering and `parse_date` wiring remain partial (carried
forward as gaps — see evolution record).

> Marked **delivered** by the DELIVER wave (feature `template-source`, branch
> `feature/template-source-skeleton`). See
> `docs/feature/template-source/feature-delta.md` and
> `docs/evolution/template-source-evolution.md`.

| Component | Delivered by | Status |
| --- | --- | --- |
| `CommandRunner::run_capturing` (UC-1 stderr seam) + `CommandOutcome` | `c144d40` | delivered |
| `TemplateSourceError` enum + `classify_git_stderr` | `037c778` | delivered |
| `AuthMode` + `auth_invocation_flags` (SSH / token via `core.askpass`) | `361b466` | delivered |
| Ref pinning (`with_ref` + checkout + resolved-SHA log) | `8887c2f` | delivered |
| `TemplateCache` (`decide` / `CacheAction` matrix + deterministic `cache_key`) | `07f7831` | delivered |
| `TemplateSource` / `LocalDirectory` / `GitHubRepository` wired into `prepare_path_for_new_cv` | `52457d1` | delivered |

Mutation: 98.1% kill on `src/template_source.rs`. DoD item 6 (HTTPS-token askpass
helper executable) is **partial/deferred** — SSH path fully works; see the
`template-source` evolution record for the two follow-ups.

### External integrations (contract-test annotation for DEVOPS handoff)

The highest-risk boundary is the **CV template repository**
(`git@github.com:chess-seventh/cv.git`) whose `Justfile` recipe contract
(`just build <variant>` producing `<prefix>-<variant>.pdf`) is consumed by
`compile_cv`. This is a build-time integration, not a web API, so consumer-driven
HTTP contract tooling (Pact) does not apply. Recommended instead:

- A **template-contract smoke test** in CI that runs `just build <variant>` for
  each of the 4 variants and asserts the expected PDF basename appears (verifies
  the recipe-name and output-naming contract that `compile_cv` assumes).
- Postgres reachability (over Tailscale) is environment, not a versioned API;
  covered by the pre-usage `tailscale status` probe (ADR-0004).

### Decided (was open)

- **`GLOBAL_VAR` `OnceCell` → injected `AppContext`** — **DELIVERED 2026-06-20**
  (commit `5214f33`). The process-global config was replaced by an immutable
  `AppContext` value threaded by borrow (`&AppContext`) through the use cases;
  `GLOBAL_VAR` and `once_cell` are gone and plain threaded `cargo test` is now
  85/85 green (was 3 failing). See [ADR-0006](adr-0006-inject-appcontext.md),
  feature `config-injection`. **Supersedes the "GLOBAL_VAR OnceCell" open item
  deferred in [ADR-0005](adr-0005.md).**

### Open questions / known smells

- **`parse_date`** (`cli_structure.rs`) — `#[allow(dead_code)]`; intended for
  filter parsing, not yet wired.
- **Filters not applied on DB** — `read_cv_from_db` / `show_cvs` carry
  `FilterArgs` but `// TODO filters on proper DB`; currently `limit(50)` only.

---

## C4 Diagrams

See [c4-diagrams.md](c4-diagrams.md) — System Context (L1), Container (L2), and a
Component (L3) diagram for the CV-build subsystem.
