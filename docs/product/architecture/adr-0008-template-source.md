# ADR-0008: Template sourcing via git shell-out through CommandRunner

## Status

Accepted (DESIGN wave, feature `template-source`; supersedes the deferred
git-mechanism question recorded in feature-delta D7 / DoD item 9)

## Context

`rusty_cv_creator` currently requires the CV template to already exist as a local
directory pointed at by `[cv] cv_template_path`. The `template-source` feature
(stories TS-01..TS-04) lets that value also be a GitHub git URL, so the tool
clones the canonical template itself, can pin an exact ref, and keeps working
offline from a cache. The DISCUSS wave locked the OO shape (D7): a
`TemplateSource` trait with `LocalDirectory` and `GitHubRepository` implementors.
The remaining decision — **how** the git work is performed — was explicitly
deferred to a SPIKE/DESIGN decision.

A throwaway SPIKE (`docs/feature/template-source/spike/findings.md`, verdict
**WORKS**) proved that shelling out to the system `git` binary satisfies every
locked requirement against the real private repo: SSH clone using the existing
agent (3.5s), arbitrary ref/SHA checkout (detached HEAD), bad-ref fails non-zero
with no silent fallback, warm-cache `fetch` reuse (1.7s), and offline `fetch`
leaving the cache usable. Shallow clone saved nothing (the 41 MB is binary
assets, not history), so it is dropped.

Testability is this codebase's primary quality driver (ADR-0002, ADR-0005). Any
mechanism must be unit-testable without real network or credentials.

## Decision

Source the template by **shelling out to the system `git` binary, routed through
the existing `CommandRunner` driven port (ADR-0002)**. No Rust git library is
added. `git` is gated by the existing pre-usage PATH check
(`ensure_tools_available`, ADR-0004) when the source is a git URL, exactly as
`just`/`tectonic` already are.

Mechanics:

- **Cache** under `[cv] cv_template_cache` (default
  `~/.cache/rusty-cv-creator/templates`), keyed by a sanitised `repo@ref`. Reuse
  is `git fetch` + `git checkout` of an existing cache entry, **not** a re-clone;
  a fresh `repo@ref` is a full clone. Shallow clone is not used.
- **Ref pinning** (`[cv] cv_template_ref`): checkout the branch/tag/SHA and log
  the resolved SHA; an unresolvable ref aborts (no silent fallback to default).
- **Auth** inferred from URL scheme (`git@…` → SSH agent; `https://…` → anon),
  overridable by `[cv] cv_template_auth = auto | ssh | token`. The `token` mode
  reads `GITHUB_TOKEN` from the environment and feeds it to git via an askpass
  helper (`git -c core.askpass=…`); the token is never read from or written to
  the INI, never placed on the git argv, and never persisted into the cache
  repo's `.git/config` remote URL.
- **Offline fallback** (TS-04): a fetch failure with a usable cache entry reuses
  it (with a warning); no cache entry aborts fast with an actionable hint.

## Alternatives considered

- **`git2` (libgit2 bindings)** — rejected. Adds a C-library dependency and build
  complexity (libgit2/OpenSSL), and its SSH-agent / credential-callback story is
  more code than simply inheriting the user's working `git` auth, which the SPIKE
  proved sufficient with zero credential plumbing. No requirement (single-user
  CLI, `git` already a devenv tool) justifies the dependency. License is fine
  (GPL-2.0-with-linking-exception) but the integration cost is not warranted.
- **`gitoxide` (pure-Rust git)** — rejected. Promising and dependency-light at
  the toolchain level, but in 2026 its higher-level clone/fetch/checkout +
  credential-helper surface is still maturing; betting the only network path in
  the codebase on it adds risk for no benefit over the system `git` the SPIKE
  validated. Reconsider only if removing the `git` runtime dependency becomes a
  goal.
- **Keep `std::process::Command` calls inline** — rejected for the same reason as
  ADR-0002: it would make the clone/fetch/checkout paths untestable without real
  network and credentials. Routing through `CommandRunner` keeps them fake-able.
- **Shallow clone (`--depth 1`)** — rejected as a performance lever: the SPIKE
  showed the working tree is 41 MB of binary assets, so `--depth 1` saved
  essentially nothing. Cache `fetch` reuse (1.7s vs 3.5s) is the real lever.

## Consequences

- Positive: zero new crates; consistent with the codebase's hexagonal style;
  clone/fetch/checkout are unit-testable with `FakeRunner`; the user's existing
  git auth (SSH agent) works with no plumbing; secrets stay out of the repo, the
  INI, the argv, and the cache config.
- Positive: the cache makes reruns ~2x faster and enables offline generation.
- Negative: the `CommandRunner` port must be **extended** with a
  stderr-capturing method so git's failure output can be classified into distinct
  actionable hints (auth vs network/offline vs bad-ref); see the template-source
  design in the feature delta. This is an additive, backward-compatible change to
  a shared port (ADR-0002).
- Negative: a hard runtime dependency on the `git` binary for git sources
  (mitigated by the ADR-0004 pre-usage check with a devenv hint).
- Negative: cache grows unbounded (no GC/eviction) — accepted for now, noted as a
  future concern (out-of-scope per DISCUSS).
