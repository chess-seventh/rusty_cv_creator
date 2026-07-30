# RCA — plaintext Postgres password in the tracked tree + three-way host/database drift

- **Analyst**: Rex (root cause analysis), Toyota 5 Whys, depth 5, multi-causal
- **Date**: 2026-07-30
- **Repo / branch**: `rusty_cv_creator` @ `feat/l74-db-creds`
  (worktree `/home/seventh/src/claude-worktrees/rusty_cv_creator/l74-db-creds`)
- **Configuration**: `investigation_depth: 5`, `multi_causal: true`, `evidence_required: true`

---

## 1. Scope and problem statement

Two faults, one investigation pass.

**Fault (a) — credential exposure.** A plaintext PostgreSQL password (`REDACTED-SEE-SOPS`,
user `rusty_cv`) is committed in the tracked tree at two sites carrying the identical
literal:

- `devenv.nix:25` — `env.DATABASE_URL = "postgres://rusty_cv:REDACTED-SEE-SOPS@nixos-03.caracara-palermo.ts.net/db_rusty_cv"`
- `rusty-cv-config-example.ini:64` — `db_pg_host = "postgres://rusty_cv:REDACTED-SEE-SOPS@nixos-01.caracara-palermo.ts.net/db_rusty_cv"`

**Fault (b) — configuration drift.** The `(host, database)` pair exists in three
copies that disagree:

| Copy | Host | Database | Tracked? |
| --- | --- | --- | --- |
| `devenv.nix:25` | `nixos-03` | `db_rusty_cv` | yes |
| `rusty-cv-config-example.ini:64` | `nixos-01` | `db_rusty_cv` | yes |
| `~/.config/rusty-cv-creator/rusty-cv-config.ini` (live) | `nixos-02` | `rusty_cv` | no (outside repo; taken as given, not read) |

**In scope**: credential handling and DB-target resolution in this repo, its
pre-commit hook set, its architecture claims, and its tracked test fixtures.

**Out of scope**: the live INI file's contents (given, not inspected), the parallel
nixos-02 database-recreation lane, secret rotation on the Postgres server itself.

**Impact classification** (per investigation-techniques): *Operational → config drift*
compounded by *System failure → security/access controls*. Severity is bounded by the
repo being private and the host being Tailscale-only, but the password is in git
history and must be treated as burned regardless of the fix.

---

## 2. Timeline (evidence-backed)

| Date | Commit | Event | Evidence |
| --- | --- | --- | --- |
| 2024-07-28 | `53ceda1` | Password literal enters the tracked tree via `rusty-cv-config-example.ini:64`, host `nixos-01`, db `db_rusty_cv`. | `git blame -L 64,64 rusty-cv-config-example.ini` → `53ceda14 (Chess7th 2024-07-28)` |
| 2025-08-19 | `fef6db6` | Second copy of the same literal enters `devenv.nix`, already pointing at a *different* host (`nixos-03`). Drift begins here. | `git show fef6db6:devenv.nix \| grep DATABASE_URL` → line 9, `…@nixos-03…/db_rusty_cv` |
| 2026-02-12 | `29bfaf5` | `devenv.nix:25` reformatted (nixfmt/treefmt), literal unchanged. | `git blame -L 25,25 devenv.nix` |
| (undated) | — | Live INI moves to `nixos-02` / `rusty_cv`. Neither tracked copy follows. | Given by the maintainer |
| 2026-07-30 | — | Fault detected. Neither tracked copy has ever been reconciled: `git log -S'REDACTED-SEE-SOPS'` returns exactly 3 commits across ~2 years. | `git log -S'REDACTED-SEE-SOPS' --all` |

Elapsed exposure: **~24 months** for the INI copy, **~11 months** for the devenv copy.

---

## 3. Hypothesis verdict (asked explicitly)

> **Hypothesis**: `check_if_db_env_is_set_or_set_from_config` prefers an already-set
> `DATABASE_URL` over the INI value; because `devenv.nix` exports `DATABASE_URL` into
> every dev shell, the hardcoded URL silently wins there, so nobody ever exercised the
> INI value and the three copies drifted apart unnoticed.

### Verdict: **PARTLY RIGHT — correct about the precedence inside that one function, wrong about its consequence, and it points at the wrong engine.**

At a glance, before the detail:

| Hypothesis claim | Verdict | Evidence |
| --- | --- | --- |
| `check_if_db_env_is_set_or_set_from_config` prefers a set `DATABASE_URL` over the INI | **TRUE** | `src/helpers.rs:86-92` |
| That preference decides which URL the postgres connection uses | **FALSE** | `src/config_parse.rs:76-81` reads the INI unconditionally; no `std::env::var` on the postgres arm. `src/main.rs:39` discards the function's result |
| `devenv.nix:25` therefore silently wins in the dev shell | **FALSE for postgres, TRUE for sqlite** | `src/config_parse.rs:83-86` — the *sqlite* arm is env-first, so the exported `postgres://` URL wins there and is fed to `SqliteConnection::establish` (E-1) |
| That is why the INI was never exercised and the copies drifted | **FALSE — right conclusion, wrong mechanism** | The INI *is* the authoritative postgres source. The tracked copies drifted because neither is ever read: `src/cli_structure.rs:29` points at the untracked live INI (Root Cause B) |

**What is confirmed.** The precedence claim is literally true of that function.
`src/helpers.rs:86-92`:

```rust
if let Ok(val) = std::env::var("DATABASE_URL") {
    drop(val);
} else {
    let db_url = ctx.get_user_input_db_url()?;
    std::env::set_var("DATABASE_URL", db_url);
    info!("Fetched the DATABASE_URL env variable");
}
```

Env-set wins; INI is the fallback. Confirmed.

**What is refuted.** That precedence has **no effect on the Postgres connection**, for
two independent reasons:

1. **The function's result is discarded and its only output is a process env var.**
   `src/main.rs:39` — `let _ = check_if_db_env_is_set_or_set_from_config(&ctx);`. It
   returns a Tailscale status string (`src/helpers.rs:94-107`), thrown away. Its sole
   lasting effect is mutating `DATABASE_URL` in the process environment.
2. **Nothing on the Postgres connect path reads `DATABASE_URL`.** The single resolution
   point is `resolve_db_target`, `src/config_parse.rs:72-91`:

```rust
match engine.trim() {
    "postgres" => {
        let url = ctx.get_user_input_db_url()      // INI [db] db_pg_host — unconditional
            .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
        Ok((engine, url))
    }
    "sqlite" => {
        let url = match std::env::var("DATABASE_URL") {   // env FIRST, INI fallback
            Ok(value) => fix_home_directory_path(&value),
            Err(_) => format!("sqlite://{}", get_db_configurations(ctx)?),
        };
        Ok((engine, url))
    }
    _ => Ok((engine, String::new())),
}
```

