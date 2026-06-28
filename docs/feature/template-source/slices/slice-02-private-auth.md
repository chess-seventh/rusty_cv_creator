# Slice 02 — Private repo auth (TS-02)

**Goal:** Pull a private GitHub repo using the machine's existing SSH key, or a token from env for HTTPS.

**Story:** TS-02

## IN scope

- SSH clone (`git@…`) using existing agent/key — the real workflow for `git@github.com:chess-seventh/cv.git`.
- Optional `cv_template_auth = auto | ssh | token`; `token` reads `GITHUB_TOKEN` from env for `https://` private URLs.
- Auth-specific fast-fail hint distinct from network errors.

## OUT scope

- Ref pinning (slice 03), caching/offline (slice 04).
- Storing any secret in INI (forbidden — env only).

## Learning hypothesis

Disproves **"the machine's existing git auth is sufficient; no bespoke token plumbing is needed"** if the real private
repo cannot be cloned with the existing SSH key.
Confirms auth is a thin add over slice 01.

## Acceptance criteria

TS-02 AC1–AC4. Production data: clone the real private `git@github.com:chess-seventh/cv.git`.

## Dependencies

Slice 01 (`GitHubRepository` impl + resolver).

## Effort / reference class

≤1 day. Mostly auth-mode branching + error mapping on the slice-01 clone path.

## Dogfood moment

Same day: generate a CV sourced from the real private repo.
