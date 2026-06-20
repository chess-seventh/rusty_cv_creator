<!-- markdownlint-disable MD024 -->
# User Stories — cv-variant-build

> LEAN retroactive backfill. Reverse-engineered from shipped code (v4.0.2) +
> DESIGN [REF]. All stories trace to job `apply-with-tailored-cv`
> (`docs/product/jobs.yaml`), persona `job-seeker`. Real persona/data:
> Francesco; examples use real companies/titles. ACs are observable outcomes.

## System Constraints

- Single-user local CLI; build depends on the external CV template repo's
  Justfile contract (`just build <variant>` → `<prefix>-<variant>.pdf`).
- External tools pre-checked on PATH with a devenv hint (ADR-0004).
- Persistence: Postgres (prod, over Tailscale) or SQLite (local/tests).

## US-01: Generate a chosen variant by flag

job_id: apply-with-tailored-cv

### Elevator Pitch

- Before: Francesco hand-picks/edits the right LaTeX driver for a manager role and risks the wrong CV shape.
- After: run `rusty_cv_creator insert --job-title "Engineering Manager" --company-name "Datadog" --variant engineering-manager` → sees `CV saved to: <output>/2026/2026-06-20-Engineering-Manager-Datadog.pdf`.
- Decision enabled: Francesco confirms the manager-framed CV before attaching it.

### Problem

Francesco is a senior engineer who, for some applications, knows exactly which role framing he wants. He finds it slow and risky to manually ensure the correct LaTeX driver is built.

### Who

- Francesco (single user) | applying to a role where he wants explicit control | wants certainty over the variant.

### Solution

`--variant` selects one of the 4 variants explicitly; an invalid value warns and falls back to inference rather than aborting.

### Domain Examples

1. Happy path — Francesco applies to Datadog as "Engineering Manager" with `--variant engineering-manager`; PDF basename contains `engineering-manager`.
2. Edge case — `--variant senior-sre` with job title "Backend Developer"; the flag wins, `senior-sre` is built.
3. Error/boundary — `--variant bogus`; tool warns "Unknown variant" and infers from the job title instead.

### UAT Scenarios (BDD)

#### Scenario: Explicit variant is honored
Given Francesco runs `insert` with `--variant senior-sre`
When the CV is built
Then the build uses `senior-sre` and the output PDF basename contains `senior-sre`

#### Scenario: Invalid variant falls back to inference
Given Francesco passes `--variant bogus` with job title "Platform Engineer"
When the CV is built
Then the tool warns and builds `senior-platform-engineer` from the title

### Acceptance Criteria

- [ ] A valid `--variant` value is used verbatim for the build.
- [ ] An invalid `--variant` value warns and falls back to title inference (no abort).

### Outcome KPIs

- Who: Francesco | Does what: produces the intended variant on the first run | By how much: 100% of flagged runs build the flag's variant | Measured by: PDF basename vs flag | Baseline: manual editing, error-prone.

## US-02: Infer the variant from the job title

job_id: apply-with-tailored-cv

### Elevator Pitch

- Before: Francesco must remember and type the exact variant name for every application.
- After: run `rusty_cv_creator insert --job-title "Site Reliability Engineer" --company-name "Cloudflare"` → log `✅ Inferred variant 'senior-sre' from job title`.
- Decision enabled: Francesco trusts the inferred framing or overrides with `--variant`.

### Problem

Typing the exact variant for every application is tedious; Francesco wants the obvious case handled automatically.

### Who

- Francesco | applying to a role with an unambiguous title | wants minimal typing.

### Solution

When `--variant` is omitted, infer from job-title keywords (manager-family first), else use the configured default.

### Domain Examples

1. Happy path — "Senior Platform Engineer" at Cloudflare → `senior-platform-engineer`.
2. Edge case — "Engineering Manager - DevOps" → `engineering-manager` (manager wins over devops).
3. Error/boundary — "Accountant" → configured default variant (fallback `senior-devops`).

### UAT Scenarios (BDD)

#### Scenario: Title keyword selects the variant
Given Francesco omits `--variant` for job title "Senior Platform Engineer"
When the CV is built
Then `senior-platform-engineer` is selected

#### Scenario: Manager titles take precedence
Given Francesco omits `--variant` for job title "Engineering Manager - DevOps"
When the CV is built
Then `engineering-manager` is selected

#### Scenario: Unmatched title uses default
Given Francesco omits `--variant` for job title "Accountant"
When the CV is built
Then the configured default variant is used

### Acceptance Criteria

- [ ] Recognized keywords map to the correct variant.
- [ ] Manager-family keywords win over devops/platform/sre.
- [ ] Unmatched titles use the configured default.

### Outcome KPIs

- Who: Francesco | Does what: applies without specifying a variant | By how much: common titles resolve correctly without `--variant` | Measured by: inferred variant vs intended | Baseline: always typed manually.

## US-03: Land the PDF in an organized, predictable location

job_id: apply-with-tailored-cv

### Elevator Pitch

