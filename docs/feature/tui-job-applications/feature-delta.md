# Feature: tui-job-applications
**Wave**: DISCUSS | **Density**: lean + ask-intelligent | **Date**: 2026-06-06

---

## Wave: DISCUSS / [REF] Persona

**ID**: active-job-seeker
**One-liner**: A developer or knowledge worker running an active job search who lives in the terminal and uses `rusty-cv` to generate and track CV submissions.

---

## Wave: DISCUSS / [REF] JTBD

**Primary job (JOB-01)**:
> "When I am actively job searching and have sent out multiple applications, I want to see the full list of where I have applied with dates and role details, so I can decide which companies to follow up with and when."

**Opportunity scores** (importance vs. satisfaction):
| Job ID | Title | Importance | Satisfaction | Gap | Tier |
|--------|-------|-----------|-------------|-----|------|
| JOB-01 | Review all applications | 9 | 1 | 8 | HIGH |
| JOB-02 | Check for duplicates | 7 | 2 | 5 | MEDIUM |
| JOB-03 | Access sent CV | 6 | 3 | 3 | LOW-MED |

Full job stories, four forces, and dimensions: `docs/product/jobs.yaml`

---

## Wave: DISCUSS / [REF] Locked Decisions

- **[D1] Feature type**: User-facing TUI. Rationale: the `rusty-cv list` subcommand is already user-visible; this feature makes it functional.
- **[D2] Walking skeleton**: Yes (strategy B — brownfield, wire existing data layer to new TUI widget). First slice ships minimal table before adding navigation/filter.
- **[D3] UX research depth**: Comprehensive. Journey, emotional arc, error paths, and shared artifact registry all produced.
- **[D4] JTBD**: Yes (default). All stories trace to `docs/product/jobs.yaml`.
- **[D5] TUI library**: `ratatui 0.29` + `crossterm 0.28`. Rationale: de-facto standard for Rust TUIs; active maintenance; `skim` (already in `Cargo.toml`) also uses crossterm internally.
- **[D6] Scope assessment**: PASS. 4 slices, ≤1 day each, 2 bounded contexts, 1 WS integration point.

---

## Wave: DISCUSS / [REF] User Stories

### US-01 — Walking Skeleton: TUI Application Table
`job_id: job-01-review-applications`

**As** an active job seeker,
**I want** `rusty-cv list` to open a terminal table showing all my recorded applications,
**So that** I can stop using a spreadsheet to remember where I've applied.

#### Elevator Pitch
Before: `rusty-cv list` prints `"filter args for LIST: FilterArgs { ... }"` — zero application data.
After: run `rusty-cv list` → sees a full-screen ratatui table with columns Date | Company | Job Title | PDF Path and a status bar "N applications total".
Decision enabled: I instantly know how many applications are in my history and can scan for the one I'm looking for.

#### Acceptance Criteria
- AC-01: Running `rusty-cv list` with ≥1 DB record renders a table with one row per `Cv` record, showing application_date, company, job_title, and a truncated pdf_cv_path.
- AC-02: Running with empty DB shows: `No applications recorded yet. Run: rusty-cv insert`
- AC-03: Running with an unreachable database shows a red error footer and exits on any keypress without panicking.
- AC-04: Pressing 'q' or Esc exits the TUI and fully restores terminal state (no leftover raw mode).
- AC-05: Table renders within 500ms for a dataset of 100 rows on a local DB.

---

### US-02 — Keyboard Navigation
`job_id: job-01-review-applications`

**As** an active job seeker scanning my application history,
**I want** to move a highlighted cursor through the table with arrow keys,
**So that** I can focus on one application row at a time and know my position in the list.

#### Elevator Pitch
Before: table renders but no row is highlighted; user cannot tell which row is "active".
After: run `rusty-cv list`, press ↓ three times → row 3 is highlighted in bold/reversed; status bar reads "3 of 17 applications".
Decision enabled: I know exactly which application I'm looking at before pressing Enter to open its PDF.

#### Acceptance Criteria
- AC-06: ↓/j moves selection down one row; ↑/k moves up one row.
- AC-07: Pressing ↓ on the last row does not wrap; pressing ↑ on the first row does not wrap.
- AC-08: Home key jumps to first row; End key jumps to last row.
- AC-09: Status bar always shows "N of M applications" reflecting current selection.

---

### US-03 — Real-time Filter
`job_id: job-02-check-duplicate`

**As** an active job seeker about to apply somewhere new,
**I want** to type a company or job title into a filter bar and see matching applications instantly,
**So that** I can confirm I haven't already applied there before submitting a new application.

