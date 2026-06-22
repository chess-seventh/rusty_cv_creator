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
| Per-run clone latency overhead after first fetch | ~3.5s cold clone | **≤1.7s** cache `fetch`+`checkout` (SPIKE-measured); **≤200ms** offline reuse with no `fetch` | Timed second run vs first run |

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

---

## Wave: DESIGN / [REF] Bounded context and ubiquitous language

Single **supporting** bounded context — *template sourcing* — feeding the core
job `apply-with-tailored-cv`. It has no DB aggregate: the only persistent state
is the on-disk **cache** (a side-effect universe, not a domain entity). Language:
*source* (local dir or git URL), *resolve* (produce a local template dir),
*ref* / *pin* (branch/tag/SHA), *cache entry* (`repo@ref`), *fetch-vs-clone*,
*reuse* (offline), *auth mode* (`auto|ssh|token`). The context boundary is the
`TemplateSource::resolve` call inside `prepare_path_for_new_cv`; everything
downstream (`copy_dir` → build → filing) is unchanged (D5).

## Wave: DESIGN / [REF] Component decomposition

| Component | Path | Responsibility | Contract shape |
| --- | --- | --- | --- |
| `TemplateSource` (trait, port) | `src/template_source.rs` | Resolve a source to a local template dir | return-only |
| `LocalDirectory` (adapter) | `src/template_source.rs` | Passthrough of an existing local dir (backward compat) | pure (return-only) |
| `GitHubRepository` (adapter) | `src/template_source.rs` | Clone/fetch/checkout a git URL at a ref into the cache | bounded-change: cache dir only |
| `TemplateCache` (collaborator) | `src/template_source.rs` | Cache-key derivation + reuse-vs-fetch-vs-abort decision + offline fallback | decision pure; executor bounded to cache dir |
| `TemplateSourceError` (enum) | `src/template_source.rs` | Typed failure classes → distinct actionable hints | return-only |
| `AuthMode` / auth resolution | `src/template_source.rs` | Map `auto/ssh/token` + URL → git invocation plan | pure (return-only) |
| `detect_template_source` / `is_git_url` (use case) | `src/template_source.rs` | Auto-detect LOCAL vs GITHUB (D1); build the source | pure (return-only) |
| `CommandRunner` (port, EXTEND) | `src/command_runner.rs` | Subprocess effects + new stderr-capturing run | effect boundary |
| `ensure_tools_available` (REUSE) | `src/helpers.rs` | Pre-usage `git` PATH gate (ADR-0004) | effect (read PATH) |

## Wave: DESIGN / [REF] Driving and driven ports

- **Driving (inbound):** unchanged CLI `insert` (D6); sourcing runs transparently
  inside `prepare_path_for_new_cv`. Config (INI) is the second driving surface:
  reused `cv_template_path` plus optional `cv_template_ref`, `cv_template_auth`,
  `cv_template_cache`, read via the immutable `AppContext` (ADR-0006).
- **Driven (outbound):** `TemplateSource` (new driven port — resolve to a local
  dir); `CommandRunner` (EXTENDED — `git` shell-out, now also capturing stderr to
  classify failures); filesystem (cache dir, capability-scoped to
  `cv_template_cache`); environment (`GITHUB_TOKEN`, read only at execution).
- **Effect isolation (principle 12):** the resolver core is split so decisions are
  pure values — `is_git_url`, cache `CacheAction` (Clone | FetchCheckout |
  ReuseStale | Abort), auth plan, and error classification are return-only pure
  functions; only `GitHubRepository::resolve` and the cache executor perform
  effects, and their mutation universe is **bounded to the cache dir**. A
  read-only source (`LocalDirectory`) exposes no write path. The git driven
  dependency is probed presence-first via `ensure_tools_available(["git"])`
  (ADR-0004); its real failure stderr is the substrate that the classifier must
  survive, enforced by gold-tests (catalogued git stderr strings) plus the
  existing real-`git` acceptance test against a `file://` fixture.

## Wave: DESIGN / [REF] Technology choices