The `postgres` arm (`src/config_parse.rs:76-81`) reads `ctx.get_user_input_db_url()`
→ `src/global_conf.rs:141-145` → `self.config.get("db", "db_pg_host")` — the INI value,
unconditionally, with **no** `std::env::var` call. All three connect sites route through
it: `src/cli_structure.rs:150-151`, `src/cv_insert.rs:32-33`, `src/user_action.rs:24-25`,
each calling `rusty_cv_creator::database::establish_connection(&engine, &url)`
(`src/database.rs:19-28`).

**Therefore, on the Postgres path the INI always wins and `devenv.nix:25` never reaches
`PgConnection::establish`.** The precedence is the *opposite* of the hypothesis.

**Where the env-wins precedence is real — and harmful.** It is the **sqlite** arm,
`src/config_parse.rs:83-86`, that prefers `DATABASE_URL`. `devenv.nix:25` exports a
`postgres://` URL into every dev shell, so a sqlite-engine run *inside the dev shell*
hands `postgres://rusty_cv:REDACTED-SEE-SOPS@nixos-03…/db_rusty_cv` to
`SqliteConnection::establish` (`src/database.rs:25`), which treats it as a filename.
This is a live latent defect created by the same line, and it is why removing
`devenv.nix:25` is a fix rather than a cosmetic change.

**Corrected causal claim for the drift.** The tracked copies drifted not because the env
shadowed the INI, but because **the only copy the program ever reads at runtime is the
untracked live INI** (`src/cli_structure.rs:29` defaults `--config-ini` to
`~/.config/rusty-cv-creator/rusty-cv-config.ini`). Both tracked copies are unexecuted
documentation: `rusty-cv-config-example.ini` is never loaded by any code path or test,
and `devenv.nix:25` is dead for postgres. Nothing — no test, no CI job, no startup
assertion — ever compares them to the live value, so they were free to rot.

A further finding surfaced during this pass reinforces that conclusion: the postgres arm
may never have been reachable at all, because quoted INI values are not stripped on the
DB accessor path (contributing factor **E-3**, risk **R-10**). If the live INI quotes its
values as the tracked example does, `resolve_db_target` has been falling through to its
`_` arm (`src/config_parse.rs:89`) for the entire life of the postgres feature.

### Alternatives considered and rejected

Competing explanations tested before settling on the four branches below.

| Alternative | Rejected because |
| --- | --- |
| **Merge conflict / bad resolution** left two divergent copies | The divergence was introduced by a single clean commit, not a merge: `fef6db6` (2025-08-19) *added* `env.DATABASE_URL` to `devenv.nix` already pointing at `nixos-03` while the INI said `nixos-01`. `git log -S'REDACTED-SEE-SOPS' --all` returns 3 commits total, none a merge. A `check-merge-conflicts` hook is also enabled (`devenv.nix:95-99`). |
| **Password rotation** desynchronised the copies | Both tracked copies carry the **identical** literal `REDACTED-SEE-SOPS`; only the host and database differ. A rotation would have produced differing passwords, not differing hosts. |
| **A CI job or generator** was meant to reconcile the copies and broke | No such job exists and none ever did: `grep -niE "database\|postgres\|DATABASE_URL" .github/workflows/` matches only GitHub/Codecov token lines across `build.yml`, `rust-tests.yml`, `release.yml`. No generator, template, or import links the three files. |
| **The example INI is a deliberate decoy** with fake values | The credential is real (maintainer-confirmed), and the file is presented as a working configuration (`rusty-cv-config-example.ini:57` sets `engine = "postgres"`, not a placeholder). Genuinely optional fields in the same file are commented out (`:16`, `:22`, `:28`, `:46`, `:53`) — this one is not. |
| **The hooks were disabled or bypassed** (`--no-verify`) for the offending commits | Not needed as an explanation: the hooks are enabled (`devenv.nix:101-111`) and *structurally cannot match* a URL userinfo password (Branch C). Bypass is unnecessary to explain the outcome, so it is not invoked. |

---

## 4. Root cause chain (Toyota 5 Whys, multi-causal)

Four independent branches. Each WHY carries a `file:line` citation.

### Branch A — a plaintext password sits in the tracked tree

```
WHY 1A: A working Postgres password is readable in the tracked tree at two sites.
  [Evidence: git grep -n "REDACTED-SEE-SOPS" -> devenv.nix:25, rusty-cv-config-example.ini:64;
   both are tracked (git ls-files) and both carry the identical literal]

WHY 2A: The config format has one field for the whole connection URL, so the secret
        is structurally inseparable from the non-secret host/database.
  [Evidence: src/global_conf.rs:141-145 - get_user_input_db_url() returns the whole
   `db_pg_host` string; src/config_parse.rs:76-81 passes it verbatim to the caller;
   src/database.rs:24 hands it verbatim to PgConnection::establish. There is no
   second field and no assembly step anywhere in the path]

WHY 3A: The example file is a full working config rather than a template, so the only
        way to make it "correct" was to paste the real, complete URL.
  [Evidence: rusty-cv-config-example.ini:57 `engine = "postgres"` (not sqlite) and
   :64 a complete credentialed URL, versus :16/:22/:28/:46/:53 where genuinely optional
   fields are commented-out placeholders. The file demonstrates the real deployment,
   not a fill-in-the-blanks skeleton]

WHY 4A: No secret-injection seam was ever designed. Every other externally-supplied
        secret in this codebase was given an env-only rule; the DB password was not.
  [Evidence: rusty-cv-config-example.ini:19-21 - "For token auth the token is read
   ONLY from the GITHUB_TOKEN environment variable - never from this INI file, the git
   command line, or the cached clone." The rule exists, is documented, and is enforced
   for GITHUB_TOKEN (ADR-0008 / docs/product/architecture/brief.md:189) - and was simply
   not applied to `db_pg_host`, which predates it by 2 years (53ceda1, 2024-07-28)]

WHY 5A: Postgres support (53ceda1, 2024-07-28) was added as a single-user convenience
        against a Tailscale-only host, so the URL was treated as a connection detail
        rather than a credential, and no threat model made the distinction.
  [Evidence: docs/product/architecture/brief.md:175 - "Security - single-user local
   tool; Postgres reached only over Tailscale". The stated control is network
   reachability; secrecy of the credential is never listed as a control]

-> ROOT CAUSE A: The connection URL was modelled as a single opaque configuration
   string with no secret/non-secret split, because the security model relied entirely
   on network reachability (Tailscale) and never treated the DB password as a secret
   requiring an injection seam - even after the same repo established exactly that
   seam for GITHUB_TOKEN.
```

### Branch B — three copies of `(host, database)` that disagree

