Feature: TUI job applications — milestone 4: open CV PDF
  As an active job seeker preparing for an interview
  I want to press Enter on an application row to open its stored PDF
  So that I can review what the interviewer has read without leaving the terminal

  @us-04 @in-memory @error
  Scenario: Open PDF action with a nonexistent path returns a file-not-found error
    Given an ApplicationRow with pdf_path "/tmp/does_not_exist_12345.pdf"
    When open_pdf is called with that path
    Then the result is an error containing "File not found"
    And the error message includes the path

  @us-04 @in-memory
  Scenario: Open PDF action is ignored when no row is selected
    Given AppState with an empty application list
    When the user presses the Enter key
    Then no open_pdf call is made

  @us-04 @in-memory
  Scenario: Open PDF action is ignored while FilterMode is active
    Given AppState in FilterMode with a selected row
    When the user presses the Enter key
    Then no open_pdf call is made
    And the filter text receives the keypress character instead

  @us-04 @in-memory
  Scenario: Open PDF action in normal mode calls open_pdf with the selected row path
    Given AppState in NormalMode with 3 rows, selected index 1
    And the row at index 1 has pdf_path "/tmp/test_cv.pdf"
    When the user presses the Enter key
    Then open_pdf is called with "/tmp/test_cv.pdf"

  @us-04 @in-memory @error
  Scenario: Open PDF with an empty path string returns a file-not-found error
    Given an ApplicationRow with pdf_path ""
    When open_pdf is called with an empty string path
    Then the result is an error containing "File not found" or "empty path"

  @us-04 @in-memory @error
  Scenario: Open PDF with a directory path instead of a file returns an error
    Given a path "/tmp" which exists but is a directory, not a PDF file
    When open_pdf is called with that path
    Then the result is an error
    And the error message includes the path