| Choice | Selection | Rationale | License |
| --- | --- | --- | --- |
| Git mechanism | System `git` via `CommandRunner` (ADR-0008) | SPIKE-proven; zero new crate; fake-able | n/a (runtime tool) |
| Error model | Hand-rolled `TemplateSourceError` enum (std only) | Distinct hints must be structural, not strings; no new dep | n/a |
| Token transport | `GITHUB_TOKEN` env via `git -c core.askpass=…` | Secret never in INI / argv / cache `.git/config` | n/a |
| Cache strategy | `fetch`+`checkout` keyed by `repo@ref`; no shallow | Real perf lever (1.7s vs 3.5s); assets dominate size | n/a |

No new crate is added; this stays within the existing std-only error idiom.

## Wave: DESIGN / [REF] Reuse analysis (HARD GATE)

| Component | Verdict | Justification |
| --- | --- | --- |
| `TemplateSource` trait | EXTEND | Skeleton exists; change `Result` error type to `TemplateSourceError`; signature otherwise stable |
| `LocalDirectory` | EXTEND | Behaviour unchanged; only the error type changes |
| `GitHubRepository` | EXTEND | Add ref pinning (TS-03), cache reuse/offline (TS-04), auth (TS-02), failure classification |
| `detect_template_source` / `is_git_url` | EXTEND | Thread `cv_template_ref` + `cv_template_auth` into construction; signature grows |
| `CommandRunner` port | EXTEND | Add a stderr-capturing method (additive, backward-compatible) to classify git failures |
| `SystemRunner` / `FakeRunner` | EXTEND | Implement the new method; `FakeRunner` gains canned stderr for gold-tests |
| `ensure_tools_available` (ADR-0004) | REUSE | `git` already gated in the skeleton; no change |
| `resolve_template_cache_dir` (file_handlers) | REUSE | Already reads `cv_template_cache` with the default path |
| `prepare_path_for_new_cv` wiring | EXTEND | Read the two new INI keys and pass them into `detect_template_source` |
| `AppContext` / `get_variable_from_config_file` | REUSE | New INI keys read via the existing accessor; no new API |
| `TemplateCache` | CREATE NEW | No existing cache-lifecycle component; SRP-isolated, fake-able |
| `TemplateSourceError` | CREATE NEW | No existing typed error; required for distinct hints |
| `AuthMode` / auth resolution | CREATE NEW | No existing auth modelling |

## Wave: DESIGN / [REF] Decisions table

> All four decisions below were **CONFIRMED by the user on 2026-06-22**
> (recommended option accepted in each case).

| ID | Decision | Choice (confirmed) — alternative considered | ADR |
| --- | --- | --- | --- |
| TS-D1 | Error model | `TemplateSourceError` enum, std-only (vs `thiserror`, vs `Box<dyn Error>` strings) | ADR-0008 |
| TS-D2 | Cache responsibility | Separate `TemplateCache` collaborator with a pure decision fn (vs all-in-`resolve`) | ADR-0008 |
| TS-D3 | Auth transport | `GITHUB_TOKEN` env + `core.askpass` helper (vs URL-embedded token) | ADR-0008 |
| TS-D4 | ADR-0004 PATH-check injectability | DEFER; keep `#[serial]`, record smell (vs inject a `ToolChecker` port now) | ADR-0004 |

## Wave: DESIGN / [REF] Confirmed decisions (user-approved 2026-06-22)

> Status: all four **CONFIRMED** (recommended option accepted). Rationale and the
> alternatives weighed are retained below for DELIVER traceability.

1. **Error model — Confirmed: a std-only `TemplateSourceError` enum** (vs
   `thiserror`, vs keeping `Box<dyn Error>` + `format!`). The ACs demand
   *distinct* hints for auth vs network/offline vs bad-ref vs no-cache; that
   distinction must be a structural `match`-able value, not a string. Hand-rolling
   keeps the repo's zero-error-crate idiom; adopt `thiserror` only if boilerplate
   grows. **Note:** classification requires git's stderr, which forces the
   `CommandRunner` extension below.