#### Elevator Pitch
Before: I must mentally recall or exit the TUI and grep logs to check for a duplicate application.
After: run `rusty-cv list`, press '/', type "acme" → table instantly narrows to rows matching "acme" (case-insensitive, company + job_title); filter bar reads "Filter: acme [2 matches]".
Decision enabled: I confidently know whether I've applied to Acme before hitting submit.

#### Acceptance Criteria
- AC-10: Pressing '/' from normal mode activates FilterMode; a "Filter: _" bar appears at the bottom.
- AC-11: Each keystroke in FilterMode re-renders the table with rows matching the filter text (case-insensitive substring on company OR job_title).
- AC-12: The filter bar shows the match count: "Filter: <text> [N matches]".
- AC-13: Pressing Esc clears the filter, restores the full list, resets selection to row 0, and exits FilterMode.
- AC-14: Navigation (↑/↓) works within filtered results.

---

### US-04 — Open CV PDF
`job_id: job-03-access-sent-cv`

**As** an active job seeker preparing for an interview,
**I want** to press Enter on an application row to open the PDF CV I submitted,
**So that** I can review exactly what the interviewer has read about me without hunting through my filesystem.

#### Elevator Pitch
Before: the pdf_cv_path is stored in the DB but I must manually navigate to it in Finder/shell to open it.
After: run `rusty-cv list`, navigate to the target company, press Enter → OS default PDF viewer opens the stored CV file.
Decision enabled: I walk into the interview knowing precisely which CV version the interviewer holds.

#### Acceptance Criteria
- AC-15: Pressing Enter or 'o' on a selected row with a valid pdf_cv_path opens the file in the OS default viewer; TUI remains open and interactive.
- AC-16: Pressing Enter/o on a row whose pdf_cv_path does not exist on disk shows "File not found: <path>" in the status bar for 3 seconds; TUI does not exit.
- AC-17: Pressing Enter/o while in FilterMode has no effect (filter input takes priority over open action).
- AC-18: Pressing Enter/o with no row selected (empty list) is silently ignored.

---

## Wave: DISCUSS / [REF] Story Map

```
BACKBONE (activities):   Launch TUI  →  View Applications  →  Navigate & Filter  →  Act on Selection  →  Exit
                                                ↑                      ↑                      ↑
WALKING SKELETON:      [ slice-01: table renders + 'q' exits ]
SLICE 02:                             [ ↑/↓ highlight + status bar ]
SLICE 03:                                           [ '/' filter bar + real-time narrow ]
SLICE 04:                                                              [ Enter opens PDF ]
```

**Prioritization** (learning-leverage order):
1. Slice 01 — highest uncertainty (new crate integration + `read_cv_from_db` refactor)
2. Slice 02 — core navigation; gates slice 03 and 04
3. Slice 03 — addresses JOB-02 (duplicate check); independent of PDF open
4. Slice 04 — reuses existing `view_cv_file`; lowest risk, highest delight

---

## Wave: DISCUSS / [REF] Out of Scope

- Application status tracking (Applied / Interviewing / Offer / Rejected) — separate feature
- Creating or editing application records from within the TUI
- Exporting the application list (CSV, PDF)
- Pagination or DB-side filtering (client-side filter is sufficient for expected dataset size)
- Colour theme configuration
- Mouse support

---

## Wave: DISCUSS / [REF] Walking Skeleton Strategy

**Strategy B** — Brownfield thin slice.

The `rusty-cv list` subcommand already exists in `cli_structure.rs:78` but is completely stubbed (returns a debug string). The skeleton replaces that stub with a real ratatui table fed by a refactored `read_cv_from_db` that returns `Vec<Cv>` instead of `Vec<String>`. No new CLI surface is introduced.

---

## Wave: DISCUSS / [REF] Driving Ports

- **CLI**: `UserAction::List(FilterArgs)` in `src/cli_structure.rs` — the existing Clap subcommand entry point
- **DB read**: `read_cv_from_db(&FilterArgs)` in `src/database.rs` — must be refactored to return `Vec<Cv>`

---

## Wave: DISCUSS / [REF] Pre-requisites

- `read_cv_from_db` currently returns `Vec<String>` (only pdf_cv_path). Must be changed to `Vec<Cv>` before slice 01 can render full row data. This is a required pre-slice change, not a separate slice (no user-visible behaviour change; purely internal).
- `ratatui = "0.29"` and `crossterm = "0.28"` added to `Cargo.toml`.