- Before: built PDFs are scattered in the template working copy and hard to find.
- After: run `rusty_cv_creator insert --job-title "Senior DevOps" --company-name "ACME"` → sees `CV saved to: <output_pdf>/2026/2026-06-20-Senior-DevOps-ACME.pdf`; working dir cleaned up.
- Decision enabled: Francesco knows exactly which file to attach.

### Problem

Without a predictable output location and name, finding the right PDF to attach is slow and error-prone.

### Who

- Francesco | after a successful build | wants the PDF filed and named deterministically.

### Solution

Copy the built PDF to `<output_pdf>/<year>/<date>-<job>-<company>.pdf` (and a sibling copy), then remove the dated working directory.

### Domain Examples

1. Happy path — application to ACME on 2026-06-20 for "Senior DevOps" → `.../2026/2026-06-20-Senior-DevOps-ACME.pdf`.
2. Edge case — job title with spaces ("Senior DevOps") → spaces become dashes in the filename.
3. Error/boundary — built PDF missing in the working dir → error, no copy.

### UAT Scenarios (BDD)

#### Scenario: PDF filed under the per-year output directory
Given a successful build for ACME
When `insert` completes
Then a PDF named `<date>-<job>-<company>.pdf` exists under `<output_pdf>/<year>/`

#### Scenario: Working directory is cleaned up
Given a successful build
When `insert` completes
Then the dated working directory is removed and only the PDF copies remain

### Acceptance Criteria

- [ ] Output PDF exists under the configured per-year directory with the deterministic name.
- [ ] Spaces in job/company are sanitized to dashes in the filename.
- [ ] The dated working directory is removed after copy-out.

### Outcome KPIs

- Who: Francesco | Does what: locates the correct PDF to attach | By how much: zero time spent searching working dirs | Measured by: presence of the named file under the output dir | Baseline: manual hunt in template copy.

## US-04: Record the application in the database

job_id: apply-with-tailored-cv

### Elevator Pitch

- Before: Francesco cannot tell whether he already applied to a company/role.
- After: run `rusty_cv_creator insert --job-title "Platform Engineer" --company-name "Stripe" --save-to-database` → log `Saved CV to database`.
- Decision enabled: Francesco avoids duplicate applications and tracks where he applied.

### Problem

Without a record, Francesco loses track of where he has applied.

### Who

- Francesco | applying and wanting an audit trail | opts in to persistence.

### Solution

With `--save-to-database`, persist job title, company, quote, PDF path, and application date to the configured backend.

### Domain Examples

1. Happy path — Stripe / Platform Engineer with `--save-to-database` → record stored.
2. Edge case — no quote provided → record stored with an empty/absent quote.
3. Error/boundary — flag omitted → no DB write; log `CV NOT SAVED TO DATABASE!`.

### UAT Scenarios (BDD)

#### Scenario: Application is recorded when opted in
Given Francesco runs `insert` with `--save-to-database` for Stripe
When `insert` completes
Then a record with job title, company, quote, PDF path and date is persisted

#### Scenario: No record when not opted in
Given Francesco runs `insert` without `--save-to-database`
When `insert` completes
Then no DB write occurs and the tool logs `CV NOT SAVED TO DATABASE!`

### Acceptance Criteria

- [ ] With the flag, a complete application record is persisted.
- [ ] Without the flag, no DB write occurs.

### Outcome KPIs

- Who: Francesco | Does what: keeps a record of applications | By how much: 100% of opted-in runs produce a queryable record | Measured by: DB row count vs opted-in runs | Baseline: no tracking.

## US-05: Preview the generated PDF before sending

job_id: apply-with-tailored-cv

### Elevator Pitch

- Before: Francesco opens the PDF manually to check it rendered correctly.
- After: run `rusty_cv_creator insert --job-title "DevOps Engineer" --company-name "GitLab" --view-generated-cv` → the PDF opens in the configured viewer.
- Decision enabled: Francesco visually verifies the CV before attaching it.

### Problem

Francesco wants to confirm the CV renders correctly without a separate manual step.

### Who

- Francesco | after a build | wants an immediate visual check.

### Solution

With `--view-generated-cv`, open the produced PDF in the configured `[optional] pdf_viewer`; pre-check the viewer is on PATH.

### Domain Examples

1. Happy path — GitLab / DevOps Engineer with `--view-generated-cv` → PDF opens in zathura.
2. Edge case — flag omitted → tool logs the saved path instead of opening a viewer.
3. Error/boundary — viewer not installed → fail fast with a devenv hint (ADR-0004).

### UAT Scenarios (BDD)

#### Scenario: PDF opens in the configured viewer
Given Francesco runs `insert` with `--view-generated-cv` and a successful build
When `insert` completes
Then the produced PDF opens in the configured `pdf_viewer`

#### Scenario: Missing viewer fails fast with guidance
Given the configured viewer is not on PATH
When `insert` runs with `--view-generated-cv`
Then the tool fails fast with a devenv hint rather than a cryptic error

### Acceptance Criteria

- [ ] With the flag and a successful build, the PDF opens in the configured viewer.
- [ ] A missing viewer fails fast with a devenv hint.

### Outcome KPIs

- Who: Francesco | Does what: verifies the CV renders before sending | By how much: visual check in the same run, no separate step | Measured by: viewer launched on opted-in runs | Baseline: manual open.
