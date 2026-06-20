# Slice 03 — Real-time Filter by Company / Job Title

**Goal**: Press '/' to open a filter bar; typing narrows the table in real-time; Esc clears and restores full list.

## IN scope
- '/' keypress enters FilterMode; renders a "Filter: _" input bar at bottom
- Each keystroke re-computes `filtered_applications` (client-side, case-insensitive substring match on company + job_title)
- Table re-renders with filtered rows; navigation state resets to row 0
- Filter bar shows match count: "Filter: acme [4 matches]"
- Esc clears filter, restores full list, exits FilterMode
- Backspace removes last character from filter text
- Any printable ASCII character appends to filter text

## OUT scope
- Regex filter
- DB-side filtering
- Filter by date or PDF path

## Learning hypothesis
Disproves "client-side filtering adds unacceptable state management complexity" if the filter state requires more than ~50 lines of new application state code.

## Acceptance criteria
- '/' from Normal mode enters FilterMode
- Typing 'rust' narrows table to rows where company or job_title contains 'rust' (case-insensitive)
- Esc restores full list and resets selection to row 0
- Filter bar shows correct match count
- Navigation (↑/↓) works within filtered results

## Dependencies
- Slice 02 merged

## Effort estimate
~3 hours | Reference class: "add input mode + client-side filter to ratatui stateful table"
