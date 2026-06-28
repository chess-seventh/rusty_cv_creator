# SPIKE Decisions — template-source

## Assumption Tested

- Shelling out to system `git` can clone the **private** `git@github.com:chess-seventh/cv.git` over SSH (existing
  agent), checkout an arbitrary ref/SHA into a cache dir, and reuse the cache offline — with no Rust git library.

## Probe Verdict

- **WORKS**: clone 3.5s, ref/SHA checkout OK, bad-ref fails non-zero (no silent fallback), cache `fetch` reuse 1.7s,
  offline leaves cache usable, build contract (`Justfile` + variant `.tex`) present. See `findings.md`.

## Promotion Decision

- **PROMOTE** (user, 2026-06-22): mechanism proven and worth building on; refactor the probe into a committed walking
  skeleton for slice 01 / TS-01.

## Walking Skeleton (PROMOTE)

- Driving adapter: `rusty_cv_creator insert` (existing CLI entry) — sourcing transparent inside
  `prepare_path_for_new_cv`.
- Mechanism: `git` shell-out via the existing `CommandRunner` port (ADR-0002); `git` gated by `ensure_tools_available`
  (ADR-0004).
- Acceptance test: real git clone from a **local bare-repo fixture (`file://`)** — exercises the real git driven adapter
  without network flakiness; asserts the resolved template dir is produced from a git-URL source.
- Commit: `af03e33` on `feature/template-source-skeleton` (3 files, +392/−11; 131 tests green, +9; clippy/fmt clean; not
  pushed).
- Files: `src/template_source.rs` (new: `TemplateSource` trait + `LocalDirectory` + `GitHubRepository` +
  `detect_template_source`/`is_git_url`), `src/file_handlers.rs` (threaded `&dyn CommandRunner`, resolve before
  `copy_dir`, `resolve_template_cache_dir`), `src/main.rs` (wiring).
- Acceptance test: `walking_skeleton_github_source_resolves_template_dir` (`src/file_handlers.rs:523`, tagged
  `// @walking_skeleton @driving_port`) — real `git clone` via `SystemRunner` from a `file://` bare-repo fixture → real
  `copy_dir`; asserts the git-sourced template lands ready to build (LaTeX build intentionally not run).
- Demo command: `devenv shell -- cargo test --bin rusty_cv_creator walking_skeleton_github_source_resolves_template_dir`

## Upstream Issues

- **UI-1** (LOW, process): mandated 82-char commit title violated repo gitlint T1 (≤72). Resolved without bypassing the
  hook (shortened subject, full sentence in body). See `upstream-issues.md`. Operator decision pending: accept short
  titles (recommended) vs. relax gitlint.

## Skeleton-revealed notes (for DESIGN)

- `file://` is now also recognised as a git URL (superset of D1's `git@…`/`https://….git`) — enables deterministic
  offline tests; breaks nothing.
- ADR-0004 PATH check uses global `PATH` → git-touching tests must be `#[serial]`. DESIGN may make the tool-check
  injectable to drop the global-state coupling.
- `GitHubRepository` always clones fresh (no reuse yet) — cache-key-by-`repo@ref` + reuse/offline are slices 03/04
  (TS-03/TS-04).

## Design Implications (for DESIGN)

- `TemplateSource` trait + `LocalDirectory` (passthrough) + `GitHubRepository` (CommandRunner-backed) — OO shape
  (DISCUSS D7) confirmed viable.
- Cache key = sanitised `repo@ref` under `cv_template_cache` (default `~/.cache/rusty-cv-creator/templates`); reuse via
  `fetch`+`checkout`, NOT re-clone. **Drop shallow-clone** (binary assets dominate; no benefit).
- Auth inferred from URL scheme (`git@…` SSH / `https://…` anon|token-from-env); no secret in INI.
- SHA checkout → detached HEAD → always log resolved SHA (TS-03/AC1).
- Downstream build flow untouched (D5): resolver returns a local dir into `copy_dir`.

## Constraints Discovered

- First-run clone latency ~3.5s (41 MB, binary assets) unavoidable; cache makes reruns ~1.7s — feeds the clone-latency
  KPI.
- Detached HEAD after SHA checkout — resolver must checkout explicitly and log the resolved SHA.
