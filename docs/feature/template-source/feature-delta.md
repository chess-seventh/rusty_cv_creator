# Feature Delta — template-source

> Wave narrative file (DISCUSS). Density: **LEAN** (Tier-1 `[REF]` only).
> Feature: pull the CV template from **either a local directory or a GitHub
> repository**. Grounds: `src/file_handlers.rs::prepare_path_for_new_cv`
> (current local-only sourcing), `rusty-cv-config-example.ini` (`[cv]
> cv_template_path`), `docs/product/architecture/brief.md` (template repo named
> the highest-risk integration boundary).

---

## Wave: DISCUSS / [REF] Persona ID

`job-seeker` — **Francesco**, senior infrastructure engineer, single user, CLI
expert, comfortable with LaTeX and git. Runs the tool locally on Linux (devenv).
Owns the LaTeX CV template repo `git@github.com:chess-seventh/cv.git` (private).

## Wave: DISCUSS / [REF] JTBD one-liner

> When my CV template lives in a GitHub repo (or I set the tool up on a new
> machine), I want the tool to pull the template from **either** a local
> directory **or** a GitHub repo at a chosen version, so I can generate CVs
> anywhere without manually cloning and babysitting a local copy.

Job: `source-cv-template` (`docs/product/jobs.yaml`). Opportunity score **11**
(importance 7 / satisfaction 3), tier MEDIUM-HIGH. Enabler for the core job
`apply-with-tailored-cv`.

## Wave: DISCUSS / [REF] Locked decisions

