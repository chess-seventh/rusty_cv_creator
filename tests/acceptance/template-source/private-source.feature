# Feature: Sourcing a private CV template with existing credentials (TS-02)
#
# Documentation SSOT mapped to concrete Rust tests (no cucumber-rust harness).
# See the traceability table in docs/feature/template-source/feature-delta.md
# (Wave: DISTILL). Maps to in-crate `distill_specs` (FakeRunner asserts the exact
# git command strings) and the UC-1 `uc1_specs`. Pending DELIVER (RED scaffolds).
#
# @in-memory: the git invocation is asserted through the FakeRunner double; the
#   secret-handling guarantees are asserted on the constructed command, not by
#   contacting a real remote.

Feature: Sourcing a private CV template with existing credentials
  As Francesco, whose canonical template repo is private
  I want it pulled with my machine's existing auth
  So that I need no bespoke token plumbing for my real workflow

  @US-02 @in-memory @contract-shape:bounded-change
  Scenario: A private SSH source clones over its git@ URL
    Given "cv_template_path" is a "git@" URL with the SSH transport selected
    When the template source is resolved
    Then the repository is cloned over its git@ URL using the agent
    And no askpass helper is used on the SSH path

  @US-02 @error @in-memory @contract-shape:unbounded-preservation
  Scenario: A token is taken from the environment and never placed on the command line
    Given the token transport is selected for an "https" private URL
    When the git invocation is prepared
    Then the credential is supplied through an askpass helper
    And the token value never appears in the git command line

  @US-02 @error @in-memory @contract-shape:unbounded-preservation
  Scenario: An authentication failure is reported with an auth-specific hint
    Given a private source whose credentials are rejected
    When the failure is classified
    Then it is reported as an authentication failure, distinct from a network error
