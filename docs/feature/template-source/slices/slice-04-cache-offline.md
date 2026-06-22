# Slice 04 — Cache + offline reuse (TS-04)

**Goal:** Cache clones per `repo@ref`; on fetch failure reuse the cache so CV generation works offline. No cache → hard
fast-fail.

**Story:** TS-04

## IN scope

- Cache keyed by `repo@ref` under `cv_template_cache` (default `~/.cache/rusty-cv-creator/templates`).
- Successful fetch creates/updates the cache.
- Fetch failure (offline/auth/url) WITH usable cache → reuse it, warn (name cache + fetch date).
- Fetch failure WITHOUT cache → abort fast, actionable hint, no partial/blank CV.

## OUT scope

- Cache eviction / GC policy (note for DESIGN; unbounded for now).
- Concurrent-run cache locking (single-user tool).

## Learning hypothesis

Disproves **"after one successful fetch, CV generation works with no network"** if cache reuse cannot produce a
build-ready template offline.
Confirms the offline-success KPI (target 100%).

## Acceptance criteria

TS-04 AC1–AC4. Production data: fetch the real repo once, drop network, re-run.

## Dependencies

Slice 01 (clone + cache dir). Composes with slices 02/03 (cache key includes ref).

## Effort / reference class

≤1 day. Cache-key + presence check + failure→reuse branch + tests for both failure paths.

## Dogfood moment

Same day: generate a CV, disable network, generate again from cache.
