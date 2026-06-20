# Feature Delta — `ci-release-hardening`

> DEVOPS wave, **PROPOSE** mode, **SYSTEM / infrastructure** scope. Density: **LEAN**.
> **DOCS-ONLY** — no `.github/workflows/*.yml`, `.releaserc*`, or source changed in
> this pass. This file specifies the design **and the exact proposed YAML diffs**;
> the orchestrator applies them after user approval.
> Outcome Collision Check: **N/A** (infra change, no new typed user-facing
> contract; the local `nwave-ai outcomes check-delta` is non-functional here —
> missing `jsonschema`, no `outcomes/registry.yaml`).
> Pre-requisite: ADR-0006 (the `GLOBAL_VAR` → `&AppContext` determinism fix,
> delivered `5214f33`) — this CI hardening makes that fix a permanent regression gate.

## Wave: DEVOPS / [REF] Goal

Harden the existing GitHub Actions CI/release suite before merging
`feature/change-generation` → `master`, so the project's *actual* local quality
bar (devenv: `cargo clippy -- -D warnings`, `treefmt`, threaded `cargo test`)
becomes an *enforced remote bar*, and the dual release configuration is collapsed
to a single, working mechanism. Four targeted changes; no new platform, no new
service, no new dependencies.

## Wave: DEVOPS / [REF] Decisions (recorded, not re-asked)

