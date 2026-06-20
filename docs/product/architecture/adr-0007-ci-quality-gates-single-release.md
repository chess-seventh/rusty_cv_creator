# ADR-0007: CI quality gates + single release mechanism

## Status
Proposed (DEVOPS wave, feature `ci-release-hardening`, PROPOSE mode, DOCS-ONLY —
the apply step edits `.github/workflows/rust-tests.yml` and deletes `.releaserc.yml`
+ `.releaserc` after approval).

Builds on [ADR-0006](adr-0006-inject-appcontext.md) (the `GLOBAL_VAR` →
`&AppContext` determinism fix): this ADR turns that fix into a permanent CI gate.

## Context

The CI suite has a real bar locally (devenv `prepare`: `cargo clippy -- -D
warnings`, `treefmt`, threaded `cargo test`) that the *remote* pipeline does not
enforce. Concretely, before this change:

- **clippy is advisory only.** `rust-tests.yml`'s `clippy` job runs
  `cargo clippy --all-targets -- -W clippy::pedantic` — `-W` warns but never
  fails, so lint regressions merge silently.
- **No formatting gate.** The `rustfmt` job (`cargo fmt --all -- --check`) is
  commented out. Formatting drift is unguarded in CI.
- **Only nextest runs.** The `unittest` job runs `cargo nextest run`
  (process-per-test). This execution model structurally **cannot** catch the
  order-dependent, shared-state flakiness that ADR-0006 just removed — a plain
  threaded `cargo test` is the test that would have caught the `GLOBAL_VAR`
  `OnceCell` bug (3 failing tests pre-refactor). CI has no threaded run.
- **Dual, conflicting release config.** `release.yml`
  (TriPSs/conventional-changelog-action) is the *active* mechanism and creates
  GitHub Releases on `master` push. Separately, `.releaserc.yml` describes a full
  semantic-release pipeline (commit-analyzer → changelog → `semantic-release-rust`
  publish to crates.io → git → github) and `.releaserc` lists branches
  `master,next` — but **no workflow invokes `semantic-release`**, so it is
  **dormant dead config** that misrepresents what the project actually does
  (it implies crates.io publishing that never happens).

This is internal infrastructure: no new typed user-facing contract, so the
Outcome Collision Check is **N/A** (and the local `nwave-ai outcomes check-delta`
is non-functional in this install — missing `jsonschema`, no
`outcomes/registry.yaml`).

## Decision

Four targeted changes (exact YAML in
[`../../feature/ci-release-hardening/feature-delta.md`](../../feature/ci-release-hardening/feature-delta.md#wave-devops--how-proposed-ci-diffs)):

1. **Enforce clippy `-D warnings` as a FAILING gate.** Add a blocking
   `cargo clippy --all-targets -- -D warnings` step to the `clippy` job; keep
   `-W clippy::pedantic` as a *separate, non-failing* (`continue-on-error: true`)
   advisory step. Mirrors the devenv `prepare` script.
2. **Re-enable the formatting gate.** Uncomment the `rustfmt` job running
   `cargo fmt --all -- --check` via `dtolnay/rust-toolchain@stable` (no nix in CI).
3. **Add a threaded `cargo test` job** (`threaded-test`, stable toolchain, plain
   `cargo test`) to regression-guard the ADR-0006 determinism fix. Runs alongside
   the existing nextest job so both execution models are covered.
4. **Resolve the dual release config by DELETING** `.releaserc.yml` + `.releaserc`,
   leaving `release.yml` (conventional-changelog-action) as the single mechanism.

Triggers stay `push: [master]` + `pull_request: [master]` (correct for GitHub
Flow). Mutation testing is intentionally **not** added — `nightly-delta` is the
intended future strategy but no tooling exists yet; `CLAUDE.md` is left unedited.

## Alternatives considered

**Contested point A — formatting gate: `cargo fmt --check` vs treefmt-in-CI.**
- *`cargo fmt --all -- --check` (chosen).* Canonical Rust gate, zero setup,
  no nix runtime in CI, identical to the long-commented-out job. Con: diverges
  from local `treefmt`, which runs `rustfmt --edition 2024 --config
  skip_children=true` (CI uses `Cargo.toml` edition 2021 and descends children),
  so the two can disagree on edge cases; treefmt also covers `*.nix`/`*.toml`/`*.yml`
  that `cargo fmt` ignores.
- *treefmt-in-CI (`treefmt --ci`) (rejected).* Would make local and remote
  byte-identical and cover all file types. Rejected: requires provisioning nix /
  the treefmt toolchain (nixfmt, toml-sort, yamlfmt) in CI — heavy for a single
  Rust gate, slower, more moving parts. Revisit only if rustfmt drift actually
  bites; the cheaper fix is to align `treefmt.toml` to edition 2021 / drop
  `skip_children`.

**Contested point B — release config: remove vs wire semantic-release.**
- *Delete `.releaserc.yml` + `.releaserc` (chosen).* `release.yml` already works
  and is in active use; deleting unreferenced config is behavior-preserving and
  lowest risk. Con: abandons the (never-used) crates.io publish path.
- *Wire semantic-release + drop `release.yml` (rejected).* Add a workflow running
  `npx semantic-release` and remove `release.yml`. Would enable crates.io
  publishing via `semantic-release-rust`. Rejected: needs a Node toolchain +
  `NPM_TOKEN`/registry-token secrets + a tool not installed; crates.io
  distribution is an unrequested new capability for a single-user CLI; swapping a
  working mechanism is higher-risk than deleting dead config.

**Contested point C — threaded test: add `cargo test` vs keep nextest-only.**
- *Add `threaded-test` (chosen).* Directly guards ADR-0006; cheap (one job,
  cached toolchain). Con: ~duplicate test execution time. Accepted — it covers a
  failure mode nextest cannot.
- *Keep nextest-only (rejected).* Process-per-test masks shared-state /
  order-dependence regressions — exactly the class ADR-0006 fixed. Leaving it
  unguarded invites silent reintroduction.

## Consequences

- **Positive — the bar becomes real from this PR forward.** Lint, format, and
  threaded-determinism regressions now block merge into `master` instead of
  landing silently. Local devenv and remote CI agree on clippy `-D warnings`.
- **Positive — ADR-0006 is permanently guarded.** A reintroduced global / order
  dependence fails `threaded-test` even when nextest stays green.
- **Positive — one release truth.** Deleting dormant `.releaserc*` removes
  misleading config; `release.yml` is unambiguously the mechanism.
- **Negative — stricter PRs.** Existing latent clippy/format issues (if any) must
  be fixed before the next merge; CI time grows by ~one job. Mitigated by caching.
- **Neutral — no new dependencies, no new platform, no source change.** Mutation
  testing remains future work (`nightly-delta`, not wired).
- **Follow-up (out of scope, recommended):** enable GitHub branch protection
  requiring `build`, `unittest`, `threaded-test`, `clippy`, `rustfmt`,
  `cleanup-deps` status checks before merge to `master`.