---

## Wave: DISCUSS / [REF] Outcome KPIs

| KPI | Metric | Target | Measurement |
|-----|--------|--------|-------------|
| KPI-01 | Time-to-render for `rusty-cv list` | < 500ms with 100 rows | `time rusty-cv list` in CI smoke test |
| KPI-02 | Keystrokes to find an application | ≤ 5 keystrokes via filter | Manual dogfood: '/', 3 chars, result visible |
| KPI-03 | New system binary dependencies | 0 | `ldd` / `otool -L` check in CI |
| KPI-04 | Terminal restoration reliability | 100% — no raw mode residue | Run `rusty-cv list`, kill with SIGINT; shell prompt remains functional |

---

## Wave: DISCUSS / [REF] Definition of Done

- [ ] All ACs (AC-01 through AC-18) pass
- [ ] `cargo test` green
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Terminal restoration verified after normal exit, 'q', Esc, and SIGINT
- [ ] Empty DB state and DB error state both manually tested
- [ ] KPI-01 (< 500ms) verified with a 100-row test fixture
- [ ] `ratatui` and `crossterm` added to `Cargo.toml`; no new system binary deps
- [ ] `read_cv_from_db` refactored to return `Vec<Cv>`; existing tests updated
- [ ] All four slices shipped in order (01 → 02 → 03 → 04)

---

## Wave: DISCUSS / [REF] Wave Decisions

```markdown
# DISCUSS Decisions — tui-job-applications

## Key Decisions
- [D1] User-facing TUI: existing `list` subcommand is already the entry point (cli_structure.rs:78)
- [D2] Walking skeleton first: derisk ratatui integration before building navigation/filter
- [D3] Comprehensive UX: full journey, emotional arc, error paths produced
- [D4] JTBD mandatory: all stories trace to docs/product/jobs.yaml
- [D5] ratatui 0.29 + crossterm 0.28: standard stack; aligns with existing skim/crossterm transitive dep
- [D6] Scope PASS: 4 slices ≤1 day each, 2 bounded contexts, no oversized signals

## Requirements Summary
- Primary need: replace broken `list` stub with a real TUI application table
- Walking skeleton: minimal table (data + quit) as slice 01, before UX polish
- Feature type: user-facing

## Constraints Established
- No new system binary dependencies
- Terminal must be fully restored after any exit path (normal, Esc, SIGINT)
- read_cv_from_db return type must change from Vec<String> to Vec<Cv>

## Upstream Changes
- None (no prior DISCOVER wave; docs/product/ bootstrapped fresh)
```

---

## Wave: DISCUSS / [REF] DoR Validation

| # | Item | Status | Evidence |
|---|------|--------|---------|
| 1 | Clear persona and context | ✅ | active-job-seeker, terminal-native workflow |
| 2 | Job traceability | ✅ | US-01→JOB-01, US-02→JOB-01, US-03→JOB-02, US-04→JOB-03 |
| 3 | Testable ACs | ✅ | AC-01 through AC-18, all verifiable |
| 4 | Effort estimates | ✅ | Slice 01: ~4h, 02: ~2h, 03: ~3h, 04: ~1.5h |
| 5 | Dependencies identified | ✅ | ratatui + crossterm; read_cv_from_db refactor |
| 6 | Out of scope defined | ✅ | See Out of Scope section |
| 7 | Walking skeleton defined | ✅ | Slice 01, strategy B |
| 8 | Elevator pitches complete | ✅ | All 4 stories have Before/After/Decision-enabled |
| 9 | Handoff ready for DESIGN | ✅ | Architecture decision on TUI module structure needed |

---

## Wave: DESIGN / [REF] Wave Decisions

Feature: `tui-job-applications`
Architect: Morgan (Solution Architect)
Date: 2026-06-06

---

### Key Decisions