| Topic | Decision |
|-------|----------|
| Deployment target | GitHub Releases binary (no server / no containers) |
| Container orchestration | None |
| CI/CD platform | GitHub Actions (existing) |
| Existing infrastructure | Existing CI/CD only (extend, don't replace) |
| Observability | `env_logger` structured-ish logs; external stack deferred (single-user CLI) |
| Deployment strategy | Recreate / N-A (release artifacts, not a running service) |
| Continuous learning | No |
| Git branching | GitHub Flow (feature branch → PR → `master`) |
| Mutation testing | OUT OF SCOPE this pass — `nightly-delta` *intended, not yet wired* (future work). No mutation CI added; `CLAUDE.md` not edited. |

## Wave: DEVOPS / [REF] Environment matrix

| Environment | Runner / host | Toolchain | Purpose | Notes |
|-------------|---------------|-----------|---------|-------|
| CI — build/test/lint/fmt | GitHub Actions `ubuntu-latest` | `dtolnay/rust-toolchain@stable` (coverage job uses `@nightly` for `llvm-cov --doc`) | Commit-stage gates | **Hermetic** — no Postgres, no `tectonic`, no `tailscale`, no `just`/`zathura` needed. Tests use **SQLite + `FakeRunner`** via the ADR-0002 `CommandRunner` + ADR-0003 `DbConnection` seams. |
| CI — release | GitHub Actions `ubuntu-latest` | `@stable` + `cargo-edit` | Version bump + GitHub Release on `master` push | `release.yml` (TriPSs conventional-changelog-action). |
| Local dev | devenv shell | nightly channel (per `Cargo.toml`) | Authoring + pre-commit/pre-push gates | `prek`/pre-commit hooks, `treefmt`, the devenv `prepare` script already runs `cargo clippy -- -D warnings` and `cargo llvm-cov nextest`. |
| Production (runtime) | End-user machine (single user) | Released binary | Run the CLI | Postgres reached over Tailscale; `just`+`tectonic`+`zathura` resolved at runtime via the ADR-0004 pre-usage PATH checks. **Not exercised in CI.** |

Hermeticity is the load-bearing property: CI never provisions the heavy external
tools because every effect boundary is an injected seam with a test adapter.

## Wave: DEVOPS / [REF] CI/CD pipeline outline

GitHub Flow ⇒ triggers stay `push: [master]` + `pull_request: [master]` (correct
as-is — PR runs the full commit stage; merge to `master` runs it again + release).

| Workflow | Job | Trigger | Type after change | Change |
|----------|-----|---------|-------------------|--------|
| `build.yml` | `build` (`cargo build --verbose`) | push/PR master | blocking | unchanged |
| `rust-tests.yml` | `unittest` (`cargo nextest run`) | push/PR master | blocking | unchanged |
| `rust-tests.yml` | `coverage` (llvm-cov → Codecov) | push/PR master | advisory (`continue-on-error: true`) | unchanged |
| `rust-tests.yml` | `clippy` | push/PR master | **blocking (new `-D warnings` step)** + advisory pedantic | **CHANGE 1** |
| `rust-tests.yml` | `rustfmt` | push/PR master | **blocking (re-enabled)** | **CHANGE 2** |
| `rust-tests.yml` | `threaded-test` (`cargo test`) | push/PR master | **blocking (new job)** | **CHANGE 3** |
| `rust-tests.yml` | `cleanup-deps` (`cargo shear`) | push/PR master | blocking | unchanged |
| `release.yml` | `release` (conventional-changelog-action) | push master | release automation | **CHANGE 4 (becomes sole mechanism)** |

Commit stage stays well under the 10-minute target; jobs run in parallel (each is
independent; `Swatinem/rust-cache` warms the toolchain cache).

## Wave: DEVOPS / [REF] Deployment strategy

- **Mechanism:** `release.yml` on `master` push → conventional-changelog-action
  bumps `Cargo.toml`, writes `CHANGELOG.md`, commits `[skip ci]`, and creates a
  GitHub Release (`ncipollo/release-action`). Strategy = **Recreate / N-A** — the
  artifact is a tagged GitHub Release, not a rolling service; canary/blue-green do
  not apply.
- **Rollback:** (1) GitHub Release is immutable history — *re-tag forward* with a
  corrected `fix:`/`revert:` commit (preferred; conventional-changelog cuts a new
  patch). (2) Delete/mark-as-pre-release the bad GitHub Release. (3) If ever
  published to crates.io (not currently wired — see CHANGE 4), `cargo yank` the
  bad version. Roll-forward is the default because releases are idempotent and
  there is no running service to revert.
- **Risk:** low. No production service, single user, hermetic tests. Highest
  residual risk is a bad version bump; mitigated by re-tag-forward.

## Wave: DEVOPS / [REF] Mutation testing strategy

**Intended: `nightly-delta`. Status: NOT YET WIRED (future work).** No mutation
tooling (`cargo-mutants` / Stryker) exists in the repo today. Rationale for the
*intended* target: at this LOC size a per-feature run would be fast enough, but
the project's release cadence and single-maintainer velocity make a scheduled
nightly delta run the right cost/feedback trade once introduced. **Not added in
this pass; `CLAUDE.md` `## Mutation Testing Strategy` intentionally left unedited.**

## Wave: DEVOPS / [REF] Observability stack

`env_logger` + `log` (already a dependency) provide structured-ish, level-filtered
stderr logs — appropriate for a single-user CLI. No metrics/traces/SLO stack:
there is no long-running service to monitor, so RED/USE/Golden-Signals and an
external collector (Prometheus/Grafana/OTel) are **deferred** as over-engineering
for this scope. CI observability = the GitHub Actions run UI + Codecov (advisory).

## Wave: DEVOPS / [REF] Branching strategy

**GitHub Flow.** Short-lived feature branches (e.g. `feature/change-generation`)
→ PR → review → merge to `master`; releases cut from `master`. Current workflow
triggers (`push` + `pull_request` to `master`) are **correct** for this model and
need no change. After CHANGE 1–3, PRs into `master` get a real, blocking commit
stage (build + nextest + threaded test + clippy-deny + fmt + shear). Recommend
(out of this docs-only scope) adding GitHub branch-protection requiring these
status checks before merge.

## Wave: DEVOPS / [REF] Coexistence matrix

The remote gates must **mirror, not fight** the existing local toolchain.

| Local capability | Source | Must keep working | Interaction with this change |
|------------------|--------|-------------------|------------------------------|
| `prek` / pre-commit hooks | devenv git-hooks (`stages = ["pre-commit"]`) | Yes | CI clippy-deny (CHANGE 1) mirrors the local devenv `prepare` script's `cargo clippy -- -D warnings`. No conflict. |
| `treefmt` (rustfmt + nixfmt + toml-sort + yamlfmt) | `treefmt.toml`, pre-commit stage | Yes | **Trade-off flagged below** — CI uses `cargo fmt --check`, not treefmt. |
| `devenv` shell / scripts | `devenv.nix` | Yes | CI installs toolchain via `dtolnay/rust-toolchain@stable`; no nix in CI. Local devenv unaffected. |
| Threaded `cargo test` | devenv `prepare`/`pre-push` | Yes | CHANGE 3 promotes this exact command to a blocking CI job (ADR-0006 guard). |

**rustfmt trade-off (CHANGE 2):** `treefmt` runs `rustfmt --edition 2024
--config skip_children=true`, whereas `cargo fmt --all -- --check` uses the
**edition from `Cargo.toml` (2021)** and **does descend into child modules**.
These two can disagree on edge cases. Decision: CI uses **`cargo fmt --all --
--check`** (the canonical, zero-setup Rust gate — no nix runtime in CI) and we
accept that local `treefmt` is a superset (also covers `*.nix`/`*.toml`/`*.yml`).
If divergence ever bites, the fix is to align `treefmt.toml`'s rustfmt options to
edition 2021 / drop `skip_children`, or add a `treefmt --ci` job — see ADR-0007
alternatives. Recommendation: `cargo fmt --check` now; revisit only on drift.

## Wave: DEVOPS / [REF] Pre-requisites

- **ADR-0006 determinism fix delivered** (`5214f33`): `GLOBAL_VAR` `OnceCell`
  replaced by injected `&AppContext`; threaded `cargo test` now 85/85. CHANGE 3
  exists to *regression-guard* this — CI currently runs only `cargo nextest run`
  (process-per-test), which structurally cannot catch the order-dependent global
  flakiness that ADR-0006 eliminated.
- **ADR-0007** (this pass) records the four gate/release decisions.
- Existing 85-test suite is the behavioral net; these changes add *enforcement*,
  not new tests.

---

## Wave: DEVOPS / [HOW] Proposed CI diffs

> All four are **PROPOSALS for the apply step**. Real files the orchestrator must
> touch after approval: `.github/workflows/rust-tests.yml` (CHANGE 1, 2, 3),
> `.releaserc.yml` + `.releaserc` (CHANGE 4 — deletions).

### CHANGE 1 — clippy `-D warnings` as a FAILING gate (`rust-tests.yml`)

**Before** (current `clippy` job, lines 57–74):

```yaml
  clippy:
    name: ✂️ Rust Clippy
    runs-on: ubuntu-latest
    steps:
      - name: ♻️ Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 1
      - name: 🛠️ Install correct toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          profile: minimal
          components: clippy
      - name: ♻️ Cache
        uses: Swatinem/rust-cache@v2
      - name: ✂️ Clippy
        run: cargo clippy --all-targets -- -W clippy::pedantic
```

**After** (deny-warnings is the blocking gate; pedantic kept as a separate,
non-failing advisory step):

```yaml
  clippy:
    name: ✂️ Rust Clippy
    runs-on: ubuntu-latest
    steps:
      - name: ♻️ Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 1
      - name: 🛠️ Install correct toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          profile: minimal
          components: clippy
      - name: ♻️ Cache
        uses: Swatinem/rust-cache@v2
      - name: ✂️ Clippy (deny warnings — FAILING gate)
        run: cargo clippy --all-targets -- -D warnings
      - name: ✂️ Clippy pedantic (advisory — never fails the build)
        run: cargo clippy --all-targets -- -W clippy::pedantic
        continue-on-error: true
```

Mirrors the devenv `prepare` script (`cargo clippy --all-targets -- -D warnings`),
so local and remote agree.

### CHANGE 2 — re-enable the formatting gate (`rust-tests.yml`)

**Before** (currently commented out, lines 75–89):

```yaml
  # rustfmt:
  #   name: ✂️ RustFMT
  #   runs-on: ubuntu-latest
  #   steps:
  #     - name: ♻️ Checkout repository
  #       uses: actions/checkout@v4
  #     - name: 🛠️ Install Rust
  #       uses: dtolnay/rust-toolchain@stable
  #       with:
  #         toolchain: stable
  #         profile: minimal
  #         components: rustfmt
  #     - name: 🛠️ Check formatting
  #       run: |
  #         cargo fmt --all -- --check
```

**After** (re-enabled as a blocking job):

```yaml
  rustfmt:
    name: ✂️ RustFMT
    runs-on: ubuntu-latest
    steps:
      - name: ♻️ Checkout repository
        uses: actions/checkout@v4
      - name: 🛠️ Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          profile: minimal
          components: rustfmt
      - name: 🛠️ Check formatting
        run: cargo fmt --all -- --check
```

Trade-off (treefmt vs `cargo fmt --check`) documented in the Coexistence matrix
and ADR-0007 — `cargo fmt --check` chosen as the pragmatic, nix-free CI gate.

### CHANGE 3 — threaded `cargo test` job, ADR-0006 determinism guard (`rust-tests.yml`)

**New job** (add alongside `unittest`; plain threaded `cargo test`, stable
toolchain — would have caught the `GLOBAL_VAR` flakiness that nextest's
process-per-test masked):

```yaml
  threaded-test:
    name: 🧵 Threaded cargo test (ADR-0006 determinism guard)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: 🛠️ Install correct toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          profile: minimal
      - uses: Swatinem/rust-cache@v2
      - name: 🧵 Run threaded cargo test
        run: cargo test
```

`cargo test` defaults to multi-threaded, single-process execution — the exact
condition under which the old shared `OnceCell` flaked. Pairs with the existing
nextest job (process-per-test) so both execution models are covered.

### CHANGE 4 — resolve the dual release config (file DELETIONS)

`release.yml` (conventional-changelog-action) is the **active, working** release
mechanism. `.releaserc.yml` + `.releaserc` describe a **dormant** semantic-release
pipeline that **no workflow invokes** — dead config that misleads readers into
thinking crates.io publishing happens (it does not).

**Recommended (lower-risk default): DELETE both dormant files.**

- `DELETE .releaserc.yml`
- `DELETE .releaserc`

This leaves `release.yml` as the single source of release truth. No workflow
references these files, so deletion is behavior-preserving.

**Alternative (rejected for now): wire semantic-release instead.** Add a workflow
invoking `npx semantic-release` (using `.releaserc.yml`) and **delete
`release.yml`**. This would enable crates.io publishing via
`semantic-release-rust`. Rejected because: (a) `release.yml` already works and is
in active use (recent `ci: version bump` commits); (b) semantic-release needs a
Node toolchain + `NPM_TOKEN`/`CARGO_REGISTRY_TOKEN` secrets + `semantic-release-rust`
not currently installed; (c) crates.io publication is an *unrequested* new
capability for a single-user CLI. Switching mechanisms is a larger, higher-risk
change than removing dead config. Revisit only if crates.io distribution becomes
a goal.