```
WHY 1B: devenv.nix says nixos-03/db_rusty_cv, the example INI says nixos-01/
        db_rusty_cv, the live INI says nixos-02/rusty_cv.
  [Evidence: devenv.nix:25; rusty-cv-config-example.ini:64; live value given by
   the maintainer]

WHY 2B: Neither tracked copy is ever read by the running program on the postgres path,
        so neither can be wrong in any way a user would notice.
  [Evidence: src/cli_structure.rs:29 defaults --config-ini to
   "~/.config/rusty-cv-creator/rusty-cv-config.ini" - the LIVE file, not the example;
   src/config_parse.rs:76-81 reads `db_pg_host` from THAT file. devenv.nix:25's
   DATABASE_URL is never consulted on the postgres arm (see section 3).
   git grep "rusty-cv-config-example" over src/ and tests/ returns no hits - the
   example file is loaded by nothing]

WHY 3B: There is no single source of truth and no cross-copy consistency check -
        not in code, not in tests, not in CI.
  [Evidence: the three values live in three unrelated formats (Nix attr, INI key,
   user-local INI key) with no generator, no import, no shared constant.
   .github/workflows/rust-tests.yml, build.yml, release.yml contain no DB config
   step: `grep -niE "database|postgres|DATABASE_URL" .github/workflows/` returns
   only GitHub/Codecov token lines]

WHY 4B: Config was designed as "documentation you copy once", so drift between the
        documented value and the operated value was never modelled as a failure mode.
  [Evidence: README.md:164-171 instructs the user to hand-write a DATABASE_URL into
   a .env - a fourth, parallel, hand-maintained copy. The docs actively multiply
   copies instead of collapsing them]

WHY 5B: The tracked copies are unexecuted artifacts. Nothing in the build, the test
        suite, or the runtime ever forces them to agree with reality, so entropy is
        the default state and 24 months of drift produced zero signal.
  [Evidence: git log -S'REDACTED-SEE-SOPS' --all returns exactly 3 commits (53ceda1
   2024-07-28, 7fd245e 2024-12-20, fef6db6 2025-08-19) - the value was never
   corrected, only copied. The host diverged at fef6db6 (nixos-03 vs the INI's
   nixos-01) and no build, test, or hook noticed for 11 months]

-> ROOT CAUSE B: Configuration exists as three hand-maintained, mutually unaware
   copies, of which only the untracked one is ever executed; there is no source of
   truth, no generator, and no consistency assertion, so the tracked copies decay
   silently and unobservably.
```

### Branch C — the pre-commit hook set could never have caught this

```
WHY 1C: Two enabled secret-detection hooks let a credentialed URL through, twice.
  [Evidence: devenv.nix:101-111 enables detect-aws-credentials and
   detect-private-keys at stage pre-commit; the literal landed anyway at 53ceda1
   and fef6db6]

WHY 2C: Neither hook has any notion of a URL userinfo password. This is structural,
        not a tuning gap.
  [Evidence: pre_commit_hooks/detect_private_key.py - BLACKLIST is a fixed list of
   10 PEM/PuTTY header byte-strings ('BEGIN RSA PRIVATE KEY', 'BEGIN OPENSSH PRIVATE
   KEY', ...) substring-matched against file bytes. "postgres://user:pass@host"
   matches none of them, and no pattern in the list is a regex.
   pre_commit_hooks/detect_aws_credentials.py - main() builds its key set ONLY from
   ~/.aws/config, ~/.aws/credentials, /etc/boto.cfg, ~/.boto and the AWS_SECRET_ACCESS_KEY
   / AWS_SECURITY_TOKEN / AWS_SESSION_TOKEN env vars, then substring-matches those exact
   values (check_file_for_aws_keys). It cannot flag a string that is not already one of
   the developer's own AWS secrets. Source:
   /nix/store/xw778bvafgfszj2vsn0zj2qdyl06pmwg-python3.13-pre-commit-hooks-6.0.0/
   lib/python3.13/site-packages/pre_commit_hooks/]

WHY 3C: The hook set has no generic, pattern- or entropy-based secret scanner at all.
        Its 11 hooks are formatters, whitespace/EOL fixers, a merge-conflict check, a
        shell linter, a markdown linter, and commit-message tooling.
  [Evidence: devenv.nix:78-197 - the complete git-hooks.hooks set:
   rusty-commit-saver, check-merge-conflicts, detect-aws-credentials,
   detect-private-keys, end-of-file-fixer, mixed-line-endings,
   trim-trailing-whitespace, shellcheck, mdsh, treefmt, commitizen, gitlint,
   markdownlint. No gitleaks, ripsecrets, trufflehog, or equivalent]

WHY 4C: The two security-shaped hook names were taken as coverage. Their presence,
        not their matching behaviour, is what got written down as the control.
  [Evidence: docs/product/architecture/brief.md:176 - "no secrets in repo (pre-commit
   hooks for keys/AWS creds in devenv)". The claim cites the hooks by name as the
   evidence for the property. Nobody read what they match]

WHY 5C: The hooks were adopted as a stock bundle rather than derived from this
        repo's own threat surface, and no adversarial test ("commit a fake secret,
        does it block?") was ever run against them.
  [Evidence: the hook block was never revised after the credential landed - the two
   detect-* hooks at devenv.nix:101-111 sit unchanged alongside the very literal at
   devenv.nix:25, in the same file, 76 lines apart. A hook set derived from this
   repo's surface would have been tested against its own content]

-> ROOT CAUSE C: The secret-detection control was selected by name rather than by
   matched pattern, was never adversarially tested against a planted secret, and
   contains no generic credential scanner - so the repo has had zero real protection
   against a credential literal for its entire life.

   Named gap: NO GENERIC SECRET SCANNER. detect-private-keys = fixed PEM header
   allowlist (URL userinfo unmatchable). detect-aws-credentials = matches only the
   committing developer's own pre-existing AWS secret values (URL userinfo
   unmatchable). Neither is a pattern scanner. No CI-side equivalent either.

   Secondary defect in the same hook: detect_aws_credentials.main() returns 2
   ("No AWS keys were found...") when no ~/.aws file and no AWS_* env var exist, and
   devenv.nix:101-105 does not pass --allow-missing-credentials. On any machine or CI
   runner without AWS credentials the hook is a hard failure, not a scan - so it has
   never contributed protection under any configuration.
```

### Branch D — the architecture brief asserts a property that is false

