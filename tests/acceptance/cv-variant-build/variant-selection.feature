# Feature: Choosing which CV variant to build
#
# RETROACTIVE backfill (LEAN). The code is already shipped and GREEN (v4.0.2).
# This .feature is a DOCUMENTATION SSOT for behaviour, not an executable harness:
# the project has NO cucumber-rust runner. Each scenario is mapped to the concrete
# existing Rust test(s) in the traceability table in
# docs/feature/cv-variant-build/feature-delta.md (Wave: DISTILL).
# Business language only; ubiquitous terms: variant, job title, default.

Feature: Choosing which CV variant to build
  As Francesco, applying to a specific role
  I want the right CV variant chosen for me
  So that I attach a correctly framed CV without hand-editing drivers

  @US-01 @contract-shape:pure-function
  Scenario: An explicit variant choice is honoured
    Given Francesco chooses the "senior-sre" variant explicitly
    And the job title is "Platform Engineer"
    When the variant to build is decided
    Then the "senior-sre" variant is selected

  @US-01 @error @contract-shape:pure-function
  Scenario: An unrecognised variant choice falls back to inference from the title
    Given Francesco chooses an unrecognised variant "bogus"
    And the job title is "Platform Engineer"
    When the variant to build is decided
    Then the tool warns about the unknown choice and does not abort
    And the "senior-platform-engineer" variant is selected from the title

  @US-02 @contract-shape:pure-function
  Scenario: A platform-engineering title selects the platform variant
    Given no explicit variant is chosen
    And the job title is "Senior Platform Engineer"
    When the variant to build is decided
    Then the "senior-platform-engineer" variant is selected

  @US-02 @contract-shape:pure-function
  Scenario: A site-reliability title selects the reliability variant
    Given no explicit variant is chosen
    And the job title is "Site Reliability Engineer"
    When the variant to build is decided
    Then the "senior-sre" variant is selected

  @US-02 @contract-shape:pure-function
  Scenario: A devops title selects the devops variant
    Given no explicit variant is chosen
    And the job title is "DevOps Specialist"
    When the variant to build is decided
    Then the "senior-devops" variant is selected

  @US-02 @edge @contract-shape:pure-function
  Scenario: A manager title wins over other role keywords
    Given no explicit variant is chosen
    And the job title is "Engineering Manager - DevOps"
    When the variant to build is decided
    Then the "engineering-manager" variant is selected

  @US-02 @error @contract-shape:pure-function
  Scenario: An unrecognised title falls back to the configured default
    Given no explicit variant is chosen
    And the job title is "Accountant"
    When the variant to build is decided
    Then the configured default variant "senior-devops" is selected
