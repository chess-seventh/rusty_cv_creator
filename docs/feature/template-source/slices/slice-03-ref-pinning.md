# Slice 03 — Ref pinning (TS-03)

**Goal:** Optional `cv_template_ref` selects an exact branch / tag / commit SHA; the resolved SHA is logged.

**Story:** TS-03

## IN scope

- `[cv] cv_template_ref` optional config; checkout of branch, tag, or full/short SHA.
- Log the resolved commit SHA for reproducibility.
- Unset → default branch HEAD (slice 01 behaviour preserved).
- Unresolvable ref → fast-fail naming the bad ref; **no** silent fallback to default branch.

## OUT scope

- Caching/offline (slice 04); CLI `--template-ref` override.

## Learning hypothesis

Disproves **"an exact template version can be reproducibly checked out and proven via its resolved SHA"** if SHA/tag
checkout does not yield a deterministic build input.
Confirms reproducibility — feeds the "0 stale-template incidents" KPI.

## Acceptance criteria

TS-03 AC1–AC4. Production data: build from two different real refs, observe different resolved SHAs.

## Dependencies

Slice 01 (clone path). Independent of slice 02 (works for public or private).

## Effort / reference class

≤1 day. Checkout-after-clone + ref resolution + error path.

## Dogfood moment

Same day: pin to a tag and generate a CV; confirm the logged SHA.