```
WHY 1D: docs/product/architecture/brief.md:176 states "no secrets in repo (pre-commit
        hooks for keys/AWS creds in devenv)". Both halves are wrong: there IS a secret
        in the repo, and the cited hooks cannot detect it.
  [Evidence: brief.md:175-176 verbatim, under "Quality attributes (ISO 25010,
   realized)" - the section header asserts these are REALIZED, not aspirational.
   Contradicted by devenv.nix:25 and rusty-cv-config-example.ini:64, and by Branch C]

WHY 2D: The claim was written by reading the devenv hook names, not by scanning the
        tree or testing the hooks.
  [Evidence: the parenthetical "(pre-commit hooks for keys/AWS creds in devenv)"
   names exactly the two hooks at devenv.nix:101-111 - it is a restatement of the
   config, offered as evidence of the outcome]

WHY 3D: The brief documents intended properties as realized ones, with no
        verification step between claim and publication.
  [Evidence: brief.md:166 heading "Quality attributes (ISO 25010, realized)".
   The four sibling claims at :168-174 (Maintainability, Portability, Reliability,
   Functional suitability) all cite a measurement or an ADR - "line coverage
   54% -> 84% (ADR-0005)", "(ADR-0004)". The Security claim at :175-176 is the only
   one citing neither a measurement nor an ADR]

WHY 4D: There is no ADR and no owner for security, so no artifact was obliged to
        justify the claim.
  [Evidence: brief.md:180-189 Decisions table, D-1..D-8: build, subprocess port,
   persistence, tool checks, coverage, AppContext, CI gates, template sourcing.
   No security ADR. docs/product/architecture/ contains adr-0003..adr-0008 plus
   adr-0006-inject-appcontext - none security-scoped]

WHY 5D: The security posture was assumed from context ("single-user local tool,
        Tailscale-only") rather than analysed, so a documentation claim became the
        only artifact standing in for a control - and once written, it discouraged
        anyone from looking again.
  [Evidence: brief.md:175 - "single-user local tool; Postgres reached only over
   Tailscale" is a context statement used directly as a security conclusion.
   docs/feature/config-injection/feature-delta.md:107 shows the ONE place the
   DATABASE_URL side channel was noticed - flagged as an "Open question" about
   coupling and testability, never as a secrets question]

-> ROOT CAUSE D: An unverified documentation claim was recorded as a realized quality
   attribute, substituting for the control it described - which both hid the exposure
   and removed the incentive to re-examine it.
```

---

## 5. Backwards chain validation

Each root cause traced forward: *if this existed, would it produce the observed symptoms?*

| Root cause | Forward trace | Produces symptom? |
| --- | --- | --- |
| A (no secret/non-secret split in the URL field) | One field must hold everything → an example file that demonstrates a real deployment must contain the password → literal is committed | **Yes** — fault (a), both sites |
| B (three unaware copies, only the untracked one executed) | A tracked copy can hold any value with zero runtime, test, or CI consequence → values diverge whenever any one is edited → three-way disagreement | **Yes** — fault (b); explains 11 months of nixos-01/nixos-03 divergence with no signal |
| C (no generic secret scanner) | A credentialed URL is committed and no hook matches it → commit succeeds → literal persists | **Yes** — explains why (a) landed twice, at `53ceda1` and `fef6db6` |
| D (unverified "no secrets" claim) | Written control substitutes for real control → no audit is triggered → exposure persists across 2 years and multiple architecture reviews | **Yes** — explains duration, not initial occurrence |

**Cross-validation, no contradictions:**

- A + B are orthogonal and compose: A explains *why a secret was in the file*, B explains *why the file's other values were wrong and stayed wrong*. Neither requires the other; both are needed to explain the full defect.
- C is independent of A and B: it explains *why nothing stopped the commit*, and applies to both sites equally.
- D depends on C (the claim cites the hooks) but is not implied by it: the hooks could have been weak *and* the brief honest. D adds the duration factor.
- **Refuted hypothesis is consistent with all four**: the hypothesised env-shadowing mechanism is not needed to explain any symptom. B fully explains the drift by non-execution. The env precedence that *does* exist (sqlite arm, `src/config_parse.rs:83`) explains a *different*, previously unreported latent defect (see contributing factor E-1) — no contradiction, an addition.

**Completeness check** — asked at every level. One additional branch surfaced and is
recorded as contributing factor E-2 (secret leaked into the environment of every
spawned subprocess). It is a consequence of A, not a fifth root cause. No further
branches found.

**All symptoms explained: yes.** Password in tree (A + C), two copies of it (C),
three-way host/db disagreement (B), 24-month persistence (D), plus one latent defect
found during the pass (E-1).

---

## 6. Contributing factors

**E-1 — Precedence is inverted between engines, and the exported URL poisons the
sqlite path.** `resolve_db_target` is INI-only for `postgres`
(`src/config_parse.rs:76-81`) but env-first for `sqlite` (`src/config_parse.rs:83-86`),
while `check_if_db_env_is_set_or_set_from_config` is env-first for *both*
(`src/helpers.rs:86-92`, `:118-124`). Three functions, two rules, one env var. Concrete
consequence: inside the dev shell, `devenv.nix:25` exports a `postgres://` URL that the
sqlite arm prefers, so `SqliteConnection::establish("postgres://rusty_cv:…@nixos-03…")`
(`src/database.rs:25`) is attempted and the URL is interpreted as a filename. This is a
live defect today, independent of the credential issue.

**E-2 — The secret is written into the process environment and inherited by every
subprocess.** `src/helpers.rs:90` — `std::env::set_var("DATABASE_URL", db_url)` — runs
before `SystemRunner` spawns `sudo tailscale` (`src/main.rs:107`), `just`/`tectonic`
(`src/main.rs:96`), and `zathura` (`src/main.rs:58`). `std::process::Command` inherits
the parent environment by default, so the credential is visible in each child's
environment. **The fix must not reuse this mechanism.**

**E-3 — The postgres arm is very likely dead on arrival: quoted INI values are never
stripped on this path.** `configparser` returns values **with** their surrounding
quotes — asserted by the repo's own test, `src/config_parse.rs:116-120`:
`load_config("[section]\nkey = \"value\"")` then
`assert_eq!(ini.get("section","key").unwrap(), "\"value\"")`. Every other config read
compensates by calling `clean_string_from_quotes`: `src/config_parse.rs:50`
(`get_variable_from_config_file`), `:58-59` (`get_db_configurations`),
`src/file_handlers.rs:379`. **The two DB accessors do not**:
`src/global_conf.rs:137-139` (`get_user_input_db_engine`) and `:141-145`
(`get_user_input_db_url`) return the raw value.

Consequences, if the live INI quotes its values the way the tracked example does
(`rusty-cv-config-example.ini:57` `engine = "postgres"`, `:64` `db_pg_host = "postgres://…"`):

- `src/config_parse.rs:75` matches `engine.trim()` against the literal `postgres`, but
  the value is `"postgres"` *including the quote characters* → no arm matches → the
  `_` fallthrough at `:89` returns `(engine, String::new())` → `establish_connection`
  (`src/database.rs:26`) errors with `Unknown DB engine: "postgres"`.
- `src/helpers.rs:82` — `engine.is_ok_and(|e| "postgres" == e)` — fails the same way and
  takes the *else* (sqlite) branch even when the engine is postgres.
- Even if the engine matched, the URL handed to `PgConnection::establish`
  (`src/database.rs:24`) would still carry literal `"` characters.

