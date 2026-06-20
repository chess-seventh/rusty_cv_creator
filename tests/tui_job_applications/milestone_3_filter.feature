Feature: TUI job applications — milestone 3: real-time filter
  As an active job seeker about to apply somewhere new
  I want to type in a filter bar and see matching applications instantly
  So that I can confirm I have not already applied there

  @us-03 @in-memory
  Scenario: Empty filter text returns all rows
    Given AppState with 5 rows
    When the filter text is set to an empty string
    Then the filtered list contains all 5 rows

  @us-03 @in-memory
  Scenario: Filter matches company name case-insensitively
    Given AppState with rows for "Acme Corp", "Beta Systems", "acme labs"
    When the filter text is set to "acme"
    Then the filtered list contains 2 rows
    And the filtered rows are "Acme Corp" and "acme labs"

  @us-03 @in-memory
  Scenario: Filter matches job title case-insensitively
    Given AppState with rows for job titles "Rust Engineer", "rust developer", "Python Dev"
    When the filter text is set to "rust"
    Then the filtered list contains 2 rows

  @us-03 @in-memory
  Scenario: Filter with no matches returns an empty list
    Given AppState with rows for "Acme Corp", "Beta Systems"
    When the filter text is set to "zzznomatch"
    Then the filtered list is empty

  @us-03 @in-memory
  Scenario: Clearing the filter restores the full list
    Given AppState with 5 rows and active filter "acme" showing 2 matches
    When the filter text is cleared
    Then the filtered list contains all 5 rows
    And the selected index resets to 0

  @us-03 @in-memory
  Scenario: Navigation works within filtered results
    Given AppState with 3 filtered rows after applying filter "rust"
    When the user presses the down arrow twice
    Then the selected index within the filtered list is 2

  @us-03 @in-memory @error
  Scenario: Filter input with only whitespace is treated as empty and returns all rows
    Given AppState with 5 rows
    When the filter text is set to "   "
    Then the filtered list contains all 5 rows

  @us-03 @in-memory @error
  Scenario: Filter input with special regex characters does not cause a panic
    Given AppState with rows for "Acme Corp", "Beta Systems"
    When the filter text is set to ".*[invalid["
    Then no panic occurs
    And the filtered list contains 0 or more rows

  @us-03 @in-memory @property
  Property: Filtered count is always less than or equal to the total row count
    Given AppState with N rows where N is between 0 and 500
    When any filter text is applied
    Then the filtered count is always between 0 and N inclusive

  @us-03 @in-memory @property
  Property: Filtering with empty string always returns the full list
    Given AppState with N rows where N is between 0 and 500
    When the filter text is set to an empty string
    Then the filtered count equals N
