# Evolution Record — template-source

> Long-term archive / finalize record for the `template-source` feature.
> Delivered as a **real DELIVER wave** (not a backfill) across the full nWave
> chain on branch `feature/template-source-skeleton`. Density: **LEAN**.
> Finalized: 2026-06-23.

## What Shipped

The CV template can now be sourced from **either a local directory or a GitHub
repository** — transparently, behind the unchanged `insert` CLI entry point and
the reused `[cv] cv_template_path` config key (D1, D6). A `GitHubRepository`
adapter clones/fetches/checks-out the repo via the system `git` binary (through
the existing `CommandRunner` port — no Rust git crate, ADR-0008), supporting:

- **Public and private access** — SSH (`git@…`, inherits the agent, the user's
  real workflow) or HTTPS token via `core.askpass` reading `GITHUB_TOKEN` from
  the environment; the secret never touches the INI, the argv, or the cache
  repo's `.git/config` (D2).
- **Ref pinning** — `[cv] cv_template_ref` checks out a branch/tag/SHA and logs
  the resolved SHA; an unresolvable ref aborts fast and **never** silently falls
  back to the default branch (D3).
- **Offline cache reuse** — clones are cached per `repo@ref` under
  `[cv] cv_template_cache`; on a fetch failure with a usable cache the entry is
  reused (with a warning), and with no cache the run aborts fast rather than
  producing a partial CV (D4).

The downstream build flow (`copy_dir` → `just build` → per-year filing) is
untouched: the feature only changes how the local template directory handed to
`copy_dir` inside `prepare_path_for_new_cv` is produced (D5). An existing local
directory behaves exactly as before — the backward-compat regression guard.

## The 6 DES-Instrumented TDD Steps (oldest → newest)

> Each step was a 3-phase RED → GREEN → COMMIT cycle with a complete DES trace
> (integrity verified, exit 0).

| Step | SHA | What landed |
| --- | --- | --- |
| 01-01 | `c144d40` | UC-1 `CommandRunner::run_capturing` stderr seam (`CommandOutcome` carrier). |
| 01-02 | `037c778` | `TemplateSourceError` enum + `classify_git_stderr` (typed failure classes → distinct hints). |
| 02-01 | `361b466` | `AuthMode` + `auth_invocation_flags` (SSH agent / token via `core.askpass`). |
| 03-01 | `8887c2f` | Ref pinning (`with_ref` + checkout + resolved-SHA log; bad-ref aborts, no fallback). |
| 04-01 | `07f7831` | `TemplateCache::decide` (the 2×2 cache/network matrix) + deterministic `cache_key`. |
| 04-02 | `52457d1` | Final wiring into `prepare_path_for_new_cv` + scaffold removal. |

### Post-step hardening

| SHA | Role |
| --- | --- |
| `710f111` | L1/L3 refactor pass (clippy/treefmt clean). |
| `e9327e1` | Adversarial-review fix — **BLOCKER**: `fetch` was missing auth flags → fixed and proven RED-without-fix; closed 3 testing-theater coverage gaps on `resolve_classified` / `resolve_cached`. |
| `3b12340` | Mutation hardening on `src/template_source.rs`. |

## Key Decisions (ADR links)

- [ADR-0008](../product/architecture/adr-0008-template-source.md) — Template
  sourcing via system `git` shell-out through `CommandRunner` (cache by
  `repo@ref`, ref pinning, env-only token via `core.askpass`). Records the
  deferred git-mechanism choice (shell-out vs `git2`/`gitoxide`) made in
  SPIKE/DESIGN — the closing DoD item.
- [ADR-0004](../product/architecture/adr-0004.md) — `git` reused as a pre-usage
  PATH-gated tool for the GITHUB source (`ensure_tools_available`).
- TS-D1..TS-D4 (DESIGN decisions table) — error model, cache responsibility,
  auth transport, and the deferred `ToolChecker` port (kept `#[serial]`).

## Measured Outcomes

- **Tests**: full `cargo test` **156/156 green, 0 ignored**, across 7 binaries
  (`lib` 9, `main` 89, `cli_smoke` 3, `integration-tests` 20,
  `template_source_scenarios` 1, `tui_*_scenarios` 3, `tui_*_specifications` 31).
