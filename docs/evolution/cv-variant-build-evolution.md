# Evolution Record — cv-variant-build

> Long-term archive / finalize record for the `cv-variant-build` feature.
> **Retroactive DELIVER backfill (DES-EXEMPT)** — the feature is already
> implemented, tested, and merged (branch `feature/change-generation`, v4.0.2).
> No source code, no DES execution-log, and no TDD cycle were produced for this
> record. Density: **LEAN**. Finalized: 2026-06-20.

## What Shipped

A variant-aware CV build pipeline for the single-user `rusty_cv_creator` CLI:
select one of 4 role variants (`senior-devops`, `senior-platform-engineer`,
`senior-sre`, `engineering-manager`) explicitly via `--variant` or by inference
from the job title (manager-family titles win), build it via a config-driven
`just build <variant>` (tectonic) in a dated working copy, file the produced PDF
under a per-year output dir with a deterministic `<date>-<job>-<company>.pdf`
name, clean up the working dir, and optionally persist the application
(`--save-to-database`) or preview it (`--view-generated-cv`). Persistence is
backend-agnostic (Postgres prod / SQLite tests) and all subprocess effects flow
through an injectable `CommandRunner` port.

## The 6 Commits (oldest → newest)

| # | SHA | Commit | Role |
|---|-----|--------|------|
| 1 | `865780d` | chore(env): update devenv configuration | Toolchain scaffolding (just/tectonic/zathura/tailscale) — enabler, not a feature step. |
| 2 | `6472189` | refactor(db): backend-agnostic queries via diesel MultiConnection | **Step 01-01** — `DbConnection` persistence port. |
| 3 | `23fde25` | feat(command-runner): add CommandRunner seam for testable subprocesses | **Step 01-02** — subprocess port. |
| 4 | `beb5034` | feat(cv): build variant via Justfile, configurable output, tool checks | **Step 01-03** — variant build subsystem (the feature core). |
| 5 | `d34990e` | chore(coverage): use llvm-cov coverage(off) exclusions, ignore schema.rs | **Step 01-04** — coverage discipline. |
| 6 | `c3807a7` | docs(nwave): backfill DESIGN wave for cv-variant-build | Documentation commit (brief/ADRs/c4) — not feature delivery. |

Non-feature: `e5d95af` chore(tooling) (claude session logs + nix-store symlinks) — droppable.

## Key Decisions (ADR links)

- [ADR-0001](../product/architecture/adr-0001.md) — Variant-based CV build via
  Justfile (replaces xelatex + `BLANKPOSITION` placeholder). [D-1 / RD-1]
- [ADR-0002](../product/architecture/adr-0002.md) — `CommandRunner` port for all
  subprocess side-effects. [D-2]
- [ADR-0003](../product/architecture/adr-0003.md) — diesel `MultiConnection` for
  backend-agnostic persistence. [D-3 / RD-4]
- [ADR-0004](../product/architecture/adr-0004.md) — Pre-usage PATH tool checks
  with a devenv hint. [D-4]
- [ADR-0005](../product/architecture/adr-0005.md) — Coverage discipline via test
  seams + `coverage_nightly` gating. [D-5]

## Measured Outcomes

- **Line coverage**: 54% → **84%** (`cargo-llvm-cov`).
- **Test count**: **80/80** GREEN under `cargo nextest run` (process-per-test
  determinism for the `GLOBAL_VAR` `OnceCell`).
- **Acceptance coverage**: 32/32 DISTILL scenarios mapped to GREEN Rust tests
  across 4 documentation `.feature` files; all 5 user stories covered.
- **Error/edge scenario ratio**: 59% (19/32).
- **Quality gates**: clippy `-D warnings` + pedantic clean.
- **Demo evidence** (2026-06-20): `--help` exit 0, `insert --help` exit 0
  (driving-port entry points present; `--variant` exposed).

## Known Gaps Carried Forward

- **No subprocess-level CLI test** — the binary is never spawned to assert exit
  code + stdout; coverage is at the `prepare_cv` orchestration layer
  (`test_prepare_cv_end_to_end_with_fake_builder`).
- **No test for `--save-to-database` omitted path** (`CV NOT SAVED TO DATABASE!`
  no-write branch).
- **`GLOBAL_VAR` `OnceCell` flakiness** under threaded `cargo test` (deterministic
  only under `cargo-nextest`) — process-global mutable config smell.
- **`parse_date` dead code** (`cli_structure.rs`, `#[allow(dead_code)]`) — built
  for filter parsing, not yet wired.
- **`list` / `update` / DB filtering partial** — `read_cv_from_db` / `show_cvs`
  accept `FilterArgs` but apply only `limit(50)` (`// TODO filters on proper DB`);
  `update` arm is a stub.
- **No real-IO contract coverage** for the external template repo, Postgres,
  viewer, or tailscale (all faked / in-memory in tests).

## Follow-up Recommendations

1. Add a **template-contract smoke test** in CI: run `just build <variant>` for
   each of the 4 variants and assert the expected PDF basename (verifies the
   recipe/output contract `compile_cv` assumes). See brief External Integrations.
2. Add a **subprocess CLI driving-adapter test** (spawn the built binary, assert
   exit code + stdout) to close the documented driving-port gap.
3. Add the missing **no-DB-write** unit test for the `--save-to-database` omitted
   path.
4. Refactor `GLOBAL_VAR` `OnceCell` → an injected `Config` value threaded through
   use cases (removes the nextest-only determinism constraint).
5. Wire `parse_date` and real DB filtering, or remove the dead code; complete or
   descope the `update`/`list` arms.

## Retrospective (LEAN)

- **What went well**: ports (`CommandRunner`, `DbConnection`) made the build and
  persistence flows testable in-memory, driving coverage 54%→84% without real
  external dependencies; variant resolution kept as a pure function.
- **What to improve**: end-to-end realism is faked everywhere — the highest-risk
  boundary (the external template repo's Justfile contract) has no automated
  verification; address via recommendation 1.
- **Process note**: this feature was delivered ahead of nWave adoption; DISCUSS/
  DESIGN/DISTILL/DELIVER artifacts are all retroactive backfills reconciled to the
  shipped code, not forward-planned. Future features should run the waves
  forward to avoid documentation/implementation drift.

## Handoff to Operations

- **Runbook**: single-user local CLI; no service to operate. "Deployment" =
  `cargo build --release` + devenv-provisioned tools on PATH.
- **Rollback**: revert to a prior tag (`git checkout <tag>` / reinstall prior
  binary); no migrations beyond diesel schema (SQLite/Postgres `cv` table).
- **Monitoring**: N/A (interactive CLI). Operational health = the pre-usage
  `ensure_tools_available` probe and `tailscale status` reachability check.
