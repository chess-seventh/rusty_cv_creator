# Feature: Overriding the CV template source/ref from the CLI (TS-05, DISCUSS delta L54)
#
# Documentation SSOT mapped to concrete Rust tests (no cucumber-rust harness).
# See the traceability table in docs/feature/template-source/feature-delta.md
# (Wave: DISTILL (delta L54)). Maps to the in-crate `distill_specs_l54` /
# `distill_specs_l54_integration` modules in src/file_handlers.rs and the
# subprocess smoke test in tests/cli_smoke.rs.
#
# D8 precedence: flag > INI > built-in default. `--repo` overrides
# `[cv] cv_template_path`; `--branch` overrides `[cv] cv_template_ref`. Both are
# global flags on the `insert` entry point; the override is a pre-resolve value
# swap in prepare_path_for_new_cv — the resolver, auth, cache and downstream
# build flow (D5) are unchanged.
#
# @in-memory: the override selection is asserted through pure functions; the
# local-dir passthrough is driven at the create_directory seam with a FakeRunner.

Feature: Overriding the CV template source and ref from the command line
  As Francesco
  I want to pass --repo and/or --branch on the insert command
  So that I can build from a different template repo or ref for a one-off run
  without editing my INI file

  @US-05 @contract-shape:pure-function
  Scenario: The --repo flag overrides the INI cv_template_path
    Given a --repo git URL is passed
    And the INI configures a different cv_template_path
    When the effective template path is resolved
    Then the --repo value is used (flag > INI)

  @US-05 @contract-shape:pure-function
  Scenario: The --branch flag overrides the INI cv_template_ref
    Given a --branch ref is passed
    And the INI configures a different cv_template_ref
    When the effective template ref is resolved
    Then the --branch value is checked out (flag > INI)

  @US-05 @edge @contract-shape:unbounded-preservation
  Scenario: With neither flag the INI values are used byte-for-byte
    Given neither --repo nor --branch is passed
    When the effective template path and ref are resolved
    Then the INI cv_template_path and cv_template_ref are used unchanged

  @US-05 @contract-shape:bounded-change
  Scenario: A --repo local directory is honoured as a local passthrough
    Given a --repo pointing at an existing local directory
    And the INI cv_template_path points elsewhere
    When the template source is resolved
    Then the template is sourced from the --repo directory (auto-detected LOCAL)

  @US-05 @contract-shape:pure-function
  Scenario: A --branch without --repo overrides only the ref
    Given a --branch ref is passed but no --repo
    When the effective template path and ref are resolved
    Then the ref override applies against the INI-configured repo
