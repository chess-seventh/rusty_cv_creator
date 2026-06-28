# SPIKE Findings — template-source

**Date:** 2026-06-22 · **Agent:** Attila (nw-software-crafter) · **Phase:** PROBE

## Assumption tested (the ONE)

> Shelling out to system `git` can clone the **private** repo
> `git@github.com:chess-seventh/cv.git` over SSH (existing agent), checkout an
> arbitrary ref/SHA, into a cache dir, and reuse the cache offline — with **no
> Rust git library**.

Probe: `/tmp/spike_template-source/probe.sh` (throwaway, shell). Mechanism
validation only; no perf budget. Run against the **real private repo**, not a
fixture.

## Verdict: ✅ WORKS

Shell-out to `git` is sufficient for every DISCUSS-locked requirement. No
`git2`/`gitoxide` dependency is needed.

| # | Check | Result | Maps to |
| --- | ------- | -------- | --------- |
| 1 | SSH clone of private repo, default branch | ✅ **3.5s**, branch `main`, head `eff41fc`. Existing SSH agent used — zero credential plumbing. | TS-02 |
| 2 | Build contract present in clone | ✅ `Justfile` + `PivaFrancesco-{senior-devops,senior-platform-engineer,engineering-manager}.tex` + `awesome-cv.cls`. Exactly what `compile_cv` / `just build <variant>` consumes. | TS-01, D5 |
| 3 | Checkout arbitrary short SHA | ✅ detached HEAD (expected) | TS-03/AC1 |
| 4 | Bad ref | ✅ **non-zero exit, no silent fallback** | TS-03/AC3 |
| 5 | Cache reuse via `fetch` | ✅ **1.7s** vs 3.5s full clone | TS-04/AC3 |
| 6 | Shallow `--depth 1` | ✅ works but **same 41M** — repo size is fonts/images, not history. Shallow is NOT the perf lever. | (rejected optimisation) |
| 7 | Offline `fetch` (host unreachable) | ✅ git fails, **cache dir remains usable** | TS-04/AC1 |

## Timing (informal, single run)

- Cold full clone (SSH, private): **~3.5s**
- Warm cache `fetch`: **~1.7s**
- Repo working tree: **41 MB** (binary assets dominate; shallow clone saves ~nothing)

## Edge cases / caveats discovered

- **HTTPS anon** path (TS-01 public) not exercised against this repo (it is
  private). HTTPS reachability to GitHub was confirmed separately
  (`git ls-remote https://github.com/...` → exit 0); the public-clone mechanism
  is the same `git clone <url>` shell-out, lower-risk than the SSH case that
  already passed.
- Probe step 7 printed a spurious `exit=0` due to a `$?`-after-pipe scripting
  slip; git's own output (`fatal: Could not read from remote repository`) and the
  surviving cache confirm the real behaviour. Mechanism conclusion unaffected.
- SHA checkout leaves a **detached HEAD** — fine for a build-only consumer, but
  the resolver must `git -C <cache> checkout <ref>` explicitly and log the
  resolved SHA (TS-03/AC1) rather than relying on branch state.

## Design implications (for DESIGN)

1. **Mechanism = shell-out to `git`**, routed through the existing
   **`CommandRunner` port (ADR-0002)** — keeps it testable with `FakeRunner`,
   no new external crate, consistent with the codebase's hexagonal style.
2. **`git` gated by the existing pre-usage PATH check** (`ensure_tools_available`,
   ADR-0004) when source is GITHUB — same pattern as `just`/`tectonic`.
3. **Cache key = sanitised `repo@ref`** under `cv_template_cache`
   (default `~/.cache/rusty-cv-creator/templates`). Reuse = `fetch` + `checkout`,
   not re-clone. This is the real perf lever (1.7s vs 3.5s); **drop shallow-clone**.
4. **Auth is inferred from URL scheme** (`git@…` → SSH/agent; `https://…` → anon,
   or token via `GIT_ASKPASS`/env for private HTTPS). No secret touches the INI.
5. **Resolver returns a local dir** that flows unchanged into
   `prepare_path_for_new_cv` → `copy_dir` (D5 holds: downstream untouched).
6. **`TemplateSource` trait** with `LocalDirectory` (passthrough) +
   `GitHubRepository` (CommandRunner-backed) implementors — the OO shape from
   DISCUSS D7 is confirmed viable.

## Constraints discovered

- Detached-HEAD after SHA checkout → always log resolved SHA explicitly.
- 41 MB clone → first-run latency ~3.5s is unavoidable; cache makes subsequent
  runs ~1.7s. Acceptable for a single-user CLI; note for the clone-latency KPI.