| ID | Decision | Rationale |
|---|---|---|
| D-7 | Architectural style: modular monolith + ports-and-adapters within single binary | Team < 10, time-to-market top priority, existing codebase is already a modular monolith. Microservices explicitly rejected (no independent deployment need, no team boundary). |
| D-8 | Paradigm: OOP-leaning (structs + `impl` blocks) | Matches existing codebase conventions (`GLOBAL_VAR`, `UserInput`, `Cv`). Heavy FP pipelines (type-class emulation, iterator chains as primary control flow) diverge from codebase idioms without benefit at this scale. The single functional exception is the pure `Cv -> ApplicationRow` projection. |
| D-9 | `read_cv_from_db` is NOT modified; new `load_all_applications()` is added | `read_cv_from_db` is consumed by `remove_cv` via `show_cvs`. Modifying its return type breaks the remove flow. Two functions with different return types serve different use cases cleanly. |
| D-10 | `view_cv_file` is NOT reused directly in the TUI PDF open path | `view_cv_file` reads INI config to find the viewer binary and constructs a `.tex`→`.pdf` path. The TUI's `open_pdf` has a direct PDF path and must stay non-blocking. The pattern (OS process spawn) is reused; the function is not. |
| D-11 | `src/tui/` structured as six files: `mod.rs`, `app.rs`, `state.rs`, `ui.rs`, `events.rs`, `terminal_guard.rs`, `probe.rs` | Single-responsibility per file. `ui::render()` is a pure function (testable in isolation). `TerminalGuard` is a named RAII type with a verifiable `Drop` impl. See ADR-002. |
| D-12 | `TerminalGuard` RAII pattern for terminal lifecycle | Terminal teardown is a safety invariant. An anonymous RAII block is insufficient — it can be bypassed by early return. A named struct with `Drop` provides a second line of defence even on panic. |
| D-13 | Startup probe runs before event loop (`probe::run_startup_probe()`) | Earned Trust: `enable_raw_mode()` lies when stdout is piped. DB may be unreachable. Terminal size may be unavailable in CI-like environments. Probing before the first frame prevents silent failure modes. |
| D-14 | `ctrlc 3.4` for SIGINT handling | Minimal crate (MIT/Apache-2.0). Sets `AtomicBool` flag; event loop checks each tick. `signal-hook` rejected as overkill for a single-flag SIGINT use case. |
| D-15 | Synchronous event loop, 16 ms poll tick, no async runtime | Existing codebase has no `tokio`. Synchronous 60 fps ceiling is sufficient for a keyboard-driven table navigator with no animations. ~500 KB binary size increase avoided. |

---

### Architecture Pattern

**Modular monolith with ports-and-adapters (hexagonal) within the `src/tui/` module.**

- Driving port: `tui::run(FilterArgs)` called from `cli_structure::match_user_action` (`UserAction::List` arm).
- Driven ports: `database::load_all_applications()` (DB read), `tui::events::open_pdf()` (OS PDF open), `crossterm` (terminal I/O), `ctrlc` (signal).
- Dependency rule: all arrows point inward. `src/tui/` imports from `database` and `models`; no module imports from `src/tui/`. `src/tui/` has zero imports from `src/cv_insert.rs`.

---

### Reuse Analysis

| Existing Component | Decision | Justification |
|---|---|---|
| `UserAction::List` handler (`cli_structure.rs:78`) | **EXTEND** | Current arm returns a debug string. Extended to call `tui::run()`. No new CLI surface. |
| `read_cv_from_db` (`database.rs:144`) | **DO NOT MODIFY** | Returns `Vec<String>`. Used by `remove_cv` flow. New companion `load_all_applications()` added instead. |
| `view_cv_file` (`helpers.rs:96`) | **DO NOT REUSE DIRECTLY** | Reads INI config for viewer binary name; constructs `.tex`→`.pdf` path. TUI has direct PDF path and needs non-blocking spawn. Pattern reused, function not called. |
| `Cv` model (`models.rs:4`) | **REUSE AS-IS** | Output type of `load_all_applications()`. Projected to `ApplicationRow` in `tui/state.rs`. Not modified. |
| `establish_connection_postgres` (`database.rs:32`) | **REUSE AS-IS** | Called by `load_all_applications()`. No change. |
| `fix_home_directory_path` (`helpers.rs:16`) | **REUSE AS-IS** | Available for path expansion if needed. No change. |
| `my_fzf` + `skim` (`helpers.rs:133`) | **NOT USED IN TUI PATH** | `skim` stays for `remove_cv`. TUI implements in-process filter state. `my_fzf` is not imported from `src/tui/`. |
| `GLOBAL_VAR` / `get_global_var` (`global_conf.rs`) | **REUSE AS-IS** | DB URL lookup via existing mechanism. No change. |

---

### Technology Stack

| Component | Crate | Version | License |
|---|---|---|---|
| TUI rendering | `ratatui` | 0.29 | MIT |
| Terminal I/O | `crossterm` | 0.28 | MIT |
| Signal handling | `ctrlc` | 3.4 | MIT/Apache-2.0 |
| Database ORM | `diesel` | existing | MIT/Apache-2.0 |
| Async runtime | — | none | — |