This is a **conditional finding**: it holds iff the live INI quotes its values. It is
cheap for the maintainer to check and it materially strengthens Root Cause B — if true,
the postgres path has never successfully connected through `resolve_db_target`, which is
the most complete possible explanation for why the tracked copies were free to rot. It
also directly affects the fix: the base URL must be de-quoted **before** the password is
spliced, or the password lands inside a quoted string (§8 R-10).

**F — The hook gap (Branch C), named.** No generic secret scanner in
`devenv.nix:78-197`; the two `detect-*` hooks are structurally incapable of matching URL
userinfo; `detect-aws-credentials` additionally hard-fails (exit 2) wherever no `~/.aws`
exists because `--allow-missing-credentials` is not passed (`devenv.nix:101-105`).

**G — The env-wins precedence (hypothesis, corrected).** Real at `src/helpers.rs:86` and
`src/config_parse.rs:83`; **not** on the postgres connect path. Listed as a contributing
factor to E-1 and to the general confusion about which value is authoritative — not as
the cause of the drift.

**H — Errors are discarded at three of the relevant call sites.**
`src/main.rs:39` (`let _ = check_if_db_env_…`), `src/cli_structure.rs:128`
(`let _ = remove_cv(ctx, &filters)`), and `src/cv_insert.rs:29-42` (DB failure
downgraded to `warn!`). This is a *pre-existing* obstacle to the decided fix's
"fail loudly" requirement — see §7.4 and §8 R-3.

**I — Docs multiply copies.** `README.md:164-171` tells the user to hand-write a
`DATABASE_URL` (including a credentialed `postgresql://user:password@localhost/cv_db`
example at `:171`) into a `.env`, creating a fourth parallel copy — and, for postgres,
one the program never reads.

**J — A credential-shaped literal also sits in a tracked test fixture.**
`tests/integration-tests.rs:44` — `db_pg_host = "postgresql://test:test@localhost/test"`.
Not a real secret, but it is the exact shape any future scanner must flag, so it must be
cleaned or the scanner will need an allowlist on day one.

---

## 7. Precise code changes required by the decided fix shape

Decided shape (not re-litigated here): INI keeps host + database as a **passwordless**
Postgres URL reconciled onto **`nixos-02` / `rusty_cv`**; the password comes from
**`RUSTY_CV_DB_PASSWORD`** (sops via home-manager on the real machine, gitignored `.env`
in dev) and is spliced in **immediately before connecting**; missing/empty
`RUSTY_CV_DB_PASSWORD` on the postgres path **fails loudly** and never falls back to a
passwordless connect; the **sqlite path is unaffected**.

### 7.1 Tracked files still holding a credential literal or a stale host/database

Complete list. `git grep -n "REDACTED-SEE-SOPS"` plus the credential-shaped and
stale-target scan below is exhaustive over tracked files.

| # | File:line | Current content | Required change |
| --- | --- | --- | --- |
| 1 | `devenv.nix:25` | `env.DATABASE_URL = "postgres://rusty_cv:REDACTED-SEE-SOPS@nixos-03…/db_rusty_cv";` | **Delete the line.** It is dead for postgres (§3) and actively poisons the sqlite arm (E-1). See R-4 for the `diesel-cli` consequence. |
| 2 | `rusty-cv-config-example.ini:64` | `db_pg_host = "postgres://rusty_cv:REDACTED-SEE-SOPS@nixos-01…/db_rusty_cv"` | **Replace with the passwordless, reconciled URL**: `postgres://rusty_cv@nixos-02.caracara-palermo.ts.net/rusty_cv`. Add a comment above it stating the password is read **only** from `RUSTY_CV_DB_PASSWORD` and must never be written here — mirroring the existing `GITHUB_TOKEN` rule at `rusty-cv-config-example.ini:19-21`. |
| 3 | `README.md:171` | `echo "DATABASE_URL=postgresql://user:password@localhost/cv_db" > .env` | **Rewrite.** Credential-shaped, and now wrong guidance: `DATABASE_URL` is not read on the postgres path. Replace with: set `[db] db_pg_host` to the passwordless URL, and supply `RUSTY_CV_DB_PASSWORD` via sops/home-manager (real) or a gitignored `.env` (dev). |
| 4 | `README.md:168` | `echo "DATABASE_URL=sqlite://…/applications.db" > .env` | **Keep** (sqlite path is genuinely env-first, `src/config_parse.rs:83`), but move it under an explicit "sqlite only" heading so it is not read as postgres guidance. |
| 5 | `README.md:174-178` | `diesel setup` / `diesel migration run` | **Add a note**: `diesel-cli` reads `DATABASE_URL` from the environment; with `devenv.nix:25` removed it must be supplied inline or from `.env` (see R-4). |
| 6 | `env.sample:1` | `DATABASE_URL=sqlite://$HOME/.config/rusty-cv-creator/applications.db` | **Keep**, and **add a commented line** `# RUSTY_CV_DB_PASSWORD=` with a note that an empty value is rejected (do not ship an uncommented empty assignment — that is exactly the failure case the fix must reject). |
| 7 | `tests/integration-tests.rs:44` | `db_pg_host = "postgresql://test:test@localhost/test"` | **Change to passwordless** `postgresql://test@localhost/test`. Required so the new scanner test (§8.1) needs no allowlist. The fixture drives a sqlite-engine test (`engine = "sqlite"` at `:43`), so `db_pg_host` is never resolved — the change is inert. |
| 8 | `tests/tui_job_applications_scenarios.rs:31` | `db_pg_host = \"postgres://placeholder\"` | **No change** — already passwordless and scanner-clean. Verify it stays that way. |
| 9 | `docs/product/architecture/brief.md:175-176` | "Security — single-user local tool; Postgres reached only over Tailscale; **no secrets in repo (pre-commit hooks for keys/AWS creds in devenv)**" | **False as written — must be rewritten** (Branch D). Replace with the realized state: DB password injected from the environment (`RUSTY_CV_DB_PASSWORD`, sops/home-manager), never in the repo or the INI; enforced by a tracked-tree credential scan in the test suite/CI; historical exposure noted with rotation status. Do not re-cite `detect-aws-credentials`/`detect-private-keys` as the control — they do not provide it. |
| 10 | *(out of tree)* `~/.config/rusty-cv-creator/rusty-cv-config.ini` | live: `nixos-02` / `rusty_cv`, password present | **Maintainer action, manual.** Strip the password from `db_pg_host`, leave host+db. Not reachable from this commit — the single largest breakage risk (§8 R-1). |

### 7.2 Exact call sites that must change

**`src/config_parse.rs:72-91` — `resolve_db_target`, the single choke point.**
The `postgres` arm at `:76-81` is the only place the postgres URL is produced, and all
three connects consume it. Splice here and every call site inherits the behaviour with
no signature change:

- `src/cli_structure.rs:150-151` — `let (engine, url) = resolve_db_target(ctx)?;` → `establish_connection(&engine, &url)?`
- `src/cv_insert.rs:32-33` — same pair, inside the lazy `open_conn` closure
- `src/user_action.rs:24-25` — same pair

Add a private helper in `src/config_parse.rs`, e.g.
`fn splice_db_password(base_url: &str) -> Result<String, Box<dyn std::error::Error>>`,
and call it in the `postgres` arm only. It must:

0. **de-quote the base URL first** — `clean_string_from_quotes` (`src/helpers.rs:46`),
   as every other config reader already does (E-3). Best applied inside
   `get_user_input_db_url` (`src/global_conf.rs:141-145`) together with the same
   treatment for `get_user_input_db_engine` (`:137-139`), so the engine `match` at
   `src/config_parse.rs:75` and the comparison at `src/helpers.rs:82` can succeed at all;
1. read `RUSTY_CV_DB_PASSWORD`; on `Err`, on empty, or on whitespace-only → return `Err`
   with a message naming the variable, both supply routes (sops/home-manager, gitignored
   `.env`), and the config file it applies to;
2. **percent-encode** the password before insertion (see §8 R-6);
3. insert it into the userinfo of `base_url` after the username;
4. **reject a `base_url` that already carries a password** (`user:pass@`) with a distinct
   error — otherwise a stale live INI yields `postgres://rusty_cv:OLD:NEW@host`
   (§8 R-1);
5. **never** return `base_url` unchanged on any error path — no passwordless fallback;
6. **never** call `std::env::set_var` (E-2).

**`src/config_parse.rs:83-86` — the `sqlite` arm: do not touch.** Its `DATABASE_URL`-first
behaviour is depended on by `tests/integration-tests.rs:78`,
`tests/tui_job_applications_scenarios.rs:62`, `:86`, `:111`, and the feature specs at
`tests/tui_job_applications/walking_skeleton.feature:18,24-26` and
`milestone_1_table_display.feature:23,58-59`.

**`src/helpers.rs:77-126` — `check_if_db_env_is_set_or_set_from_config`.**
The postgres arm's env block at `:86-92` must be **removed**: it has no consumer (§3)
and, once a password exists, would export it to every subprocess (E-2). Keep
`ensure_tools_available(&["sudo","tailscale"])` at `:84` and the Tailscale probe at
`:94-107`. The sqlite arm at `:118-124` is unchanged. Update the doc comment at
`src/config_parse.rs:66-71`, which currently documents the old mirror relationship.

**`src/global_conf.rs:141-145` — `get_user_input_db_url`.** Now returns a *passwordless
base* URL. Two required edits:
- the error string is wrong today — `"Could not get the database engine"` for a missing
  `db_pg_host` (`:144`). Make it name `[db] db_pg_host` and the config file path.
- rename to something like `get_db_base_url` so no caller mistakes it for a
  ready-to-connect URL. Callers: `src/config_parse.rs:77`, `src/helpers.rs:89` (the
  latter disappears with the change above).

**`src/database.rs:19-28` — `establish_connection`: unchanged.** It receives a resolved
`(engine, url)` and stays config-free, per ADR-0006 (`docs/product/architecture/adr-0006-inject-appcontext.md:60`)
and D-5 (`docs/feature/config-injection/feature-delta.md:24`). Do not move splicing here.

**Loudness plumbing — three sites that currently swallow the error** (contributing
factor H). `resolve_db_target` returning `Err` is necessary but not sufficient:

| Site | Today | Effect on a missing-password error |
| --- | --- | --- |
| `src/cli_structure.rs:150` (`run_list_tui`) | `?` → `match_user_action` → `src/main.rs:43-49` `eprintln!` + `exit(1)` | **Already loud.** No change needed. |
| `src/cli_structure.rs:128` (`Remove` arm) | `let _ = remove_cv(ctx, &filters);` | **Silently swallowed.** Must propagate. |
| `src/cv_insert.rs:29-42` | `warn!("Could not save CV to database: {e:}")` | **Effectively silent** — `env_logger` (`src/main.rs:33`) defaults to `Error`, so `warn!` prints nothing without `RUST_LOG`. |