2. **Cache responsibility — Confirmed: a `TemplateCache` collaborator whose
   reuse-vs-fetch-vs-abort decision is a pure `CacheAction` function** (vs
   everything inside `GitHubRepository::resolve`). Keeps `resolve` thin, makes the
   four cache behaviours (clone-fresh, fetch-reuse, offline-reuse-stale,
   no-cache-abort) independently fake-testable, and renders "a preview silently
   wrote to disk" non-representable (the decision returns data).
3. **Auth transport — Confirmed: `auto` inferred from URL scheme; `token` reads
   `GITHUB_TOKEN` from env and feeds git via `git -c core.askpass=<helper>`** (vs
   embedding the token in the `https://x-access-token:…@…` URL). The askpass route
   keeps the secret off the argv (`ps`), out of the INI, and out of the cache
   repo's persisted `.git/config` remote URL. SSH (`git@…`) inherits the agent
   with no flags (SPIKE-proven).
4. **ADR-0004 PATH-check injectability — Confirmed: DEFER; keep `#[serial]` on
   the git-touching tests and record the coupling as a known smell** (vs
   introducing a `ToolChecker` port now). A port is a cross-cutting refactor that
   also touches the DB/Tailscale path — a >50% solution for a <10% problem and
   scope-creep beyond template-source; the skeleton already proves `#[serial]`
   works. Cheap upgrade path if friction appears: a module-local injection seam
   (a `tool_check` closure parameter defaulting to `ensure_tools_available`).

## Wave: DESIGN / [REF] Open questions / deferred

- Cache GC/eviction policy (unbounded growth) — out-of-scope per DISCUSS; future
  concern.
- `ToolChecker` port extraction to drop `#[serial]` — deferred (TS-D4); candidate
  for a focused follow-up ADR.
- A `--template-ref` CLI override — out-of-scope per DISCUSS (config-first).

---

## Wave: DISTILL / [REF] Authoring note

> Scaffolded RED per ADR-025 (DISTILL is the canonical AT author). The four
> `.feature` files under `tests/acceptance/template-source/` are the human-readable
> scenario SSOT (the project has **no cucumber-rust harness**), mapped to concrete
> Rust tests via the Traceability table below. The already-GREEN walking skeleton
> (`src/file_handlers.rs::walking_skeleton_github_source_resolves_template_dir`) is
> referenced, not duplicated or retagged. Density: **LEAN**. Reconciliation gate:
> **passed — 0 contradictions** across DISCUSS / DESIGN (no DEVOPS wave exists).

## Wave: DISTILL / [REF] Inherited commitments

| Origin | Commitment | DDD | Impact |
| --- | --- | --- | --- |
| DISCUSS#D1 | Source is auto-detected from `cv_template_path` (readable dir → LOCAL, git URL → GITHUB). | n/a | Drives TS-01 detection scenarios; pure `is_git_url` property test. |
| DISCUSS#D2 | Public + private access; token from env only, never the INI. | TS-D3 | TS-02 auth scenarios assert the token never reaches the git argv. |
| DISCUSS#D3 | Ref pinning via `cv_template_ref`; unset → default branch. | n/a | TS-03 pinned-checkout + bad-ref-no-fallback scenarios. |
| DISCUSS#D4 | On fetch failure reuse cache (warn); no cache → abort fast. | TS-D2 | TS-04 `CacheAction` matrix (reuse-vs-fetch-vs-abort). |
| DESIGN#UC-1 | `CommandRunner` gains an additive stderr-capturing run. | ADR-0008 | Needed to classify auth vs network vs bad-ref; scaffolded as `run_capturing`. |
| DESIGN#TS-D1 | Each failure is a distinct `TemplateSourceError` variant. | ADR-0008 | Hints are `match`-able, not string-matched; one variant per error path. |

## Wave: DISTILL / [REF] Scenario list with tags

16 scenarios across 4 documentation feature files. 9 error/edge (**56%**).