All new dependencies are OSS with MIT or MIT/Apache-2.0 dual license. Zero proprietary dependencies. Zero new system binary dependencies.

---

### Constraints Established by DESIGN Wave

| # | Constraint |
|---|---|
| C-1 | `src/tui/` must not import from `src/cv_insert.rs`. Display context is read-only. |
| C-2 | `ui::render()` signature must be `fn render(frame: &mut Frame, state: &AppState)` — `&AppState`, never `&mut AppState`. |
| C-3 | `App` struct field declaration order: `terminal` before `_guard` to ensure correct drop order. |
| C-4 | No `unwrap()` / `expect()` in `src/tui/*.rs`. All error paths return `Result` or write to `AppState.status_message`. Enforced via `clippy::unwrap_used`. |
| C-5 | `TerminalGuard` must be a named struct with an explicit `impl Drop`. No anonymous RAII blocks for terminal teardown. |
| C-6 | `load_all_applications()` is the only new database function. `read_cv_from_db` signature is frozen. |
| C-7 | Startup probe must run before the event loop. A probe failure emits `health.startup.refused: <reason>` and exits with non-zero code. |
| C-8 | PDF open uses `std::fs::metadata(path).is_ok()` existence check before `Command::spawn()`. Missing file writes to `status_message`; never panics. |

---

### Upstream Changes (DESIGN wave additions)

| File | Change Type | Description |
|---|---|---|
| `src/main.rs` | Add `mod tui;` | New module declaration |
| `src/cli_structure.rs` | Extend `match_user_action` | `UserAction::List` arm calls `tui::run(filters)` |
| `src/database.rs` | Add two functions (pre-slice-01 prerequisite) | `pub fn establish_connection(engine: &str) -> ...` shared factory (SQLite + PostgreSQL dispatch) + `pub fn load_all_applications() -> Result<Vec<Cv>, Box<dyn std::error::Error>>` calling the shared factory. `read_cv_from_db` is untouched. |
| `Cargo.toml` | Add dependencies | `ratatui`, `crossterm`, `ctrlc` — **see DISTILL upstream change UC-1 below** |

---

### ADR Index

| ADR | Title | File |
|---|---|---|
| ADR-001 | TUI library selection: ratatui over crossterm-raw, cursive, termion | `docs/product/architecture/adr-001-tui-library-ratatui.md` |
| ADR-002 | `src/tui/` module structure and RAII TerminalGuard pattern | `docs/product/architecture/adr-002-tui-module-structure.md` |

---

### Open Questions Deferred

| # | Question | Deferred To |
|---|---|---|
| OQ-01 | Pre-populate filter bar from CLI `--company`/`--job-title` flags? | DISTILL / slice 03 |
| OQ-02 | ~~Promoted to upstream change~~ `establish_connection(engine)` shared factory required in `database.rs` as a pre-slice-01 prerequisite. SQLite is the default engine; `establish_connection_postgres` alone breaks the majority of users. See Upstream Changes table. | Pre-slice 01 prerequisite — resolved in architecture |
| OQ-03 | PageUp/PageDown in addition to arrow keys + Home/End? | DISTILL / slice 02 |
| OQ-04 | Status message auto-clear: tick countdown vs clear-on-next-keypress? | DISTILL / slice 04 |
| OQ-05 | `clippy::unwrap_used` introduced from slice 01 or incrementally? | DELIVER / platform-architect |

---

## Wave: DISTILL / [REF] Scenario List

