# Pre-DELIVER RED Classification — template-source

> Per `nw-distill` § Pre-DELIVER fail-for-the-right-reason gate. One line per
> scenario test. Command: `devenv shell -- cargo test` (compile + existing-green)
> then `devenv shell -- cargo test --bin rusty_cv_creator -- --ignored` (confirm
> the pending specs panic). Bare `cargo` cannot link libpq — devenv is required.

## Result line

- `cargo test` (whole workspace): **133 passed; 0 failed; 8 ignored** — crate
  **COMPILES** with the new scaffolds; the prior suite stayed green (131 → +2 new
  green: `ts01_is_git_url_classifies_known_forms`, `ts01_ac3_…` subprocess).

## Classification (each pending spec)

| Test | Reason | Verdict |
| --- | --- | --- |
| `command_runner::uc1_specs::uc1_run_capturing_exposes_stderr` | `panic!` at `run_capturing` scaffold | RED (MISSING_FUNCTIONALITY) |
| `template_source::distill_specs::ts02_ac1_ssh_source_clones_via_git_at_url` | setup (TempDir + FakeRunner) OK, then `panic!` at `resolve_classified` | RED (MISSING_FUNCTIONALITY) |
| `template_source::distill_specs::ts02_ac2_token_uses_askpass_and_never_on_argv` | `panic!` at `auth_invocation_flags` | RED (MISSING_FUNCTIONALITY) |
| `template_source::distill_specs::ts02_ac3_auth_failure_stderr_classified_as_auth` | `panic!` at `classify_git_stderr` | RED (MISSING_FUNCTIONALITY) |
| `template_source::distill_specs::ts03_ac1_pinned_ref_is_checked_out` | setup OK, then `panic!` at `with_ref` | RED (MISSING_FUNCTIONALITY) |
| `template_source::distill_specs::ts03_ac3_bad_ref_classified_no_silent_fallback` | `panic!` at `classify_git_stderr` | RED (MISSING_FUNCTIONALITY) |
| `template_source::distill_specs::ts04_cache_action_matrix` | `panic!` at `TemplateCache::decide` | RED (MISSING_FUNCTIONALITY) |
| `template_source::distill_specs::ts04_cache_key_is_deterministic` | `panic!` at `TemplateCache::cache_key` | RED (MISSING_FUNCTIONALITY) |

All eight fail by `panic!("not yet implemented — RED scaffold")` **after** any
fixture setup succeeds — genuine RED, none BROKEN (no import/compile/setup error).
The gate **passes**: handoff to DELIVER is unblocked.

## Already-green scenarios (no RED expected)

- `walking_skeleton_github_source_resolves_template_dir` (skeleton, TS-01/AC1, TS-03/AC2 default branch)
- `ts01_is_git_url_classifies_known_forms` (TS-01/D1, proptest)
- `ts01_ac3_bad_template_value_fails_fast_naming_the_value` (TS-01/AC3, subprocess driving adapter)
- `test_detect_existing_dir_is_local` / `test_local_directory_resolves_to_passthrough_path` (TS-01/AC2)
