# Feature: Building and filing the tailored CV
#
# RETROACTIVE backfill (LEAN). Documentation SSOT mapped to existing GREEN Rust
# tests (no cucumber-rust harness). See the traceability table in
# docs/feature/cv-variant-build/feature-delta.md (Wave: DISTILL).
#
# @in-memory: the build runner is exercised through the FakeRunner double
#   (no real `just`/`tectonic` subprocess); the filesystem is real (tmp dir).
#   Real-IO boundary: in production the SystemRunner shells out to `just build
#   <variant>`; that real boundary is covered by the template-contract smoke test
#   recommended in the architecture brief, NOT by these acceptance specs.

Feature: Building and filing the tailored CV
  As Francesco, after choosing a variant
  I want the CV built and filed in a predictable place
  So that I know exactly which file to attach to my application

  @walking_skeleton @driving_port @US-03 @in-memory @contract-shape:bounded-change
  Scenario: Francesco generates a tailored CV end to end
    Given a configured template for the "senior-devops" variant
    And Francesco applies to "ACME" for a "Senior DevOps" role
    When Francesco generates the CV
    Then a CV PDF is produced and filed under the per-year output location

  @US-03 @in-memory @contract-shape:bounded-change
  Scenario: The built CV is filed under the per-year output directory and the working copy is cleaned up
    Given a configured template for the "senior-devops" variant
    And a successful build for "ACME" and a "Senior DevOps" role
    When the CV is filed
    Then a PDF is placed under the configured per-year output directory
    And the dated working directory is removed, leaving only the PDF copies

  @US-03 @edge @contract-shape:pure-function
  Scenario: Spaces in the job and company become dashes in the filed name
    Given a job description "Senior DevOps Engineer"
    When the name is made safe for the filing location
    Then it becomes "Senior-DevOps-Engineer"

  @US-01 @in-memory @contract-shape:bounded-change
  Scenario: Building a chosen variant invokes the configured builder recipe
    Given a configured template for the "senior-devops" variant
    When the CV is built
    Then the configured builder is invoked as "just build senior-devops"

  @US-01 @error @in-memory @contract-shape:unbounded-preservation
  Scenario: The build is refused when the variant driver file is missing
    Given a working directory with no driver file for "senior-sre"
    When the CV is built
    Then the build is refused with a missing-driver error
    And no builder is invoked

  @US-03 @error @in-memory @contract-shape:unbounded-preservation
  Scenario: The build is refused when the working directory is missing
    Given a working directory that does not exist
    When the CV is built
    Then the build is refused with a missing-directory error

  @US-01 @error @in-memory @contract-shape:bounded-change
  Scenario: The build reports failure when the builder itself fails
    Given a configured template whose builder fails
    When the CV is built
    Then the build reports a builder-failure error

  @US-03 @error @contract-shape:unbounded-preservation
  Scenario: Filing is refused when the expected PDF is missing
    Given the expected built PDF is absent from the working directory
    When the CV is filed
    Then filing is refused with a copy error and nothing is filed

  @US-03 @contract-shape:unbounded-preservation
  Scenario: The dated working directory is removed after filing
    Given a dated working directory with build artifacts
    When the working directory is cleaned up
    Then the working directory no longer exists