- **Mutation testing** (per-feature, CLAUDE.md): `cargo-mutants` on
  `src/template_source.rs` → 51 caught / 1 missed / 9 unviable =
  **98.1% kill rate** (gate ≥ 80%). Lone survivor: `is_git_url` `&&` → `||`
  (minor; URL detection retains a correct happy path).
- **Acceptance coverage**: 16 DISTILL scenarios across 4 documentation
  `.feature` files, 9 error/edge (**56%**), all mapped to GREEN Rust tests.
- **Adversarial review**: `needs_revision` → all 7 findings resolved → approved.
- **KPI baselines** (recorded inline — `docs/product/kpi-contracts.yaml` is
  absent and was not bootstrapped): zero-manual-git **achieved** (one config
  value, 0 git commands on a fresh machine); offline reuse **implemented**
  (TS-04 cache-reuse path); clone-latency overhead **SPIKE-measured** ~3.5s cold
  clone vs ~1.7s cache `fetch`+`checkout`.

## Demo Evidence (test-based)

The live LaTeX + DB PDF demo was intentionally **not** run — the downstream
build is pre-existing and untouched (D5). Demo evidence is therefore test-based:

- The DISTILL **walking skeleton** performs a real `file://` `git clone` →
  `copy_dir` (real `SystemRunner`, real filesystem, no network, no git mock) —
  `src/file_handlers.rs::walking_skeleton_github_source_resolves_template_dir`.
- `tests/cli_smoke.rs` spawns the built binary and asserts the driving-adapter
  contract (bad template value fails fast naming the offending value).

## Deferred / Follow-ups (carried forward)

1. **Askpass helper executable (TS-02/AC2 HTTPS-token mode)** — the
   `core.askpass=git-askpass-from-env` flag wiring and the secret-absence path
   are done and tested, but the helper **executable** that reads `GITHUB_TOKEN`
   is not materialized and no hermetic test exercises a real private-HTTPS token
   clone. SSH (the user's real workflow) fully works. Follow-up: add the helper
   plus an integration test.
2. **Dual `TemplateCache` derivation** — `resolve_cached` and
   `clone_destination` derive the cache-entry path in two spots from the same
   `cache_dir` (correct today, could diverge if a future change splits them).
   Scoped refactor: have `resolve_cached` use the injected
   `cache.entry_path(...)`.

## Retrospective (LEAN)

- **What went well**: the 6 carpaccio slices each added one behaviour onto the
  slice-01 resolver — the new `TemplateSource` abstraction shipped **inside**
  slice 01 with a working concrete impl, never as standalone plumbing. Pure
  decision values (`is_git_url`, `CacheAction`, `auth_invocation_flags`,
  `classify_git_stderr`) kept effects bounded to the cache dir and made every
  failure path fake-testable. Mutation testing drove a real `3b12340` hardening
  pass to 98.1% kill.
- **What caught us**: the adversarial review found a real BLOCKER — `fetch` was
  silently missing the auth flags that `clone` had — proving the value of the
  review gate and of RED-without-fix demonstration over coverage theater.
- **What to improve**: the highest-realism boundary (a real private-HTTPS token
  clone) is still unproven (follow-up 1); the SSH path that the single user
  actually uses is fully exercised, so this was a deliberate, scoped deferral.

## Handoff to Operations

- **Runbook**: single-user local CLI; no service to operate. Sourcing is
  config-driven — set `[cv] cv_template_path` to a git URL (optionally
  `cv_template_ref` / `cv_template_auth` / `cv_template_cache`). For private
  repos: a working SSH agent/key, or `GITHUB_TOKEN` in the environment.
- **Rollback**: revert to a prior tag / reinstall the prior binary; a local-dir
  `cv_template_path` is the unchanged fallback path. No migrations.
- **Monitoring**: N/A (interactive CLI). Operational health = the pre-usage
  `ensure_tools_available(["git"])` probe (ADR-0004) and the offline
  cache-reuse warning.