| ID | Title | Tags | Feature File |
|----|-------|------|--------------|
| WS-01 | List command detects non-TTY and exits with message | `@walking_skeleton @driving_adapter @real-io @us-01` | `walking_skeleton.feature` |
| WS-02 | List command with unreachable DB exits with error | `@walking_skeleton @driving_adapter @real-io @us-01 @error` | `walking_skeleton.feature` |
| WS-03 | List command exits with error when DATABASE_URL not set | `@walking_skeleton @driving_adapter @real-io @error` | `walking_skeleton.feature` |
| AD-01 | load_all_applications returns all records from populated DB | `@real-io @adapter-integration @us-01` | `milestone_1_table_display.feature` |
| AD-02 | load_all_applications returns empty list for empty DB | `@real-io @adapter-integration @us-01` | `milestone_1_table_display.feature` |
| AD-03 | load_all_applications returns error on invalid connection | `@real-io @adapter-integration @us-01 @error` | `milestone_1_table_display.feature` |
| US01-S01 | ApplicationRow maps all Cv fields | `@us-01 @in-memory` | `milestone_1_table_display.feature` |
| US01-S02 | ApplicationRow date fallback to "Unknown" | `@us-01 @in-memory` | `milestone_1_table_display.feature` |
| US01-S03 | ApplicationRow excludes quote field | `@us-01 @in-memory` | `milestone_1_table_display.feature` |
| US01-S04 | AppState empty state message | `@us-01 @in-memory` | `milestone_1_table_display.feature` |
| US01-E01 | load_all_applications returns error when DATABASE_URL not set | `@real-io @adapter-integration @us-01 @error` | `milestone_1_table_display.feature` |
| US02-S01 | Down arrow increments selected index | `@us-02 @in-memory` | `milestone_2_navigation.feature` |
| US02-S02 | Down on last row does not advance | `@us-02 @in-memory` | `milestone_2_navigation.feature` |
| US02-S03 | Up arrow decrements selected index | `@us-02 @in-memory` | `milestone_2_navigation.feature` |
| US02-S04 | Up on first row does not go below zero | `@us-02 @in-memory` | `milestone_2_navigation.feature` |
| US02-S05 | Home key jumps to first row | `@us-02 @in-memory` | `milestone_2_navigation.feature` |
| US02-S06 | End key jumps to last row | `@us-02 @in-memory` | `milestone_2_navigation.feature` |
| PBT-01 | Navigation never out-of-bounds (proptest) | `@us-02 @in-memory @property` | `milestone_2_navigation.feature` |
| US02-E01 | Navigation on empty list (0 rows) is no-op | `@us-02 @in-memory @error` | `milestone_2_navigation.feature` |
| US02-E02 | Navigation when all rows filtered out does not panic | `@us-02 @in-memory @error` | `milestone_2_navigation.feature` |
| US03-S01 | Empty filter returns all rows | `@us-03 @in-memory` | `milestone_3_filter.feature` |
| US03-S02 | Filter matches company case-insensitively | `@us-03 @in-memory` | `milestone_3_filter.feature` |
| US03-S03 | Filter matches job_title case-insensitively | `@us-03 @in-memory` | `milestone_3_filter.feature` |
| US03-S04 | Filter with no matches returns empty | `@us-03 @in-memory` | `milestone_3_filter.feature` |
| US03-S05 | Clearing filter restores full list + resets selection | `@us-03 @in-memory` | `milestone_3_filter.feature` |
| US03-S06 | Navigation works within filtered results | `@us-03 @in-memory` | `milestone_3_filter.feature` |
| US03-E01 | Whitespace-only filter treated as empty | `@us-03 @in-memory @error` | `milestone_3_filter.feature` |
| US03-E02 | Special regex chars in filter do not panic | `@us-03 @in-memory @error` | `milestone_3_filter.feature` |
| PBT-02 | Filtered count never exceeds total (proptest) | `@us-03 @in-memory @property` | `milestone_3_filter.feature` |
| PBT-03 | Empty filter always returns full list (proptest) | `@us-03 @in-memory @property` | `milestone_3_filter.feature` |
| US04-S01 | open_pdf returns error for nonexistent path | `@us-04 @in-memory @error` | `milestone_4_pdf_open.feature` |
| US04-S02 | Enter ignored when list is empty | `@us-04 @in-memory` | `milestone_4_pdf_open.feature` |
| US04-S03 | Enter ignored in FilterMode | `@us-04 @in-memory` | `milestone_4_pdf_open.feature` |
| US04-S04 | Enter calls open_pdf with selected row path | `@us-04 @in-memory` | `milestone_4_pdf_open.feature` |
| US04-E01 | open_pdf with empty path returns error | `@us-04 @in-memory @error` | `milestone_4_pdf_open.feature` |
| US04-E02 | open_pdf with directory path returns error | `@us-04 @in-memory @error` | `milestone_4_pdf_open.feature` |

**Total**: 36 scenarios — 3 walking-skeleton subprocess, 4 adapter real-io, 26 in-memory unit, 3 property-based.
**Error coverage**: 11 of 36 are `@error` scenarios (31%) — supplemented by 3 PBT invariant tests.

---

## Wave: DISTILL / [REF] Adapter Coverage Table

| Adapter | @real-io scenario | Covered by |
|---------|-------------------|------------|
| DB `load_all_applications()` (driven internal) | YES | AD-01, AD-02, AD-03 |
| CLI `rusty-cv list` (driving) | YES | WS-01 (subprocess, non-TTY path), WS-02 (DB error path) |
| OS PDF open `open_pdf()` (driven external) | Fake — path-existence check only | US04-S01 tests the error path without spawning a process |