| # | Feature file | Scenario | Tags |
| --- | --- | --- | --- |
| 1 | public-source | Francesco sources a public template by URL end to end | @walking_skeleton @driving_port @US-01 @real-io @contract-shape:bounded-change |
| 2 | public-source | An existing local directory is used exactly as before | @US-01 @edge @real-io @contract-shape:unbounded-preservation |
| 3 | public-source | A value that is neither a directory nor a git URL is refused | @US-01 @error @driving_adapter @real-io @contract-shape:pure-function |
| 4 | public-source | Recognised URL forms are detected as git sources | @US-01 @property @contract-shape:pure-function |
| 5 | public-source | A bare token or local path is not mistaken for a git URL | @US-01 @property @edge @contract-shape:pure-function |
| 6 | private-source | A private SSH source clones over its git@ URL | @US-02 @in-memory @contract-shape:bounded-change |
| 7 | private-source | A token is taken from the environment and never on the command line | @US-02 @error @in-memory @contract-shape:unbounded-preservation |
| 8 | private-source | An authentication failure is reported with an auth-specific hint | @US-02 @error @in-memory @contract-shape:unbounded-preservation |
| 9 | pinned-version | A pinned version is checked out and its resolved revision is logged | @US-03 @in-memory @contract-shape:bounded-change |
| 10 | pinned-version | With no version pinned the default branch is used | @US-03 @contract-shape:bounded-change |
| 11 | pinned-version | An unknown version is refused without falling back | @US-03 @error @in-memory @contract-shape:unbounded-preservation |
| 12 | offline-cache | Offline, the most recent cached template is reused with a warning | @US-04 @error @in-memory @contract-shape:unbounded-preservation |
| 13 | offline-cache | With no cache and no network the run aborts without a partial CV | @US-04 @error @in-memory @contract-shape:unbounded-preservation |
| 14 | offline-cache | A successful fetch refreshes the cache for next time | @US-04 @in-memory @contract-shape:bounded-change |
| 15 | offline-cache | The reuse-or-fetch-or-abort decision is total over cache and network state | @US-04 @property @contract-shape:pure-function |
| 16 | offline-cache | A repository and version map to one deterministic cache entry | @US-04 @property @edge @contract-shape:pure-function |

## Wave: DISTILL / [REF] Walking-Skeleton Strategy

