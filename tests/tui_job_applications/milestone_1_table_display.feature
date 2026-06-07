Feature: TUI job applications — milestone 1: application table display
  As an active job seeker
  I want load_all_applications to return all my recorded applications
  So that the TUI table has data to display

  # Adapter integration — driven internal port (DB)

  @real-io @adapter-integration @us-01
  Scenario: Load all applications returns all records from a populated database
    Given a SQLite database with 3 seeded application records
    When load_all_applications is called
    Then the result contains exactly 3 Cv records
    And each record has a non-empty job_title and company

  @real-io @adapter-integration @us-01
  Scenario: Load all applications returns an empty list for an empty database
    Given an empty SQLite database
    When load_all_applications is called
    Then the result is an empty list

  @real-io @adapter-integration @us-01 @error
  Scenario: Load all applications returns an error when the database is unreachable
    Given DATABASE_URL is set to an invalid connection string
    When load_all_applications is called
    Then the result is an error containing a connection message

  # ApplicationRow projection — unit level

  @us-01 @in-memory
  Scenario: ApplicationRow maps all Cv fields to display-ready types
    Given a Cv record with application_date "2024-03-15", company "Acme Corp", job_title "Rust Engineer", pdf_cv_path "/home/user/cvs/acme.pdf"
    When the Cv record is projected to an ApplicationRow
    Then the ApplicationRow has date "2024-03-15"
    And the ApplicationRow has company "Acme Corp"
    And the ApplicationRow has job_title "Rust Engineer"
    And the ApplicationRow has pdf_path "/home/user/cvs/acme.pdf"

  @us-01 @in-memory
  Scenario: ApplicationRow falls back to "Unknown" when application_date is absent
    Given a Cv record with no application_date, company "Beta Ltd", job_title "Dev"
    When the Cv record is projected to an ApplicationRow
    Then the ApplicationRow has date "Unknown"

  @us-01 @in-memory
  Scenario: ApplicationRow excludes the quote field
    Given a Cv record with a non-empty quote field "I am an excellent candidate"
    When the Cv record is projected to an ApplicationRow
    Then the ApplicationRow has no quote field

  @us-01 @in-memory
  Scenario: TUI enters empty state when application list is empty
    Given the application list is empty
    When AppState is initialised with zero rows
    Then AppState reports empty_state as true
    And the status message is "No applications recorded yet"

  @real-io @adapter-integration @us-01 @error
  Scenario: Load all applications returns an error when DATABASE_URL is not set
    Given DATABASE_URL environment variable is not set
    When load_all_applications is called
    Then the result is an error containing a configuration or connection message