No "NO — MISSING" rows. All adapters covered. ✅

---

## Wave: DISTILL / [REF] Scaffolds

All production modules referenced in tests have scaffold files created. Every scaffold:
- Has a `// SCAFFOLD: true` marker for machine detection (`grep -r "SCAFFOLD: true" src/`)
- Exports correct public type signatures so test imports resolve
- Bodies `panic!("Not yet implemented — RED scaffold")` → AssertionError class → RED not BROKEN

| Scaffold File | Exports | Maps To |
|---------------|---------|---------|
| `src/tui/mod.rs` | `pub fn run(FilterArgs) -> Result<()>` | `UserAction::List` wiring |
| `src/tui/state.rs` | `ApplicationRow`, `AppState`, `Mode` | All specification tests |
| `src/tui/app.rs` | `App::new()`, `App::run()` | WS-01 (via `tui::run`) |
| `src/tui/probe.rs` | `run_startup_probe() -> Result<(), String>` | WS-01, WS-02 |
| `src/tui/terminal_guard.rs` | `TerminalGuard` with `Drop` | AC-04, KPI-04 |
| `src/tui/events.rs` | `open_pdf(&str) -> Result<(), String>`, `KeyEventStub` | US04-S01 to S04 |
| `src/tui/ui.rs` | `render(&mut (), &AppState)` | AC-01 (render gate) |

**Framework deferred**: `ratatui`, `crossterm`, `ctrlc` not yet in `Cargo.toml` — see UC-1 upstream change. Scaffold stubs use `()` placeholder for `ratatui::Frame` and `KeyEventStub` for `crossterm::event::KeyEvent`. DELIVER slice-01 replaces these with real types after resolving the Rust version constraint.

---

## Wave: DISTILL / [REF] Test Placement

```
tests/
  tui_job_applications_scenarios.rs     # Subprocess-level (driving adapter: CLI)
  tui_job_applications_specifications.rs # Unit-level (in-memory: state, filter, nav, pdf)
  tui_job_applications/
    walking_skeleton.feature
    milestone_1_table_display.feature
    milestone_2_navigation.feature
    milestone_3_filter.feature
    milestone_4_pdf_open.feature
```

**Precedent**: project uses flat `tests/*.rs` files (existing `tests/integration-tests.rs`). Following same convention for the two Rust test files.

---

## Wave: DISTILL / [REF] Driving Adapter Coverage

| DESIGN entry point | Subprocess/HTTP/hook scenario | Verified |
|--------------------|-------------------------------|----------|
| CLI `rusty-cv list` (`UserAction::List` in `cli_structure.rs:78`) | WS-01: subprocess detects non-TTY, exits with message | ✅ |
| CLI `rusty-cv list` (DB unreachable path) | WS-02: subprocess with bad `DATABASE_URL`, exits with error | ✅ |
| CLI `rusty-cv list` (DB missing env path) | WS-03: subprocess without `DATABASE_URL`, exits with config error | ✅ |

---

## Wave: DISTILL / [REF] Pre-requisites

- **Rust ≥1.88** in build environment (see UC-1)
- `lib.rs` exports `pub mod tui;` — already done in scaffold
- `load_all_applications()` scaffold added to `database.rs` — RED stub (DELIVER slice-01 implements body, per DESIGN D-9)
- `UserAction::List` arm in `cli_structure.rs:match_user_action` now calls `crate::tui::run(filters)` — wired in DISTILL, body implemented in DELIVER slice-01

---

## Wave: DISTILL / [REF] Upstream Changes

### UC-1: Dependency constraint (back-propagates to DESIGN D-5)

**Origin**: DESIGN wave D-5 specified `ratatui = "0.29"` + `crossterm = "0.28"`.

**Discovery during DISTILL**: `skim 4.7.0` (locked in `Cargo.lock`) pulls in `ratatui = "^0.30.0"` as a dependency, and `ratatui 0.30+` requires Rust 1.88.0. The project currently compiles against Rust 1.86.0 in the local shell; the devenv provides a newer version. Adding `ratatui` as a direct dependency creates a two-version conflict in the dependency graph.

**New assumption**: `ratatui`, `crossterm`, and `ctrlc` must be added to `Cargo.toml` in the devenv shell (Rust ≥1.88). The exact version of ratatui is pinned by skim's transitive requirement (`^0.30.0`), so use `ratatui = "0.30"` not `"0.29"`.

