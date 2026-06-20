# DISCUSS Wave Decisions — cv-variant-build

> Retroactive backfill (LEAN, Tier-1 [REF] only). Requirements, JTBD, and user
> stories were reverse-engineered from the shipped code (branch
> `feature/change-generation`, v4.0.2) and the DESIGN [REF] sections. This wave
> documents realized behavior and is consistent with DESIGN — it proposes no new
> work. interaction: non-interactive backfill.

## Key Decisions

| ID | Decision | Source |
|----|----------|--------|
| RD-1 | Variant selection is explicit-or-inferred: valid `--variant` → keyword inference → configured default (`senior-devops` fallback). | realized by DESIGN D-1 / ADR-0001 |
| RD-2 | Exactly 4 role variants in scope: `senior-devops`, `senior-platform-engineer`, `senior-sre`, `engineering-manager`. | `CV_VARIANTS` (file_handlers.rs) |
| RD-3 | Manager-family titles resolve to `engineering-manager` before any other keyword. | `infer_variant_from_job_title` |
| RD-4 | Persistence is opt-in via `--save-to-database`; default run produces a PDF only. | cv_insert.rs |
| RD-5 | View is opt-in via `--view-generated-cv`; deterministic output name `<date>-<job>-<company>.pdf` under per-year output dir. | main.rs / file_handlers.rs |

## Requirements Summary

Single-user CLI job `apply-with-tailored-cv` (SSOT: `docs/product/jobs.yaml`).
Five focused user stories, all tracing to that job:

- US-01 Generate a chosen variant by flag
- US-02 Infer the variant from the job title
- US-03 Land the PDF in an organized, predictable location
- US-04 Record the application in the database
- US-05 Preview the generated PDF before sending

Full LeanUX form, ACs, and outcome KPIs in `discuss/user-stories.md`; DISCUSS
[REF] sections appended to `feature-delta.md`.

## Constraints (carried from DESIGN)

- Single-user local CLI; Postgres reachable only over Tailscale; SQLite local/tests.
- Toolchain pinned to Rust nightly; test determinism needs `cargo-nextest`
  (process-global `GLOBAL_VAR` `OnceCell`).
- Build correctness depends on the external CV template repo's Justfile
  recipe/output contract (`just build <variant>` → `<prefix>-<variant>.pdf`).
- External tools gated by pre-usage PATH checks with a devenv hint (ADR-0004):
  `just`, `tectonic`, `zathura` (view), `sudo`+`tailscale`.

## Elephant-Carpaccio Slicing

**N/A — already shipped.** The feature is implemented and merged (v4.0.2, 6
commits). No walking skeleton, slice briefs, or learning hypotheses are
fabricated for shipped code; vertical slicing is not applicable to a retroactive
backfill.

## Upstream Changes

None. This is a backfill consistent with the DESIGN wave; no DISCOVER/DIVERGE
artifacts existed and no upstream nWave artifacts were modified. The DESIGN [REF]
sections and ADR-0001..0005 were read and reconciled, not changed.
