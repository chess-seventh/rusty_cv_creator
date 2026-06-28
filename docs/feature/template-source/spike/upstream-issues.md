# Upstream Issues — template-source

> Contradictions encountered during SPIKE Phase 3 (walking-skeleton promotion)
> that could not be resolved without violating a locked decision, the DISCUSS
> spec, or a project quality gate. Recorded here instead of being force-fitted
> (walking-skeleton DoD item 7).

## UI-1 — Mandated commit title exceeds the repo's gitlint 72-char limit

**Date:** 2026-06-22 · **Wave:** SPIKE Phase 3 (skeleton) · **Severity:** LOW (process)

**Contradiction.** The skeleton task mandated this exact commit title:

```text
feat(template-source): walking skeleton — clone git URL template via CommandRunner
```

That title is **82 characters**. The repository's `commit-msg` hook runs
`gitlint` (`.pre-commit-config.yaml`, hook id `gitlint`), whose default **T1
rule caps the title at 72 characters**. The two requirements are mutually
exclusive: the mandated message can never pass the commit-msg gate.

**Evidence.**

```text
✨ GitLint...............................................................Failed
  1: T1 Title exceeds max length (82>72):
     "feat(template-source): walking skeleton — clone git URL template via CommandRunner"
```

All substantive gates passed in the same run (Clippy ✓, TreeFMT ✓, EOF ✓,
secrets ✓), and the DoD-named gates were verified independently:
`rustfmt --edition 2024` (clean), `cargo clippy --all-targets -- -D warnings`
(clean), `cargo test` (131 passed / 0 failed).

**Resolution applied (no gate bypassed).** The commit was landed with a
conventional-commit subject shortened to ≤72 chars, and the **exact mandated
sentence preserved verbatim as the first line of the commit body**, so the
intended message is recoverable from `git log` without `--no-verify`.

Committed title used:

```text
feat(template-source): walking skeleton clones git URL via CommandRunner
```

**Decision needed upstream (DESIGN / operator).** Pick one:

1. Accept the shortened title convention for this feature (gitlint stays 72), or
2. Relax/configure gitlint's T1 limit (platform-architect territory) if longer
   titles with em-dash phrasing are desired going forward.
