# Feature: Pinning the CV template to an exact version (TS-03)
#
# Documentation SSOT mapped to concrete Rust tests (no cucumber-rust harness).
# See the traceability table in docs/feature/template-source/feature-delta.md
# (Wave: DISTILL). Maps to in-crate `distill_specs`. The unset-ref default-branch
# behaviour is the already-GREEN skeleton path. Pinned/bad-ref are pending DELIVER.
#
# @in-memory: the checkout command is asserted through the FakeRunner double.

Feature: Pinning the CV template to an exact version
  As Francesco
  I want to pin the template to a branch, tag, or commit
  So that a given application reproducibly uses a known-good template

  @US-03 @in-memory @contract-shape:bounded-change
  Scenario: A pinned version is checked out and its resolved revision is logged
    Given a template version is pinned to "v2.1"
    When the template source is resolved
    Then that exact version is checked out
    And its resolved revision is logged

  @US-03 @contract-shape:bounded-change
  Scenario: With no version pinned the default branch is used
    Given no template version is pinned
    When the template source is resolved
    Then the repository default branch is used

  @US-03 @error @in-memory @contract-shape:unbounded-preservation
  Scenario: An unknown version is refused without falling back
    Given a template version that does not resolve in the repository
    When the failure is classified
    Then it is reported as an unknown-version error
    And the default branch is not silently used instead