Per the Architecture of Reference, the driving port is the CLI (`insert`). The
single `@walking_skeleton @driving_port` scenario (#1) maps to the **existing
GREEN** in-crate skeleton, which performs a real `git clone` from a `file://`
bare-repo fixture (real `SystemRunner`, real filesystem, real `copy_dir`) — no
network, no mock of the git layer. DISTILL adds the next layer (auth, ref,
offline, detection) on top; it does not rewrite the skeleton.

## Wave: DISTILL / [REF] Adapter coverage table

| Adapter (driven) | Treatment | Covered by | Real-IO boundary note |
| --- | --- | --- | --- |
| `CommandRunner` — git (`clone`/`fetch`/`checkout`/`-c core.askpass`) | `FakeRunner` @in-memory (asserts the exact git command string) **and** real `SystemRunner` @real-io | scenarios 6, 7, 9 (FakeRunner) + scenario 1 (real `file://` clone) | The `@real-io` git path uses a local bare repo via `file://` (real shell-out, no network), exactly the skeleton's trick. FakeRunner unit specs assert `git clone git@…` / `checkout v2.1` / `core.askpass` strings. |
| Filesystem (cache dir + working copy) | **real I/O** via `tempfile::TempDir` | scenarios 1, 2 | Genuine real-IO adapter on an isolated tmp tree (capability-scoped to the cache dir). |
| Environment (`GITHUB_TOKEN`) | read at execution; secret-handling asserted structurally | scenario 7 | Not faked; `auth_invocation_flags(Token, …)` is a pure fn asserted to route via `core.askpass` and to keep the token off the argv. |

The `git` driven adapter therefore has **≥1 `@real-io` scenario** (scenario 1,
real `file://` clone) **and** FakeRunner unit specs (scenarios 6/7/9) —
satisfying Mandate 6.

## Wave: DISTILL / [REF] Scaffolds

RED scaffolds (Mandate 7, Rust): the crate **compiles** and every un-ignored new
test is GREEN; every `#[ignore]`d pending spec fails by `panic!`, never by a
compile/import error. Markers: `// SCAFFOLD: true`. ADDITIVE ONLY — the green
skeleton signatures are untouched.

| File | Symbol | Kind |
| --- | --- | --- |
| `src/command_runner.rs` | `CommandOutcome` | struct (UC-1 carrier) |
| `src/command_runner.rs` | `CommandRunner::run_capturing` | additive trait method (default panics) |
| `src/template_source.rs` | `AuthMode` (`Auto`/`Ssh`/`Token`) + `AuthMode::from_config` | enum + parser |
| `src/template_source.rs` | `auth_invocation_flags` | pure fn (askpass plan) |
| `src/template_source.rs` | `TemplateSourceError` (`Auth`/`NetworkOffline`/`BadRef`/`NoCache`/`BadValue`) + `Display`/`Error` | typed error enum (TS-D1) |
| `src/template_source.rs` | `classify_git_stderr` | pure fn (UC-1 stderr → error class) |
| `src/template_source.rs` | `CacheAction` (`Clone`/`FetchCheckout`/`ReuseStale`/`Abort`) | pure decision enum (TS-D2) |
| `src/template_source.rs` | `TemplateCache` + `new`/`cache_key`/`decide` | cache collaborator |
| `src/template_source.rs` | `GitHubRepository::with_ref`/`with_auth`/`resolve_classified` | additive builders + classified resolve |

## Wave: DISTILL / [REF] Test placement

- **Documentation SSOT**: `tests/acceptance/template-source/*.feature` (4 files) —
  precedent: `tests/acceptance/cv-variant-build/`.
- **Driving-adapter / subprocess scenarios**: `tests/template_source_scenarios.rs`
  (external integration crate) — precedent: `tests/tui_job_applications_scenarios.rs`
  - `tests/cli_smoke.rs` (`env!("CARGO_BIN_EXE_rusty_cv_creator")` + `tempfile`).
- **Unit-level specifications**: **in-crate** `#[cfg(test)] mod distill_specs` in
  `src/template_source.rs` and `mod uc1_specs` in `src/command_runner.rs`. They
  live in-crate (NOT in a `tests/template_source_specifications.rs` file) because
  `TemplateSource`, the scaffolds, and `command_runner::testing::FakeRunner` are
  **binary-private** — they are not on the `lib.rs` facade (only
  `database`/`models`/`schema`/`tui` are), and `helpers` (a transitive dependency)
  references the `main.rs`-local `is_tailscale_connected`, so exposing them via the
  library would cascade. This matches the existing green precedent: the skeleton's
  own FakeRunner specs already live in `src/template_source.rs::tests`. Recorded in
  `distill/upstream-issues.md`.

## Wave: DISTILL / [REF] Driving Adapter coverage

The sole driving adapter is the `insert` CLI subcommand (clap, D6 — sourcing is
transparent). Scenario 3 (`ts01_ac3_bad_template_value_fails_fast_naming_the_value`)
spawns the **built binary** and asserts a non-zero exit naming the offending
value — a real subprocess driving-adapter test, not an orchestration-layer call.
The git happy path is driven at the `create_directory` seam by the in-crate
walking skeleton (binary-private symbols unreachable from an external crate).

## Wave: DISTILL / [REF] Traceability (AC → scenario → test)

| AC | Scenario | Test (file::name) | State |
| --- | --- | --- | --- |
| TS-01/AC1 | 1 | `src/file_handlers.rs::walking_skeleton_github_source_resolves_template_dir` | GREEN (skeleton) |
| TS-01/AC2 | 2 | `src/template_source.rs::tests::test_detect_existing_dir_is_local` (+ `…test_local_directory_resolves_to_passthrough_path`) | GREEN |
| TS-01/AC3 | 3 | `tests/template_source_scenarios.rs::ts01_ac3_bad_template_value_fails_fast_naming_the_value` (+ unit `…test_detect_unrecognised_value_errors_naming_value`) | GREEN |
| TS-01/D1 | 4, 5 | `src/template_source.rs::distill_specs::ts01_is_git_url_classifies_known_forms` | GREEN (proptest) |
| TS-02/AC1 | 6 | `…distill_specs::ts02_ac1_ssh_source_clones_via_git_at_url` | PENDING (RED) |
| TS-02/AC2 | 7 | `…distill_specs::ts02_ac2_token_uses_askpass_and_never_on_argv` | PENDING (RED) |
| TS-02/AC3 | 8 | `…distill_specs::ts02_ac3_auth_failure_stderr_classified_as_auth` (+ `command_runner.rs::uc1_specs::uc1_run_capturing_exposes_stderr` for UC-1) | PENDING (RED) |
| TS-03/AC1 | 9 | `…distill_specs::ts03_ac1_pinned_ref_is_checked_out` | PENDING (RED) |
| TS-03/AC2 | 10 | skeleton default-branch resolve (`…walking_skeleton…`) | GREEN (reference) |
| TS-03/AC3 | 11 | `…distill_specs::ts03_ac3_bad_ref_classified_no_silent_fallback` | PENDING (RED) |
| TS-04/AC1 | 12 | `…distill_specs::ts04_cache_action_matrix` (`ReuseStale`) | PENDING (RED) |
| TS-04/AC2 | 13 | `…distill_specs::ts04_cache_action_matrix` (`Abort`) | PENDING (RED) |
| TS-04/AC3 | 14, 15 | `…distill_specs::ts04_cache_action_matrix` (`Clone`/`FetchCheckout`) | PENDING (RED) |
| TS-04 (key) | 16 | `…distill_specs::ts04_cache_key_is_deterministic` | PENDING (RED) |

## Wave: DISTILL / [REF] Self-Completeness Audit (Phase 2.5)

`nw-at-completeness-check` over the 16 scenarios. Verdict:
**ACCEPTABLE_WITH_DOCUMENTED_GAPS** — happy/error/edge/property coverage for every
AC; all four driven behaviours (detect, auth, ref, cache) have explicit error
paths (56% error/edge). All gaps are `AT_GAP_IN_DELIVERY_SCOPE` (filled by the
pending specs in DELIVER); **zero `SPECIFICATION_AMBIGUITY`** blockers — DISCUSS
ACs and DESIGN ports/decisions fully determine every scenario.

## Wave: DISTILL / [REF] Mandate-12 (SSOT) note — informational

Rust + no cucumber-rust harness ⇒ no `pytest-bdd`-style step decorators; the
DSL/step-reuse-ratio metric does not apply. Domain concepts ARE expressed once via
the type system (criterion 1): `AuthMode`, `CacheAction`, `TemplateSourceError`,
`TemplateCache` are typed enums/structs the specs reuse. Step-reuse-ratio is N/A
for this harness shape (config/decision-shaped feature). Criteria 1–3 are met in
spirit; criterion 4 (ratio) is not measured (no decorator surface).

## Wave: DISTILL / [REF] Pre-requisites

- DESIGN driving port (CLI `insert`) + driven ports (`TemplateSource`,
  `CommandRunner` incl. UC-1, filesystem cache, environment) per the DESIGN [REF]
  sections + ADR-0008.
- `git`/`just`/`tectonic` on PATH (ADR-0004 pre-usage checks) for the subprocess
  scenario — satisfied under `devenv shell`.
- No DEVOPS environment matrix exists; sensible defaults applied (single-user local
  CLI; `tempfile::TempDir`; `file://` bare repo for deterministic git real-IO). Not
  a blocker.
- Outcomes registry: **N/A — deferred** (no `docs/product/outcomes/registry.yaml`
  in this project; not bootstrapped per orchestrator instruction).
