# Feature: Previewing the generated CV and guarding required tools
#
# RETROACTIVE backfill (LEAN). Documentation SSOT mapped to existing GREEN Rust
# tests (no cucumber-rust harness). See the traceability table in
# docs/feature/cv-variant-build/feature-delta.md (Wave: DISTILL).
#
# @in-memory: the PDF viewer is exercised through the FakeRunner double
#   (no real zathura launch). Tool-availability checks read the real PATH.

Feature: Previewing the generated CV and guarding required tools
  As Francesco, before sending an application
  I want to preview the CV and be warned early about missing tools
  So that I verify it renders and get clear remediation when something is absent

  @US-05 @in-memory @contract-shape:bounded-change
  Scenario: The generated PDF opens in the configured viewer
    Given a generated CV PDF at "/tmp/cv.pdf"
    When Francesco previews the CV with the "zathura" viewer
    Then the viewer is asked to open "/tmp/cv.pdf"

  @US-05 @edge @in-memory @contract-shape:bounded-change
  Scenario: A source document path is previewed as its PDF counterpart
    Given a CV source document at "/tmp/cv.tex"
    When Francesco previews the CV with the "zathura" viewer
    Then the viewer is asked to open "/tmp/cv.pdf"

  @US-05 @error @in-memory @contract-shape:bounded-change
  Scenario: Preview reports an error when the viewer cannot be launched
    Given a generated CV PDF that the viewer cannot open
    When Francesco previews the CV
    Then the preview reports an error

  @US-05 @error @contract-shape:pure-function
  Scenario: Missing required tools fail fast with a devenv hint
    Given a required tool that is not on the system path
    When tool availability is checked before use
    Then the check fails fast naming the missing tool and the devenv remedy

  @US-05 @edge @contract-shape:pure-function
  Scenario: The tool check passes when nothing is required
    Given no tools are required
    When tool availability is checked before use
    Then the check passes

  @US-05 @error @contract-shape:pure-function
  Scenario: A tool absent from the system path is reported unavailable
    Given a tool that is not installed on the system path
    When the tool's availability is queried
    Then it is reported as unavailable

  @US-05 @contract-shape:pure-function
  Scenario: A tool present on the system path is reported available
    Given a tool that is installed on the system path
    When the tool's availability is queried
    Then it is reported as available