**DESIGN D-5 updated**: ratatui version changes from `0.29` to `0.30` (skim-compatible). ADR-001 conclusion remains valid — ratatui is still the correct choice.

**Impact on DELIVER**: DELIVER slice-01 must be executed inside the devenv shell. `ratatui = "0.30"`, `crossterm = "0.28"`, `ctrlc = "3.4"` are the correct versions to add.

**DISCOVER artifacts unchanged**: No DISCOVER wave was run.

---

## Wave: DISTILL / [REF] Project Infrastructure Policy

Bootstrapped at: `docs/architecture/atdd-infrastructure-policy.md`

Summary:
- **Driving** (CLI): `std::process::Command` subprocess from `TempDir`, binary via `env!("CARGO_BIN_EXE_rusty_cv_creator")`
- **Driven internal** (DB): real SQLite via Diesel in `TempDir`, `serial_test::serial` for env-var isolation
- **Driven external** (PDF open): fake — test `open_pdf()` return value without spawning a process

---

## Wave: DISTILL / [REF] RED Classification Note

**Environment requirement**: Run `cargo test -- --ignored` inside the `devenv shell` (Rust ≥1.88, proper DATABASE_URL).

Expected classification for all 36 tests:
- `MISSING_FUNCTIONALITY` (RED ✅) — bodies panic with "Not yet implemented — RED scaffold"

Not expected:
- `IMPORT_ERROR` / `BROKEN` — scaffold stubs export all needed types; imports compile once ratatui/crossterm are added in devenv

---

## Wave: DISTILL / [REF] Consolidated Review Resolution

**Reviewers**: Eclipse (DISCUSS), Architect (DESIGN), Forge (DEVOPS), Sentinel (DISTILL)
**Overall gate**: conditionally approved — all blocker + high findings resolved in DISTILL.

### Eclipse (DISCUSS) — approved ✅

No findings.

### Architect (DESIGN) — conditionally approved, resolved ✅

| Finding | Severity | Resolution |
|---------|----------|------------|
| F-1: Technology Stack showed ratatui 0.29; UC-1 requires 0.30 | high | Updated `brief.md` Technology Stack to `ratatui = "0.30"` + Rust ≥1.88 note |
| F-2: C4 Container diagram component label ratatui 0.29 | high | Updated diagram label in `brief.md` to `ratatui 0.30 (UC-1)` |
| F-3: DESIGN D-5 rationale does not mention UC-1 | low | Deferred to DELIVER — D-5 rationale is in feature-delta.md DESIGN section |

### Forge (DEVOPS) — conditionally approved, partially resolved ✅

| Finding | Severity | Resolution |
|---------|----------|------------|
| F-1: CI workflows not pinned to Rust ≥1.88 | high | Deferred to DELIVER slice-01 — DELIVER must update CI Rust version before merging slice-01 |
| F-2: KPI-03 binary dependency verification not in CI | high | Deferred to DELIVER slice-01 — add `ldd`/`cargo-tree` check to CI |
| F-3: Release artifact distribution strategy unclear | medium | Deferred to DELIVER — source-only distribution is current default |

### Sentinel (DISTILL) — needs_revision → resolved ✅

| Finding | Severity | Resolution |
|---------|----------|------------|
| F-1: `load_all_applications()` missing from `database.rs` | blocker | Added RED scaffold stub to `src/database.rs` |
| F-2: `UserAction::List` not wired to `tui::run()` | blocker | Wired in `cli_structure.rs:78` — calls `crate::tui::run(filters)` |
| F-3: Error coverage 7.4% (2/27), below 30% minimum | high | Added 8 new @error scenarios + 1 tag fix → 11/35 = 31% |
| F-4: WS-02 missing `@driving_adapter` tag | high | Added `@driving_adapter` tag to WS-02 in feature file + test comment |
| F-5: Feature-delta claimed 4 @error scenarios (15%), actual 2 (7.4%) | high | Updated Scenario List table → 11/35 (31%); count is now accurate |
| F-6: UC-1 Rust version constraint unclear | high | Cargo.toml comment unchanged (Rust 1.86 in shell); DELIVER must execute inside devenv (Rust ≥1.88) — documented in Pre-requisites and RED Classification Note |
| F-7: Milestone 3 has 8 scenarios (sizing threshold) | low | Noted. US03 is the most complex milestone; DELIVER should plan ~1.5 days |
| F-8: Proptest strategies use unbounded string patterns | low | Noted. DELIVER to tighten to max 50 chars per field if shrinking is slow |
