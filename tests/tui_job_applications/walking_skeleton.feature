Feature: TUI job applications — walking skeleton
  As an active job seeker
  I want `rusty-cv list` to launch a terminal UI showing my applications
  So that I can see where I have applied without leaving the terminal

  @walking_skeleton @driving_adapter @real-io @us-01
  Scenario: List command exits cleanly when invoked via CLI entry point
    Given the rusty-cv binary is installed
    And the database contains 3 recorded applications
    When the user runs `rusty-cv list` in a non-TTY context
    Then the process exits with a non-zero code
    And stdout contains "not a terminal" or "No TTY detected" or similar startup probe message
    And the terminal state is not left in raw mode

  @walking_skeleton @driving_adapter @real-io @us-01 @error
  Scenario: List command with unreachable database exits with a clear error message
    Given the rusty-cv binary is installed
    And DATABASE_URL points to a nonexistent path
    When the user runs `rusty-cv list` in a non-TTY context
    Then the process exits with a non-zero code
    And stdout or stderr contains a database error message

  @walking_skeleton @driving_adapter @real-io @error
  Scenario: List command exits with an error when DATABASE_URL is not configured
    Given the rusty-cv binary is installed
    And DATABASE_URL environment variable is not set
    When the user runs `rusty-cv list` in a non-TTY context
    Then the process exits with a non-zero code
    And stdout or stderr contains a configuration or database error message