`src/cv_insert.rs` is a deliberate design ("A failed DB save must not discard a
successfully generated CV", `src/cv_insert.rs:28`). A missing password is a
*configuration* error, not a transient DB error, and the decided shape requires it to be
loud. Minimum change consistent with both: give the missing-password error a
distinguishable type (or check `RUSTY_CV_DB_PASSWORD` in a pre-flight in `main` when the
resolved engine is postgres and `--save-to-database` is set) so it reaches stderr while
genuine connection failures keep the warn-and-continue behaviour. **Decision point for
the maintainer** — flagged, not chosen here.

### 7.3 Files explicitly NOT to change

`src/database.rs` (config-free by ADR-0003/0006) · `src/config_parse.rs:83-86` (sqlite
arm) · `src/helpers.rs:118-124` (sqlite arm) · `env.sample:1` value ·
`tests/tui_job_applications_scenarios.rs:31`.

---

## 8. Regression-test plan

### 8.1 The test that would have caught a credential literal landing in a tracked config file

**`tests/no_tracked_credentials.rs`** — a normal (non-`#[ignore]`) integration test so
`.github/workflows/rust-tests.yml` runs it on every push.

- **Source of truth**: `git ls-files -z` from `env!("CARGO_MANIFEST_DIR")` — scans the
  *tracked* set, so untracked local configs and `.env` are correctly out of scope.
- **Detection**: regex over each file's text —
  `(?i)\b(postgres|postgresql|mysql|mongodb|redis|amqp|https?)://[^\s"'/]*:[^\s"'@/]+@`
  (scheme, then userinfo containing a `:`-separated non-empty password, then `@`).
- **Assertion**: zero matches. Failure message prints `file:line` and the matched
  scheme+user with the password redacted.
- **Allowlist**: empty by construction — that is why `tests/integration-tests.rs:44`
  must be cleaned first (§7.1 #7). An allowlist is where this class of test goes to die.
- **Self-test (guards against a silently-broken regex)**: assert the matcher fires on a
  known-bad string assembled at runtime from fragments, so the test file itself does not
  trip the scan.
- **Historical proof**: the same matcher run against `git show 53ceda1:rusty-cv-config-example.ini`
  and `git show fef6db6:devenv.nix` must match — i.e. it *would* have blocked both
  original commits. Verify manually once; do not commit as a test (it would pin history).
- **Fast local feedback**: add a real scanner to `devenv.nix` `git-hooks.hooks` (gitleaks
  or ripsecrets) at stage `pre-commit`. Pre-commit sees only staged files; the cargo test
  sees the whole tracked tree. Both are needed. Also add `--allow-missing-credentials` to
  `detect-aws-credentials` (`devenv.nix:101-105`) or drop the hook — today it exits 2
  wherever no `~/.aws` exists (Branch C, secondary).

### 8.2 The test that proves the loud failure on a missing password variable

Three layers. Env-mutating tests use `#[serial_test::serial]` with save/restore — the
pattern already in the repo at `src/helpers.rs:274-289` and
`tests/integration-tests.rs:236-245`. (`cargo-nextest` isolates per process, but
`cargo test` uses threads and `tests/integration-tests.rs:78` sets `DATABASE_URL`
without restoring it — serial is mandatory.)

**Layer 1 — unit, on the splice helper (`src/config_parse.rs`):**

| Case | `RUSTY_CV_DB_PASSWORD` | Expected |
| --- | --- | --- |
| unset | `env_remove` | `Err`; message contains `RUSTY_CV_DB_PASSWORD` |
| empty | `""` | `Err`; same message |
| whitespace | `"   "` | `Err`; same message |
| valid | `"s3cret"` | `Ok`; URL contains `rusty_cv:s3cret@`, and host `nixos-02…` + db `/rusty_cv` preserved byte-for-byte |
| **no-fallback guard** | unset | returned value is `Err`, and no code path returns the base URL — assert the `Err` branch explicitly rather than `is_err()` alone |
| base already has a password | any | `Err`, distinct message (guards the stale live INI, R-1) |

**Layer 2 — `resolve_db_target` integration (`src/config_parse.rs:72`):** build an `Ini`
with `[db] engine = "postgres"` and a passwordless `db_pg_host`; with the var unset,
assert `Err`. With the var set, assert the returned tuple is
`("postgres", "<url with password>")`. Mirror with `engine = "sqlite"` and the var unset
→ `Ok`, proving **the sqlite path is unaffected**.

**Layer 3 — subprocess, proving end-to-end loudness** (pattern:
`tests/tui_job_applications_scenarios.rs:79-118`): run `CARGO_BIN_EXE_rusty_cv_creator`
with a temp INI (`engine = "postgres"`, passwordless `db_pg_host`) and
`.env_remove("RUSTY_CV_DB_PASSWORD")`. Assert **non-zero exit** and that combined
stdout+stderr contains `RUSTY_CV_DB_PASSWORD`. This is the only layer that catches the
error being swallowed at `src/cli_structure.rs:128` or downgraded to `warn!` at
`src/cv_insert.rs:41` (contributing factor H) — a unit test alone will pass while the
binary stays silent.

**Layer 4 — sqlite non-regression (must stay green with `RUSTY_CV_DB_PASSWORD` unset):**
`tests/integration-tests.rs:78`, `tests/tui_job_applications_scenarios.rs:62`, `:86`,
`:111`, `tests/tui_job_applications_specifications.rs:140-146`,
`src/database.rs:152-158`.

**Layer 5 — password-encoding property test:** `proptest` over passwords drawn from an
alphabet including `@ : / ? # % [ ] space`; assert the spliced URL parses back to the
same host, database, user, and password. Sops-generated passwords routinely contain these
(§8 R-6). The repo already uses proptest (`tests/integration-tests.proptest-regressions`).

---

## 9. Risk assessment for the decided fix

Ordered by expected damage.

**R-1 — The live INI still contains the password, and it is outside this commit's
reach. HIGH.**
`~/.config/rusty-cv-creator/rusty-cv-config.ini` currently holds
`postgres://rusty_cv:<pw>@nixos-02…/rusty_cv`. After the fix, `splice_db_password`
receives that string as its "base". Naive insertion yields
`postgres://rusty_cv:<old>:<new>@nixos-02…` — malformed, and the failure mode is a
confusing parse/auth error rather than the intended actionable message.
*Mitigation (mandatory)*: the `base_url` already-has-a-password check in §7.2 item 4,
with an error that says exactly which file to edit and what to delete. Plus a
release-note line: strip the password from the live INI before upgrading. This is the
one breakage the user will actually hit.

**R-2 — Cannot prove a real connect: the database exists on no host. MEDIUM (schedule,
not correctness).**
`nixos-02`/`rusty_cv` is being recreated by a parallel lane. Any end-to-end
`PgConnection::establish` will fail regardless of this fix, so a red test is not evidence
of a defect here.
*Mitigation*: define done as **(i)** the assembled URL asserted string-wise against an
expected literal, and **(ii)** the loud-failure behaviour proven by §8.2 Layers 1-3 —
neither needs a server. When `nixos-02` comes up, the one remaining check is that a
successful connect distinguishes *auth failure* (wrong password → the splice works, the
secret is stale) from *config error* (missing variable). Record as an open verification
item; do not let it block the merge.

**R-3 — "Fail loudly" is not achieved by returning `Err` alone. MEDIUM.**
Two of the three call chains discard it: `src/cli_structure.rs:128` (`let _ =`) and
`src/cv_insert.rs:41` (`warn!`, invisible at `env_logger`'s default `Error` level, set at
`src/main.rs:33`). A fix that only touches `resolve_db_target` will pass unit tests and
still be silent for a user doing `insert --save-to-database` — the single most common
postgres operation.
*Mitigation*: §8.2 Layer 3 subprocess test is the gate. Ship the loudness plumbing in the
same commit as the splice, never after.

**R-4 — Removing `devenv.nix:25` breaks `diesel-cli` in the dev shell. LOW-MEDIUM,
certain to be noticed.**
`diesel setup` / `diesel migration run` (`README.md:174-178`, `diesel-cli` at
`devenv.nix:31`) read `DATABASE_URL` from the environment. Today the dev shell supplies
it; after removal it is unset.
*Mitigation*: `env.sample:1` already carries a correct sqlite `DATABASE_URL` — document
copying it to the gitignored `.env` (`.gitignore` line `.env`; `dotenv.enable = true` at
`devenv.nix:22` loads it at shell entry). For postgres migrations, pass it inline for
that command only. Note the upside: removal also fixes E-1, where the exported
`postgres://` URL was being fed to `SqliteConnection::establish`.

**R-5 — `dotenvy` does not find `.env` outside the repo. MEDIUM on the real machine.**
`src/main.rs:34` calls `dotenv().ok()`, which searches from the **current working
directory** upward. The real user runs the binary from `~/Documents/CV_Applications`
and similar, so the `.env` route is dev-only in practice.
*Mitigation*: sops/home-manager must export `RUSTY_CV_DB_PASSWORD` into the **user
session** environment (systemd user environment / `home.sessionVariables`), not only into
an interactive shell rc. If it lands only in `.zshrc`, non-interactive invocations
(cron, desktop launchers, `systemd --user` units) will fail loudly on every postgres run
— correct behaviour, maximally annoying. Hand this constraint to the home-manager lane
explicitly.

**R-6 — Special characters in the sops-generated password corrupt the URL. MEDIUM.**
`@ : / ? # %` and space are all legal in a generated password and all significant in a
URL. Unencoded splicing silently produces a wrong host or a truncated password, and the
error surfaces as an auth failure — the hardest kind to debug against a DB that does not
exist yet (R-2).
*Mitigation*: percent-encode (§7.2 item 2) plus the property test at §8.2 Layer 5.

**R-7 — Test-suite blast radius. LOW.**
`tests/integration-tests.rs:44` fixture edit is inert (`engine = "sqlite"` at `:43`). New
env-var tests must be `#[serial_test::serial]` — `tests/integration-tests.rs:78` sets
`DATABASE_URL` process-wide and never restores it, so an unserialized test will flake
under `cargo test`'s threading (nextest's process-per-test would hide it locally and CI
would differ).

**R-8 — The secret is already burned. Not fixed by this change. HIGH, out of scope but
must be recorded.**
`REDACTED-SEE-SOPS` has been in git history since `53ceda1` (2024-07-28) and remains in every
clone and every CI checkout after this fix. Removing it from the working tree changes
nothing about history.
*Required follow-up*: rotate the `rusty_cv` role password on `nixos-02` as part of the
database recreation, so the new sops secret is never the burned one. Decide separately
whether to rewrite history (`git filter-repo`) or accept it — rotation makes the
exposure inert either way, and is the cheaper of the two.

**R-9 — Rollback path. LOW (this is the good news).**
The sqlite arm is untouched, so `engine = "sqlite"` in the INI remains a fully working
escape hatch if the postgres path misbehaves. No migration, no data movement, no schema
change is involved in this fix. **Caveat**: the rollback is INI-only — the `--engine`
flag (`src/cli_structure.rs:32-34`) is dead code and cannot be used for it (see §11.4).

**R-10 — Quoted INI values may defeat the fix entirely. MEDIUM-HIGH, cheap to check
first.**
Per E-3, `get_user_input_db_engine` and `get_user_input_db_url`
(`src/global_conf.rs:137-145`) do not strip quotes, unlike every other config reader
(`src/config_parse.rs:50`, `:58-59`). If the live INI quotes its values, then **today**
the engine `match` at `src/config_parse.rs:75` never selects the `postgres` arm, and
**after the fix** the password would be spliced into a URL that still carries literal
`"` characters. The visible symptom in both cases is a connection error that looks like
a credential problem — maximally misleading against a database that does not exist yet
(R-2).
*Mitigation*: apply `clean_string_from_quotes` in both accessors (§7.2 item 0) and add a
test with a quoted-value INI (`engine = "postgres"`, quoted `db_pg_host`) asserting that
`resolve_db_target` selects the postgres arm and returns an unquoted URL. **Check the
live INI's quoting before writing any code** — it determines whether this is a
pre-existing outage being uncovered or a non-issue.

---

## 10. Solution map — every root cause has a mapped solution

| Root cause | Solution | Type | Priority |
| --- | --- | --- | --- |
| **A** — URL modelled as one opaque string, no secret seam | Passwordless base URL in the INI + `RUSTY_CV_DB_PASSWORD` spliced at connect time (§7.2); the `GITHUB_TOKEN` env-only rule extended to the DB password | Permanent fix | P1 |
| **B** — three unaware copies, only the untracked one executed | Delete `devenv.nix:25` (one copy gone); reconcile the example INI onto `nixos-02`/`rusty_cv` (§7.1 #2); README stops instructing a fourth copy (§7.1 #3) | Permanent fix | P1 |
| **C** — no generic secret scanner, never adversarially tested | Tracked-tree credential scan as a CI-run cargo test (§8.1) + a real scanner hook in `devenv.nix`; fix or drop `detect-aws-credentials` | Early detection | P1 |
| **D** — unverified "no secrets" claim standing in for a control | Rewrite `brief.md:175-176` to the realized state, citing the scan test as the evidence (§7.1 #9); record the exposure and its rotation status | Prevention | P2 |
| **E-1** — inverted precedence, postgres URL poisoning the sqlite arm | Removing `devenv.nix:25` resolves it in practice; document the one remaining rule (postgres = INI + env password; sqlite = env-first) in the `resolve_db_target` doc comment | Permanent fix | P1 (free with B) |
| **E-2** — secret exported to every subprocess | Delete the env-set at `src/helpers.rs:86-92`; splice into a local `String` only, never `set_var` (§7.2) | Permanent fix | P1 |
| **H** — errors swallowed at two of three call chains | Propagate at `src/cli_structure.rs:128`; distinguish config-error from DB-error at `src/cv_insert.rs:29-42`; gated by the §8.2 Layer 3 subprocess test | Permanent fix | P1 |
| **E-3** — quotes never stripped on the DB accessors | Apply `clean_string_from_quotes` in `get_user_input_db_engine` and `get_user_input_db_url` (`src/global_conf.rs:137-145`), plus a quoted-INI test (§8.2 / R-10) | Permanent fix | P1 (prerequisite — the splice is worthless if the postgres arm is never selected) |
| **J** — credential-shaped test fixture | Clean `tests/integration-tests.rs:44` to passwordless so the scanner needs no allowlist | Prevention | P1 (blocks §8.1) |
| **R-8** — burned secret in git history | Rotate the `rusty_cv` password during the nixos-02 recreation | Mitigation | P0 (schedule with the parallel lane) |

**Immediate mitigation vs permanent fix.** There is no active incident, so no mitigation
is required to restore service. The one time-sensitive item is **R-8 (rotation)**, which
should ride the nixos-02 recreation lane already in flight — after that, everything above
is a permanent fix or a prevention measure.

---

## 11. Open verification items

1. **Live connect unproven** (R-2) — re-verify once `nixos-02`/`rusty_cv` exists.
   *Hypothesis until then: the spliced URL is correct; only string-level assertions back it.*
2. **Rotation status of `REDACTED-SEE-SOPS`** (R-8) — unknown at the time of this analysis.
3. **sops/home-manager delivery scope** (R-5) — whether `RUSTY_CV_DB_PASSWORD` reaches
   non-interactive user-session processes is a property of the home-manager lane, not
   verifiable from this repo.
4. **`--engine` is dead code — CONFIRMED, not just suspected.** `src/cli_structure.rs:32-34`
   defines `--engine` (default `sqlite`), but `UserInput.engine` is never read anywhere:
   a grep for `engine` across `src/` and `tests/` yields only struct-literal
   initialisations in test fixtures, plus the three `resolve_db_target`/`establish_connection`
   destructurings, plus `get_user_input_db_engine` — which reads the **INI**
   (`src/global_conf.rs:137-139` → `[db] engine`), not the flag. The CLI flag therefore
   has no effect on any code path. Consequence for this work: engine selection and the
   R-9 rollback are INI-only. Out of this defect's scope to fix, but it must not be
   relied on.

5. **Does the live INI quote its values?** (E-3 / R-10) — determines whether the postgres
   path is currently broken independently of the credential issue. One `grep` by the
   maintainer settles it; everything in E-3 is conditional on the answer.
