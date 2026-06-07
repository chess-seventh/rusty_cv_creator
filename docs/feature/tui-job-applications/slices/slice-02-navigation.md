# Slice 02 — Keyboard Navigation and Row Highlight

**Goal**: ↑/↓ arrows move a highlighted cursor through the application rows; status bar shows "N of M applications".

## IN scope
- `ratatui::widgets::TableState` tracks selected index
- ↑/↓ (and k/j vim-keys) move selection, wrapping at boundaries
- Selected row rendered with highlight style (bold + reversed colours)
- Status bar format: "3 of 17 applications"
- Home/End keys jump to first/last row

## OUT scope
- Filter bar (slice 03)
- PDF open from selected row (slice 04)

## Learning hypothesis
Disproves "ratatui's `StatefulWidget` Table handles 100+ row lists without lag" if scrolling stutters or panics on large datasets.

## Acceptance criteria
- ↓ on last row stays on last row (no wrap-around panic)
- ↑ on first row stays on first row
- k/j behave identically to ↑/↓
- Home moves to row 0, End moves to last row
- Status bar count is always accurate

## Dependencies
- Slice 01 merged

## Effort estimate
~2 hours | Reference class: "add stateful widget navigation to existing ratatui table"
