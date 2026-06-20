# Wave Decisions — `ci-release-hardening` (DEVOPS)

> PROPOSE mode, SYSTEM/infrastructure scope, LEAN. **DOCS-ONLY** — produces design
> + exact proposed YAML diffs; the orchestrator applies them after approval.
> Pre-requisite: ADR-0006 determinism fix (delivered `5214f33`).
> Outcome Collision Check: **N/A** (infra; no registry; `jsonschema` missing).

## Key Decisions

| # | Decision | Rationale | Ref |
|---|----------|-----------|-----|
| 1 | Add a blocking `cargo clippy --all-targets -- -D warnings` step; keep pedantic as a separate `continue-on-error: true` advisory step. | clippy is currently advisory-only (`-W`); mirror the devenv `prepare` bar in CI. | ADR-0007 / CHANGE 1 |
| 2 | Re-enable the commented-out `rustfmt` job (`cargo fmt --all -- --check`, stable toolchain). | No CI format gate today; `cargo fmt --check` is the nix-free canonical Rust gate. | ADR-0007 / CHANGE 2 |
| 3 | Add a `threaded-test` job running plain `cargo test` (stable). | CI runs only nextest (process-per-test) which can't catch the order-dependent flakiness ADR-0006 fixed; threaded `cargo test` can. | ADR-0007 / CHANGE 3 |
| 4 | DELETE dormant `.releaserc.yml` + `.releaserc`; keep `release.yml` as the single release mechanism. | Dead config no workflow invokes; misrepresents (non-existent) crates.io publishing. Removal is lower-risk than wiring semantic-release. | ADR-0007 / CHANGE 4 |
| 5 | Keep triggers `push`+`pull_request` to `master`. | Correct for GitHub Flow; no change needed. | feature-delta |
| 6 | Do NOT add mutation testing; record `nightly-delta` as intended/not-yet-wired; do NOT edit `CLAUDE.md`. | No tooling exists; out of scope this pass. | feature-delta |
| 7 | CI uses `cargo fmt --check`, not `treefmt`; accept treefmt as a local superset. | Avoid provisioning nix in CI; revisit only on rustfmt edition/skip_children drift. | ADR-0007 alt A |

## Production Readiness Summary

This is artifact-release infrastructure, not a running service, so the
production-readiness checklist is applied in its CI-relevant subset:

- [x] All acceptance/unit tests passing — 85/85 under both nextest and threaded `cargo test` (ADR-0006).
- [x] Quality gates enforced after apply — build, nextest, threaded test, clippy-deny, rustfmt, cargo-shear all blocking.
- [x] Coverage collected — llvm-cov → Codecov (advisory, `continue-on-error`).
- [x] Rollback documented — re-tag-forward / mark release pre-release / `cargo yank` (if ever published).
- [x] Logging — `env_logger` structured-ish (single-user CLI scope).
- [~] Monitoring/alerting — N/A (no running service); external observability deferred.
- [x] Hermetic CI — no Postgres/tectonic/tailscale/just/zathura; SQLite + FakeRunner seams.
- [x] Single release mechanism — `release.yml` after CHANGE 4.

## Deployment Strategy

Recreate / N-A — GitHub Release artifact via `release.yml`
(conventional-changelog-action) on `master` push. No canary/blue-green (no
service). **Rollback = re-tag-forward** (preferred), or mark the bad GitHub
Release as pre-release, or `cargo yank` if crates.io publishing is ever wired.

## Stakeholder / Sign-off

Single maintainer (`franci@piva.online`). Demo evidence = green CI on the
`feature/change-generation` PR after apply: clippy-deny passes, rustfmt passes,
threaded `cargo test` passes (proving ADR-0006 holds), and exactly one release
mechanism remains. Sign-off = PR merge to `master`.

## Constraints

- DOCS-ONLY this wave; no `.yml`/`.releaserc*`/source edits until apply step.
- Behavior-preserving for the release pipeline (deletions target unreferenced files).
- No new dependencies, no new platform, no container/orchestration.
- GitHub Flow triggers unchanged.

## Apply-step targets (for the orchestrator)

| File | Action | Changes |
|------|--------|---------|
| `.github/workflows/rust-tests.yml` | EDIT | CHANGE 1 (clippy `-D warnings` step + advisory pedantic), CHANGE 2 (uncomment/enable `rustfmt` job), CHANGE 3 (add `threaded-test` job) |
| `.releaserc.yml` | DELETE | CHANGE 4 (dormant semantic-release config) |
| `.releaserc` | DELETE | CHANGE 4 (dormant semantic-release branches) |

## Upstream Changes

None to source/domain/PRD. New: ADR-0007. `brief.md` gets a one-line CI/release
pointer (no duplication).