| ID | Decision | Verdict |
| ---- | ---------- | --------- |
| D1 | **Source discrimination is auto-detected** from the existing `[cv] cv_template_path` value: a readable local directory → LOCAL; a `git@…`/`https://….git` URL → GITHUB. No new mandatory key; least config churn. | LOCKED |
| D2 | **Both public and private GitHub access**, explicitly configurable. Default is inferred from URL scheme (`git@…` → SSH using the machine's existing keys; `https://…` → anonymous). Optional `cv_template_auth = auto \| ssh \| token` overrides; `token` reads a secret **from an env var (e.g. `GITHUB_TOKEN`)**, never from the INI file (security — brief: "no secrets in repo"). | LOCKED |
| D3 | **Ref pinning to branch / tag / commit SHA** via optional `[cv] cv_template_ref`. Unset → repo default branch HEAD. | LOCKED |
| D4 | **On fetch failure, reuse a cached clone.** Clones are cached per `repo@ref`. Network/auth/url failure with a usable cache → reuse it (warn). No cache → abort fast with an actionable hint (ADR-0004 style). | LOCKED |
| D5 | **Downstream build flow is unchanged.** The feature only changes how the *local template directory* handed to `copy_dir` (in `prepare_path_for_new_cv`) is produced. Variant resolve → `just build` → per-year filing are untouched. | LOCKED |
| D6 | **No new subcommand.** Sourcing is transparent to the existing `insert` entry point; it is config-driven. (A CLI `--template-ref` override is OUT of initial scope — see Out-of-scope.) | LOCKED |
| D7 | **OO paradigm** (CLAUDE.md): introduce a `TemplateSource` trait with `LocalDirectory` and `GitHubRepository` implementors; a resolver returns a local dir. The git mechanism (shell-out `git` vs `git2`/`gitoxide`) is **deferred to a SPIKE/DESIGN decision**, not fixed here. | LOCKED |

## Wave: DISCUSS / [REF] Driving ports

- **CLI** (existing) — `rusty_cv_creator insert --job-title <t> --company-name <c> [--variant <v>]`. Unchanged surface;
  sourcing runs inside `prepare_path_for_new_cv` before `copy_dir`.
- **Configuration (INI)** — `[cv] cv_template_path` (reused, auto-detected), plus optional `cv_template_ref`,
  `cv_template_auth`, `cv_template_cache`. Read via the immutable injected `AppContext` (ADR-0006).
- No HTTP / TUI surface for this feature.

## Wave: DISCUSS / [REF] Pre-requisites

- `git` available on PATH (gated by the existing `ensure_tools_available` pre-usage check, ADR-0004) when the source is
  GITHUB.
- A writable cache directory (`cv_template_cache`, default `~/.cache/rusty-cv-creator/templates`).
- For private repos: working machine git auth (SSH agent/key) or `GITHUB_TOKEN` in env.
- No prior wave artifacts (DISCOVER/DIVERGE absent); requirements grounded in code + architecture brief.

## Wave: DISCUSS / [REF] User stories with elevator pitches

> Story IDs `TS-01..TS-04`; each traces to job `source-cv-template`. Each maps
> 1:1 to a carpaccio slice (see Story map). The `insert` command is the single
> user-invocable entry point; observable output is its stdout/log + the produced
> PDF path.

### TS-01 — Pull a public template repo by URL (walking skeleton)

As Francesco, I set `cv_template_path` to a **public** git URL so the tool clones the template itself instead of
requiring a pre-existing local checkout.

#### Elevator Pitch

Before: the tool only works if I have already `git clone`-d the template to a local dir and pointed `cv_template_path`
at it.
After: set `cv_template_path = https://github.com/chess-seventh/cv.git`, run
`rusty_cv_creator insert --job-title "SRE" --company-name "Acme"` → sees
`✅ Cloning template from https://github.com/chess-seventh/cv.git (default branch)` then
`✅ CV saved to: …/2026/2026-06-22_Acme_SRE.pdf`.
Decision enabled: I can confirm the tool builds from the canonical GitHub template with zero manual git steps.

**Acceptance criteria**

- AC1: Given `cv_template_path` is a `https://….git` URL to a reachable public repo, when I run `insert`, then the
  repo's default branch is cloned into the cache and the CV PDF is produced from it (same path/name contract as today).
- AC2: Given `cv_template_path` is an existing local directory, when I run `insert`, then behaviour is **identical to
  today** (no clone attempted) — backward compatible.
- AC3: Given a value that is neither a readable local dir nor a recognisable git URL, when I run `insert`, then it fails
  fast with a message naming the offending value and the two accepted forms.
- AC4 (production data): AC1 is verified against the real public repo form, not a synthetic fixture.

### TS-02 — Pull a private template repo using existing credentials

As Francesco, I point `cv_template_path` at my **private** repo so the canonical template (which is private) can be
pulled using my machine's existing auth.

#### Elevator Pitch

Before: a private template repo can't be used unless I manually clone it first with my own credentials.
After: set `cv_template_path = git@github.com:chess-seventh/cv.git`, run
`rusty_cv_creator insert --job-title "Platform Eng" --company-name "Acme"` → sees
`✅ Cloning template from git@github.com:chess-seventh/cv.git (ssh auth)` then `✅ CV saved to: …`.
Decision enabled: I confirm my existing SSH key is sufficient — no token plumbing needed for my real workflow.

**Acceptance criteria**

- AC1: Given a `git@…` URL and a working SSH agent/key, when I run `insert`, then the private repo is cloned over SSH
  and the CV is produced.
- AC2: Given `cv_template_auth = token` and `GITHUB_TOKEN` set in env, when I run `insert` with an `https://` private
  URL, then the repo is cloned using the token. The token is **never** read from or written to the INI file.
- AC3: Given a private repo and no usable credentials, when I run `insert`, then it fails fast with an auth-specific
  hint (which auth mode was attempted, how to fix) — distinct from a network error.
- AC4 (production data): AC1 verified against the real private `git@github.com:chess-seventh/cv.git`.

### TS-03 — Pin the template version (branch / tag / commit SHA)

As Francesco, I pin the template to an exact version so a given application reproducibly uses a known-good template.

#### Elevator Pitch

Before: I always get whatever is on the template's default branch — no way to reproduce an older CV shape.
After: set `cv_template_ref = v2.1` (or a tag/branch/SHA), run
`rusty_cv_creator insert --job-title "SRE" --company-name "Acme"` → sees `✅ Checked out template at ref v2.1 (a1b2c3d)`
then `✅ CV saved to: …`.
Decision enabled: I decide exactly which template version a CV was built from, and can reproduce it later.

**Acceptance criteria**

- AC1: Given `cv_template_ref` set to a valid branch, tag, or full/short commit SHA, when I run `insert`, then that
  exact ref is checked out and its resolved SHA is logged.
- AC2: Given `cv_template_ref` unset, when I run `insert`, then the repo default branch HEAD is used (TS-01 behaviour
  preserved).
- AC3: Given a `cv_template_ref` that does not resolve in the repo, when I run `insert`, then it fails fast naming the
  bad ref — and does NOT silently fall back to the default branch.
- AC4 (production data): AC1 verified by building from two different real refs of the actual template repo and observing
  different resolved SHAs in the log.

### TS-04 — Generate offline from a cached template

As Francesco, I generate a CV with no network by reusing the last successfully fetched template, so a flaky connection
or VPN hiccup never blocks an application.

#### Elevator Pitch

Before: any GitHub source means CV generation depends on the network being up at run time.
After: with a populated cache and the network down, run
`rusty_cv_creator insert --job-title "SRE" --company-name "Acme"` → sees
`⚠️ Fetch failed (offline); reusing cached template chess-seventh/cv@main (fetched 2026-06-21)` then `✅ CV saved to: …`.
Decision enabled: I decide to keep applying even while offline, trusting it uses the most recent known-good template.

**Acceptance criteria**

- AC1: Given a prior successful fetch cached for `repo@ref`, when a subsequent `insert` fails to reach GitHub
  (offline/auth/url), then the cached clone is reused and the CV is produced, with a warning naming the cache and its
  fetch date.
- AC2: Given NO cache for the requested `repo@ref` and a fetch failure, when I run `insert`, then it aborts fast with an
  actionable hint (no partial/blank CV produced).
- AC3: Given a successful fetch, when it completes, then the cache for `repo@ref` is created/updated so the next offline
  run (AC1) succeeds.
- AC4 (production data): AC1 verified by fetching the real repo once, then simulating loss of network and re-running.

## Wave: DISCUSS / [REF] Acceptance criteria (summary)

All ACs are embedded per story above. Cross-cutting invariants:

- **Backward compatibility**: an existing local-dir `cv_template_path` must behave exactly as today (TS-01/AC2). This is
  the regression guard for all four slices.
- **No silent wrong-version**: bad ref (TS-03/AC3) and missing-cache failure (TS-04/AC2) abort rather than fall back, to
  protect the "never send a mis-framed CV" social dimension.
- **Secrets never in repo/INI** (TS-02/AC2): tokens come from env only.

## Wave: DISCUSS / [REF] WS strategy

**Strategy B — thin end-to-end skeleton first.** TS-01 (public clone → existing
build) is the walking skeleton: it exercises the entire new path (detect →
clone → cache dir → hand to `copy_dir` → build → file PDF) with the smallest
surface, then TS-02/03/04 thicken auth, ref, and offline independently. Not
strategy D (no env-switching configurable skeleton).

## Wave: DISCUSS / [REF] Story map

**Backbone activity:** *Get the right CV template onto the machine before building.*

Elephant-carpaccio slices (each end-to-end, ≤1 day crafter dispatch, each with a
named learning hypothesis; briefs in `docs/feature/template-source/slices/`):

| Slice | Story | Ships | Learning hypothesis (disproves if it fails) |
| ------- | ------- | ------- | ---------------------------------------------- |
| 01 | TS-01 | Auto-detect + public clone at default branch → existing build (skeleton) | "A `TemplateSource` resolver can turn a git URL into a usable local dir the existing `copy_dir` flow accepts." |
| 02 | TS-02 | Private clone via existing SSH / optional token | "The machine's existing git auth is sufficient; no bespoke token plumbing is needed for the real private repo." |
| 03 | TS-03 | Optional ref pinning (branch/tag/SHA) | "An exact template version can be reproducibly checked out and proven via its resolved SHA." |
| 04 | TS-04 | Cache + offline reuse on fetch failure | "After one successful fetch, CV generation works with no network." |

**Carpaccio taste tests:** (a) no slice ships 4+ new components — each adds one
behaviour onto the slice-01 resolver; (b) the new abstraction (`TemplateSource`)
ships **inside** slice 01 with a working concrete impl, not as a standalone
plumbing slice; (c) each slice disproves a distinct pre-commitment (above);
(d) every slice has a production-data AC against the real repo; (e) no two slices
are scale-duplicates. **All taste tests pass.**

**Scope Assessment: PASS (with mandated slicing).** Single bounded context
(template sourcing), single persona, ≤4 stories. The four-outcome breadth is
handled by carpaccio slicing, not by splitting into separate features. Slice 01
is independently shippable and dogfoodable the same day.

## Wave: DISCUSS / [REF] Definition of Done (9-item)

1. All four slices' ACs are covered by executable acceptance tests (authored in DISTILL).
2. Backward-compat regression (local-dir path unchanged) is green.
3. `TemplateSource` trait + `LocalDirectory` + `GitHubRepository` implementors landed (OO paradigm, CLAUDE.md).
4. Per-feature mutation testing run on the new `TemplateSource` module (CLAUDE.md mutation strategy).
5. Config keys (`cv_template_ref`, `cv_template_auth`, `cv_template_cache`) documented in
   `rusty-cv-config-example.ini` + README.
6. No secret ever read from / written to INI; token path uses env only (verified by test).
7. Fetch-failure paths (cache reuse, hard abort) have explicit tests, not just happy path.
8. `git` pre-usage check (ADR-0004) wired for the GITHUB source.
9. An ADR records the deferred git-mechanism choice (shell-out vs `git2`/`gitoxide`) made in SPIKE/DESIGN.

## Wave: DISCUSS / [REF] Out-of-scope

- The git-fetch mechanism choice (shell-out `git` vs `git2`/`gitoxide`) — a SPIKE/DESIGN decision, not DISCUSS.
- A `--template-ref` / `--template-path` **CLI override** (config-first for now; can be a later slice).
- Non-GitHub remotes (GitLab, Bitbucket, arbitrary git hosts) — URL detection may incidentally work, but only GitHub is
  a committed acceptance target.
- Sparse-checkout / partial-clone performance optimisation.
- Cache eviction / GC policy (cache grows unbounded for now; note for DESIGN).
- Templating placeholder changes — unrelated to sourcing.

## Wave: DISCUSS / [REF] Outcome KPIs

| KPI | Baseline | Target | Measurement |
| ----- | ---------- | -------- | ------------- |
| Manual git commands to first CV on a fresh machine | ≥1 (`git clone`) + path config | **0** git commands (one config value) | Setup walkthrough on a clean checkout |
| Stale-template incidents (CV built from out-of-date local copy) | possible & silent | **0** (canonical pull, pinnable) | TS-03 reproducibility test + log of resolved SHA |
| Offline CV-generation success after first fetch | 0% (network-required) | **100%** | TS-04/AC1 test (cache reuse with network down) |
| Per-run clone latency overhead after first fetch | n/a | **≈0** (cache hit, no re-clone on unchanged `repo@ref`) | Timed second run vs first run |

## Wave: DISCUSS / [REF] DoR validation (9 items)

| # | DoR item | Status | Evidence |
| --- | ---------- | -------- | ---------- |
| 1 | Job traceability | ✅ | All TS-01..04 → job `source-cv-template` in `jobs.yaml` |
| 2 | Persona identified | ✅ | `job-seeker` / Francesco |
| 3 | Stories in LeanUX format w/ elevator pitch | ✅ | Each story has Before/After/Decision triplet against the real `insert` command |
| 4 | ACs testable & unambiguous | ✅ | Each AC is a Given/When/Then with observable output; production-data AC per story |
| 5 | Journey mapped | ✅ | `docs/product/journeys/source-cv-template.yaml` (feeds `generate-cv`) |
| 6 | Walking skeleton defined | ✅ | TS-01 / slice 01 |
| 7 | Slices ≤1 day w/ learning hypotheses | ✅ | Story map + 4 slice briefs; taste tests pass |
| 8 | Outcome KPIs w/ numeric targets | ✅ | KPI table above |
| 9 | Out-of-scope explicit | ✅ | Out-of-scope section |

**Requirements completeness: 0.96** (> 0.95 gate). Residual 0.04: the git
mechanism is intentionally deferred to SPIKE/DESIGN (DoD item 9).

## Wave: DISCUSS / [REF] Wave decisions summary

- **D1–D7** locked (table above). Primary need: remove the manual-pre-clone
  precondition and stale-template risk; support local + GitHub (public/private),
  ref-pinned, cache-backed/offline. Feature type: backend with config/CLI
  surface. WS scope: TS-01 public-clone skeleton.
- **Constraints:** downstream build flow unchanged (D5); secrets env-only (D2);
  no silent version/cache fallback (TS-03/AC3, TS-04/AC2); `git` gated by
  ADR-0004 pre-usage check.
- **Upstream changes:** none — no DISCOVER/DIVERGE existed; SSOT extended
  (new job + new journey), nothing contradicted.

## Wave: DISCUSS / [REF] Handoff

**To DESIGN (nw-solution-architect):** design the `TemplateSource` abstraction
and resolver; choose the git mechanism via a SPIKE first (network I/O is new to
this codebase — recommend `/nw-spike template-source` before `/nw-design`).
**To DEVOPS (nw-platform-architect):** `outcome-kpis` above (offline success,
zero-manual-git, reproducibility) for instrumentation.
**Recommended next command:** `/nw-spike template-source` (de-risk git
mechanism), then `/nw-design template-source`.
