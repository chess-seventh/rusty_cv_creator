# DESIGN Decisions — template-source

> Wave: DESIGN (nw-solution-architect, Morgan). Mode: PROPOSE. Density: LEAN.
> Builds on DISCUSS (D1-D7 locked) and the SPIKE (mechanism proven). Paradigm:
> OOP/hexagonal (traits=ports, structs=adapters, free functions=use cases).

## Key Decisions

| ID | Decision | Choice (vs alternative) | Status |
| --- | --- | --- | --- |
| TS-D1 | Error model | std-only `TemplateSourceError` enum (vs `thiserror`, vs `Box<dyn Error>` strings) | confirmed 2026-06-22 |
| TS-D2 | Cache responsibility | `TemplateCache` collaborator + pure `CacheAction` decision (vs all-in-`resolve`) | confirmed 2026-06-22 |
| TS-D3 | Auth transport | `GITHUB_TOKEN` env + `core.askpass` (vs URL-embedded token) | confirmed 2026-06-22 |
| TS-D4 | ADR-0004 PATH check | DEFER; keep `#[serial]`, record smell (vs `ToolChecker` port now) | confirmed 2026-06-22 |
| TS-D5 | Git mechanism | System `git` via `CommandRunner` (vs `git2`, vs `gitoxide`) | accepted (ADR-0008) |

TS-D1..D4 were **confirmed by the user on 2026-06-22** (recommended option
accepted in each case). Rationale is in `feature-delta.md` under
`## Wave: DESIGN / [REF] Confirmed decisions`.

## Architecture Summary

`TemplateSource` is a new driven port resolving `cv_template_path` to a local
template dir consumed unchanged by `copy_dir` (D5). `detect_template_source`
auto-detects LOCAL (`LocalDirectory` passthrough, backward compat) vs GITHUB
(`GitHubRepository`). The git adapter shells out to system `git` through the
existing `CommandRunner` port (ADR-0008), gated by the ADR-0004 PATH check. A
`TemplateCache` collaborator owns cache-key derivation and a pure
reuse-vs-fetch-vs-abort `CacheAction` decision; only its executor writes, bounded
to the cache dir. Ref pinning logs the resolved SHA and never silently falls
back. A typed `TemplateSourceError` enum distinguishes auth / network-offline /
bad-ref / no-cache so each emits a distinct hint; classification reads git stderr
via an additive `CommandRunner` extension. C4 (L1/L2/L3) in
`docs/product/architecture/c4-diagrams.md` under "Template sourcing".

## Reuse Analysis

| Component | Verdict | Justification |
| --- | --- | --- |
| `TemplateSource` trait | EXTEND | Skeleton exists; error type → `TemplateSourceError` |
| `LocalDirectory` | EXTEND | Behaviour unchanged; error type only |
| `GitHubRepository` | EXTEND | Add ref pinning, cache reuse/offline, auth, classification |
| `detect_template_source` / `is_git_url` | EXTEND | Thread ref + auth into construction |
| `CommandRunner` port | EXTEND | Additive stderr-capturing run for failure classification |
| `SystemRunner` / `FakeRunner` | EXTEND | Implement new method; `FakeRunner` canned stderr |
| `ensure_tools_available` (ADR-0004) | REUSE | `git` already gated; no change |
| `resolve_template_cache_dir` | REUSE | Already reads `cv_template_cache` + default |
| `prepare_path_for_new_cv` wiring | EXTEND | Read two new INI keys; pass to detect |
| `AppContext` / `get_variable_from_config_file` | REUSE | New keys via existing accessor |
| `TemplateCache` | CREATE NEW | No existing cache-lifecycle component |
| `TemplateSourceError` | CREATE NEW | No existing typed error; needed for distinct hints |
| `AuthMode` / auth resolution | CREATE NEW | No existing auth modelling |

## Tech Stack

| Choice | Selection | License |
| --- | --- | --- |
| Git mechanism | System `git` via `CommandRunner` (ADR-0008) | n/a (runtime tool) |
| Error model | Hand-rolled enum, std only | n/a |
| Token transport | `GITHUB_TOKEN` env + `core.askpass` | n/a |
| Cache | `fetch`+`checkout` by `repo@ref`; no shallow | n/a |

No new crate added. All existing deps remain OSS (MIT/Apache-2.0).

## Constraints

- D5: downstream build flow unchanged — resolver returns a local dir into
  `copy_dir`.
- Secrets env-only: token never in INI, argv, or cache `.git/config` (D2).
- No silent fallback: bad ref (TS-03/AC3) and missing cache (TS-04/AC2) abort.
- `git` gated by ADR-0004 pre-usage PATH check.
- First-run clone ~3.5s (41 MB binary assets) unavoidable; cache reruns ~1.7s.
- Cache write universe bounded to `cv_template_cache` (capability scope).
- Detached HEAD after SHA checkout → always log the resolved SHA.

## Upstream Changes

One additive, backward-compatible upstream change to a shared component — the
`CommandRunner` port (ADR-0002) gains a stderr-capturing run so git failures can
be classified. Detailed in `upstream-changes.md`. No contradictions with locked
DISCUSS decisions or the existing SSOT were found.

## External Integration Annotation (DEVOPS handoff)

The GitHub template repo is consumed over **git transport** (SSH/HTTPS), not a
REST/GraphQL API, so consumer-driven HTTP contract tooling (Pact) does not apply
(consistent with the brief's existing template-contract note). Recommended
instead:

- A reachability/auth smoke test in CI against a real `file://` (or the real)
  repo fixture exercising clone + ref checkout.
- A **gold-test catalogue of real `git` stderr strings** guarding the
  `TemplateSourceError` classifier — git's wording is the substrate that can
  change; the classifier must keep mapping it to the right hint.
