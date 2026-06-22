# Feature: Sourcing the CV template from a public location (TS-01)
#
# Documentation SSOT mapped to concrete Rust tests (no cucumber-rust harness).
# See the traceability table in docs/feature/template-source/feature-delta.md
# (Wave: DISTILL). The walking skeleton is already GREEN; the other scenarios map
# to in-crate `distill_specs` / the subprocess driving-adapter test.
#
# @real-io git uses a real `git clone` from a local `file://` bare-repo fixture
#   (deterministic, no network) — the genuine git shell-out, not a mock.

Feature: Sourcing the CV template from a public location
  As Francesco, setting up on a new machine
  I want the tool to pull the template itself from a public git URL
  So that I never have to clone and babysit a local copy by hand

  @walking_skeleton @driving_port @US-01 @real-io @contract-shape:bounded-change
  Scenario: Francesco sources a public template by URL end to end
    Given "cv_template_path" is a reachable public git URL
    When Francesco generates a CV
    Then the repository is cloned into the cache
    And the cloned template is staged for the build, ready under the working copy

  @US-01 @edge @real-io @contract-shape:unbounded-preservation
  Scenario: An existing local directory is used exactly as before
    Given "cv_template_path" is an existing local template directory
    When the template source is resolved
    Then the directory is used unchanged and nothing is cloned

  @US-01 @error @driving_adapter @real-io @contract-shape:pure-function
  Scenario: A value that is neither a directory nor a git URL is refused
    Given "cv_template_path" is "definitely-not-a-dir-nor-a-git-url"
    When Francesco generates a CV
    Then the run fails fast naming the offending value and the accepted forms

  @US-01 @property @contract-shape:pure-function
  Scenario: Recognised URL forms are detected as git sources
    Given a value in a recognised git-URL form
    When the source type is auto-detected
    Then it is classified as a git source

  @US-01 @property @edge @contract-shape:pure-function
  Scenario: A bare token or local path is not mistaken for a git URL
    Given a bare word or an ordinary local path
    When the source type is auto-detected
    Then it is not classified as a git source
