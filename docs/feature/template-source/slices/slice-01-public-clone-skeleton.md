# Slice 01 — Public clone skeleton (TS-01)

**Goal:** A git URL in `cv_template_path` is auto-detected and the public repo's default branch is cloned, then handed
to the existing build flow.

**Story:** TS-01 · **Walking skeleton:** yes

## IN scope

- Auto-detect LOCAL (existing dir) vs GITHUB (`git@…` / `https://….git`) from `cv_template_path`.
- `TemplateSource` trait + `LocalDirectory` (passthrough) + `GitHubRepository` (public clone) impls.
- Clone default branch into the cache dir; return that local dir to `prepare_path_for_new_cv` before `copy_dir`.
- Backward-compat: existing local-dir config path unchanged.
- `git` pre-usage PATH check (ADR-0004) for GITHUB.

## OUT scope

- Private auth (slice 02), ref pinning (slice 03), cache-reuse-on-failure / offline (slice 04).
- CLI overrides; non-GitHub hosts; git-mechanism optimisation.

## Learning hypothesis

Disproves **"a `TemplateSource` resolver can turn a git URL into a local dir the existing `copy_dir` flow accepts"** if
the cloned dir cannot drive an unmodified `just build`.
Confirms the end-to-end seam works → slices 02–04 only thicken it.

## Acceptance criteria

TS-01 AC1–AC4 (see feature-delta.md). Production data: clone the real public repo form.

## Dependencies

None (first slice). Establishes the abstraction the others extend.

## Effort / reference class

≤1 day. Reference: prior port-introduction slices in this repo (`CommandRunner`, ADR-0002) — trait + one concrete impl +
wiring.

## Pre-slice SPIKE

**Recommended:** `/nw-spike template-source` to pick the git mechanism (shell-out `git` vs `git2`/`gitoxide`) before
implementing this slice — network I/O is new to the codebase.

## Dogfood moment

Same day: point `cv_template_path` at the public repo URL and generate one real CV.
