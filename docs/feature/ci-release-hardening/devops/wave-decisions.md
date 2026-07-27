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
| 8 | `release.yml` lands its version bump through a bot-opened, bot-merged pull request instead of pushing to `master`; the tag is created only after that PR merges. | The `branch-discipline` ruleset requires a PR on `master` with no bypass actor, so the direct push failed with GH013 and left the version stuck at 5.3.0 plus an orphan `v5.3.1` tag. Making the automation compliant beats weakening the ruleset. | L66 |
| 9 | Delete the orphan `v5.3.1` tag on the remote before the first run of the new flow. | The version lookup walks reachable tags only, so the unreachable `v5.3.1` is invisible: the run computes 5.3.1 again from `v5.3.0`, and the release API silently attaches that release to the pre-existing orphan tag at `63f7d0b` instead of the merged bump commit. Every later run recomputes the same collision, so the pipeline stays stuck until the tag is gone. | L66 |

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
(conventional-changelog-action) on `master` push. Since decision 8 the workflow
writes nothing to `master` directly: it opens a `release-bot/<run-id>-<attempt>` PR with
the bump, merges it with the built-in `GITHUB_TOKEN`, then tags the bump commit
the merge brought in. No canary/blue-green (no service). **Rollback = re-tag-forward**
(preferred), or mark the bad GitHub Release as pre-release, or `cargo yank` if
crates.io publishing is ever wired.

## Stakeholder / Sign-off

Single maintainer (`franci@piva.online`). Demo evidence = green CI on the
`feature/change-generation` PR after apply: clippy-deny passes, rustfmt passes,
threaded `cargo test` passes (proving ADR-0006 holds), and exactly one release
mechanism remains. Sign-off = PR merge to `master`.

## Constraints

- DOCS-ONLY this wave; no `.yml`/`.releaserc*`/source edits until apply step.
- Behavior-preserving for the release pipeline (deletions target unreferenced
  files). Superseded for `release.yml` by decision 8, which deliberately changes
  how the bump reaches `master` - the ruleset left the old behavior unusable.
- No new dependencies, no new platform, no container/orchestration.
- GitHub Flow triggers unchanged.

## Apply-step targets (for the orchestrator)

| File | Action | Changes |
|------|--------|---------|
| `.github/workflows/rust-tests.yml` | EDIT | CHANGE 1 (clippy `-D warnings` step + advisory pedantic), CHANGE 2 (uncomment/enable `rustfmt` job), CHANGE 3 (add `threaded-test` job) |
| `.releaserc.yml` | DELETE | CHANGE 4 (dormant semantic-release config) |
| `.releaserc` | DELETE | CHANGE 4 (dormant semantic-release branches) |
| `.github/workflows/release.yml` | EDIT | Decision 8 (bump lands via a bot pull request; tag created after the merge) |

## Upstream Changes

None to source/domain/PRD. New: ADR-0007. `brief.md` gets a one-line CI/release
pointer (no duplication).
