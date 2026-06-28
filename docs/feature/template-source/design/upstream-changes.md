# Upstream Changes — template-source (DESIGN)

> No **contradictions** with locked DISCUSS decisions (D1-D7) or the existing
> SSOT were found. One additive upstream change to a shared component is recorded
> below so the orchestrator is aware before it touches another feature's surface.

## UC-1 — EXTEND the `CommandRunner` port (ADR-0002) with stderr capture

- **Type:** additive, backward-compatible (new method; existing
  `status`/`output`/`spawn` unchanged).
- **Why:** TS-02/AC3, TS-03/AC3 and TS-04/AC2 require *distinct* actionable hints
  for auth vs network/offline vs bad-ref vs no-cache. Distinguishing them requires
  inspecting git's **stderr** (e.g. "Permission denied (publickey)" → auth;
  "Could not read from remote repository" → network/offline; "couldn't find
  remote ref" → bad-ref). The current port exposes only `status` (bool) and
  `output` (stdout only) — neither captures stderr.
- **Proposed shape (described, not prescribed):** a run method returning a small
  outcome value carrying `success`, `stdout`, and `stderr` (and optionally a `cwd`
  argument, as `status` already has). `SystemRunner` fills it from
  `Command::output()`; `FakeRunner` gains a canned-stderr field for gold-tests.
- **Blast radius:** `src/command_runner.rs` only. Existing call sites
  (`compile_cv`, `view_cv_file`, `is_tailscale_connected`) are untouched because
  the change is additive. ADR-0002 should get a short amendment note that the
  port grew a stderr-capturing run for template sourcing (recorded here and in
  ADR-0008's Consequences).
- **Decision owner:** software-crafter implements during GREEN; this is a design
  heads-up, not a code change in the DESIGN wave.

## Non-issues (checked, no action)

- The skeleton's `is_git_url` recognising `file://` is a superset of D1 and breaks
  nothing (enables deterministic offline tests) — already noted in the SPIKE.
- D5 holds: the resolver returns a local dir into the unchanged `copy_dir` flow.
- The brief's L2 "Config … OnceCell/GLOBAL_VAR" label predates ADR-0006 but is out
  of scope for this feature (already flagged as delivered in the brief).
