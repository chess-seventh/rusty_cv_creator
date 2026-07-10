# RCA — tailored CVs render 3 pages instead of the 2-page contract (Q4)

Investigated 2026-07-09 (nw-troubleshooter, reproduced for real). Bug fix via
/nw-bugfix; this document is the Phase-1 RCA context for the DELIVER roadmap.

## Defect

Tailored CVs (senior-sre variant, jobs 12/19/34 from the 2026-07-09 jobs-forge
run) render **3 pages**; the contract is 2. rusty_cv_creator reported success.

## Root cause chain (condensed)

1. **Layout is quantized:** `cvpasts` + each `cventry` in awesome-cv.cls are
   single unbreakable `tabular*` boxes — overflow moves a whole ~10cm block to
   page 3, all-or-nothing.
2. **Zero headroom:** the template was tuned to exactly 2 pages; tailored
   content grew +0.6–1.1% and tipped it. jobs-forge's +15% char guard is a
   placebo (real slack < 1%).
3. **No enforcement (THIS repo's root cause):** `compile_cv`
   (`src/file_handlers.rs`, ~line 115) treats *builder exited 0* as success.
   tectonic exits 0 for any page count; nothing anywhere asserts the artifact
   meets the 2-page contract. `grep -i page` over `src/` + `tests/` = zero hits.

## Fix for THIS repo (approved by Franci, 2026-07-09)

Page-count assertion at the render seam, fail closed:

- `compile_cv` (`src/file_handlers.rs`): instead of `runner.status(...)`, use
  `runner.run_capturing(...)` (already on the `CommandRunner` trait,
  `src/command_runner.rs`) and pass a `tectonic=tectonic --print` just-variable
  override so the TeX transcript reaches stdout/stderr.
- Parse `Output written on <file> ((\d+) pages?` from stdout+stderr combined
  (tectonic routes notes to stderr; parse both so routing changes can't blind
  the guard). Take the LAST match (tectonic runs multiple passes).
- Error if `pages > max_pages` **or if no match at all** (fail closed — a
  transcript format change must break loudly, never silently ship a 3-pager).
- `BuildConfig`: new `max_pages` field, default 2, read from `[build]
  max_pages` in the config INI; document in `rusty-cv-config-example.ini`.
- The just-override string should be configurable too (`[build]
  page_count_probe` or similar) — the renderer knowing a template Justfile
  variable name is a coupling; default `tectonic=tectonic --print`.

## Regression tests (the primary deliverable)

Via `FakeRunner` (exists in `src/command_runner.rs` testing module):

1. transcript says `(3 pages, ...)` → `compile_cv` returns `Err` (THIS is the
   bug regression test — it FAILS on current code, which returns Ok).
2. transcript says `(2 pages, ...)` → `Ok`.
3. transcript has NO page line → `Err` (fail closed).
4. page line on **stderr** instead of stdout → still parsed (extend FakeRunner
   with a `with_stderr` constructor if absent).
5. `max_pages` honored from config (e.g. 1 → a 2-page transcript fails).
6. Existing e2e `test_prepare_cv_end_to_end_with_fake_builder` (`src/main.rs`,
   `PdfWritingRunner`) must keep passing — update `PdfWritingRunner` to emit a
   2-page transcript line.

## Sibling fix (already landed, separate repo)

`_cv_template` commit `4e84a47` (branch `fix/2-page-overflow-compact`):
senior-sre goes `\cvcompact` + `\setstretch{0.96}`; `just test-build` now
page-gates all variants (RED on jobs 12/19/34 old template, GREEN with fix).
This repo's assertion is the second layer: catch ANY future overflow at render
time regardless of variant/content.

## Risk

Low. Transcript line is TeX-kernel stable; tectonic is flake-pinned in the
consumer (jobs-forge). Fail-closed means format drift breaks loudly.
