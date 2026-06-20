# DISTILL Wave Decisions — cv-variant-build

> RETROACTIVE backfill (LEAN, Tier-1 [REF] only). The implementation is already
> shipped and GREEN (branch `feature/change-generation`, v4.0.2): 80 tests pass
> under `cargo nextest run`, line coverage ~84%. This wave authors `.feature`
> documentation SSOT mapped to the EXISTING Rust tests. No source code modified,
> no dependencies added, no RED scaffolds created. interaction: non-interactive.

## Reconciliation HARD GATE

Read DISCUSS (`discuss/wave-decisions.md` RD-1..RD-5) and DESIGN
(`design/wave-decisions.md` D-1..D-5 / ADR-0001..0005). DISCUSS uses the `RD-n`
namespace; DESIGN uses `D-n`/`ADR-000n`. Each requirement maps cleanly onto a
design decision (RD-1↔D-1, RD-2↔`CV_VARIANTS`, RD-3↔`infer_variant_from_job_title`,
RD-4/RD-5↔ADR-0002/0003 + filesystem). No DEVOPS directory exists (sensible
defaults applied, not a blocker).

**Reconciliation passed — 0 contradictions.**

## Language + Policy + Port Bootstrap (Phase 0)

- `[lang-mode] rust` — detected from `Cargo.toml`.
- `[policy-mode] write` — `docs/architecture/atdd-infrastructure-policy.md`
  created (was absent); 3 tables populated for the in-scope ports.
- `[port-mode] n/a` — the Universe/state-delta port (`tests/common/state_delta.<ext>`)
  is **not** bootstrapped. Rationale: this is a documentation-only backfill over
  shipped GREEN code with no executable BDD harness and no PBT machinery; adding a
  state-delta port would be unused infrastructure. Mandate 8 is documented as N/A
  here (see Mandate compliance below).

## Two-Tier Acceptance Decision

**Tier A only.** Justification: the feature is config/CLI-shaped (single-shot
`insert` run: resolve → build → file → optional persist/view). Variant resolution
is a pure decision function; persistence is exercised against in-memory SQLite.
No journey of ≥3 chained scenarios with a domain-rich generative input space —
the Tier B trigger (Mandate 10) is not met. Tier B (`RuleBasedStateMachine`) is
correctly skipped; it would also require `proptest`, which is intentionally not a
dependency.

## Mandate-7 RED Scaffolding — N/A

The implementation already exists and is GREEN. Mandate 7 (RED-ready scaffolds)
is **not applicable** to a retroactive backfill over shipped code. No production
modules were stubbed and no `__SCAFFOLD__`/`SCAFFOLD: true` markers were added.

### Red-classification statement

All scenarios authored in this wave map to tests that are **already GREEN** under
`cargo nextest run` (80 passing, ~84% line coverage). There is no RED set to
classify: the pre-DELIVER fail-for-the-right-reason gate is satisfied vacuously
because DELIVER has already happened. Each mapping is verified against the
concrete test function in the traceability table (feature-delta, Wave: DISTILL).

## Mandate-12 Step-Reuse Ratio — N/A

There is no BDD step-definition harness (no cucumber-rust); the `.feature` files
are documentation SSOT, not executable step-bound specs. The step-reuse-ratio
metric (`total_step_invocations / unique_step_decorators`) is therefore
**N/A** — there are zero step decorators to measure. Domain types (criterion 1)
already live in production code as Rust types (`CV_VARIANTS`, `BuildConfig`,
`DbConnection`, `FilterArgs`); no separate `domain_types` test module is
warranted for a documentation backfill.

## Self-Completeness Audit (Phase 2.5)

`nw-at-completeness-check` 15-item checklist computed over the 32 scenarios
(see feature-delta DISTILL § Self-Completeness Audit for the per-item table).
Verdict: **ACCEPTABLE_WITH_DOCUMENTED_GAPS** (11/15). Gaps are
`AT_GAP_IN_DELIVERY_SCOPE` documented and bounded to shipped behaviour (no
upstream `SPECIFICATION_AMBIGUITY` blockers). Notable documented gaps: no
subprocess-level CLI driving-adapter test; no test for the "save flag omitted →
no DB write" path; C7 interruption/concurrency not applicable (single-user
single-shot CLI).

## Density

**LEAN** — Tier-1 [REF] sections only. No Tier-2 expansions rendered.

## Upstream Changes

None. DISCUSS/DESIGN [REF] sections and ADR-0001..0005 were read and reconciled,
not modified. The 4-reviewer consolidated gate is intentionally skipped by the
parent for this shipped-code backfill; a self-review checklist is recorded in the
feature-delta DISTILL section.
