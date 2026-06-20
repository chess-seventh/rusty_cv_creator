Feature: TUI job applications — milestone 2: keyboard navigation
  As an active job seeker scanning my applications
  I want to move a highlighted cursor through the table
  So that I can focus on one row at a time and know my position

  @us-02 @in-memory
  Scenario: Moving down increments the selected index
    Given AppState with 5 rows and selected index 0
    When the user presses the down arrow
    Then the selected index is 1
    And the status text shows "2 of 5 applications"

  @us-02 @in-memory
  Scenario: Moving down on the last row does not advance beyond the list
    Given AppState with 5 rows and selected index 4
    When the user presses the down arrow
    Then the selected index is still 4

  @us-02 @in-memory
  Scenario: Moving up decrements the selected index
    Given AppState with 5 rows and selected index 3
    When the user presses the up arrow
    Then the selected index is 2

  @us-02 @in-memory
  Scenario: Moving up on the first row does not go below zero
    Given AppState with 5 rows and selected index 0
    When the user presses the up arrow
    Then the selected index is still 0

  @us-02 @in-memory
  Scenario: Home key jumps to the first row
    Given AppState with 5 rows and selected index 3
    When the user presses the Home key
    Then the selected index is 0

  @us-02 @in-memory
  Scenario: End key jumps to the last row
    Given AppState with 5 rows and selected index 1
    When the user presses the End key
    Then the selected index is 4

  @us-02 @in-memory @property
  Property: Navigation never yields an out-of-bounds index for any list size and any sequence of keypresses
    Given AppState with N rows where N is between 1 and 1000
    When any sequence of up/down/home/end keypresses is applied
    Then the selected index is always between 0 and N minus 1 inclusive

  @us-02 @in-memory @error
  Scenario: Navigation on an empty list is a no-op and does not panic
    Given AppState with 0 rows
    When the user presses the down arrow
    Then the selected index remains 0
    And AppState is in a valid state with no panic

  @us-02 @in-memory @error
  Scenario: Navigation when all rows are filtered out does not panic
    Given AppState with 3 rows and active filter "zzznomatch" showing 0 matches
    When the user presses the down arrow
    Then no index out-of-bounds error occurs
    And the selected index remains 0
