# DISTILL Upstream Issues / Deviations — template-source

> No untestable ACs and no `SPECIFICATION_AMBIGUITY` blockers were found. The
> DISCUSS ACs (TS-01..04) and DESIGN ports/decisions (TS-D1..D4, UC-1) fully
> determine every scenario. Recorded below: one test-placement deviation and one
> graceful-degradation note for the orchestrator.

## DI-1 — Unit specifications are in-crate, not in `tests/template_source_specifications.rs`

- **Type:** test-placement deviation (process), no behaviour impact.
- **What:** the dispatch brief named an external `tests/template_source_specifications.rs`
  for the FakeRunner unit specs. They are instead in-crate:
  `src/template_source.rs::distill_specs` and `src/command_runner.rs::uc1_specs`.
- **Why (hard Rust constraint):** `command_runner` and `template_source` are
  **binary-private** modules — `lib.rs` exposes only `database`/`models`/`schema`/`tui`.
  `command_runner::testing::FakeRunner` is additionally `#[cfg(test)]`-gated, so it
  is not in the library build at all. An external `tests/` integration crate links
  the **library** target and therefore cannot reach `TemplateSource`, the scaffolds,
  or `FakeRunner`. Exposing them via `lib.rs` cascades: `template_source` →
  `helpers`, and `helpers` references the `main.rs`-local `is_tailscale_connected`,
  which would force relocating binary orchestration into the library — a larger,
  riskier refactor explicitly out of scope ("add, don't mutate working signatures").
- **Precedent:** the existing GREEN skeleton already keeps its FakeRunner specs
  in-crate (`src/template_source.rs::tests`). The deviation keeps the new specs in
  the same idiomatic home.
- **Split preserved:** the driving-adapter/subprocess scenarios DO live in the
  external `tests/template_source_scenarios.rs` (they only need the built binary,
  not the private symbols) — so the scenarios/specifications split the brief asked
  for is intact; only the specifications half moved to its compilable location.
- **Action for operator:** accept the in-crate placement (recommended), or schedule
  a separate ADR/refactor to promote `command_runner`/`template_source` (and a
  testing-only `FakeRunner` behind a `testing` feature) onto the library facade if
  an external specifications crate is later desired.

## DI-2 — Outcomes registry deferred (graceful degradation)

- `docs/product/outcomes/registry.yaml` is absent. Per the orchestrator
  instruction, a new registry was **not** bootstrapped. Outcome registration for
  the new typed contracts (`TemplateSource::resolve_classified`, `CacheAction`
  decision, `TemplateSourceError` classification) is **N/A — deferred**; revisit if
  an outcomes registry is adopted project-wide.

## DI-3 — Subprocess scenario depends on devenv PATH (not a blocker)

- `ts01_ac3_…` runs `insert` end-to-end; `prepare_cv` runs the ADR-0004 pre-usage
  check for `just`/`tectonic` **before** template detection. Under `devenv shell`
  those tools are present, so the run reaches `detect_template_source` and fails
  naming the bad value (asserted). Outside devenv the test would fail earlier at the
  tool check. The project CI runs gates in devenv (commit `6e4b905`), so this is
  consistent; documented for awareness only.
