# Slice 04 — Open CV PDF from Selected Row

**Goal**: Press Enter or 'o' on a selected row to open its stored PDF in the OS default viewer; gracefully handle missing files.

## IN scope
- Enter or 'o' in Normal mode (not FilterMode) triggers open action on selected row
- Reuse existing `view_cv_file(path)` helper from `src/helpers.rs`
- If file exists: open in background (non-blocking — TUI stays alive)
- If file not found: status bar shows "File not found: <path>" for 3 seconds, then clears
- If no row selected (empty list): keypress is silently ignored

## OUT scope
- In-TUI PDF preview
- Copy path to clipboard
- Edit application record

## Learning hypothesis
Disproves "the existing `view_cv_file` helper is safe to call from ratatui context without blocking the event loop" if the PDF open causes the TUI to freeze or corrupt terminal state.

## Acceptance criteria
- Enter/o on a valid row opens the PDF and TUI remains interactive
- Enter/o on a row with a missing PDF shows the error in the status bar without exiting
- Enter/o while FilterMode is active does nothing (filter input takes priority)

## Dependencies
- Slice 01 merged (provides `view_cv_file` integration point)
- Slice 02 merged (provides `selected_index`)

## Effort estimate
~1.5 hours | Reference class: "wire keypress action to existing shell helper inside TUI event loop"
