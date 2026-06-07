# ADR-001: TUI Library Selection — ratatui

**Status**: Accepted
**Date**: 2026-06-06
**Feature**: `tui-job-applications`
**Deciders**: Morgan (Solution Architect)

---

## Context

The `rusty-cv list` subcommand currently returns a raw debug string with zero application data. The feature `tui-job-applications` requires replacing it with an interactive full-screen terminal table that:

- Renders a scrollable table of `Cv` application records (company, job title, date, PDF path)
- Handles keyboard navigation (arrow keys, Home/End, `q`/Esc to quit)
- Supports a real-time substring filter bar activated by `/`
- Opens a PDF in the OS default viewer on Enter/`o`
- Restores terminal state on all exit paths (normal quit, panic, SIGINT)

**Constraints that shape the decision**:
- The existing codebase already contains `skim` (an `fzf`-like library) which brings `crossterm` in as a transitive dependency. A terminal I/O crate is therefore already present in the dependency tree.
- Zero new _system_ binary dependencies (no external `fzf`, `less`, `bat`). The library must be a pure Rust crate.
- No async runtime. The event loop must be synchronous.
- Target: single-user, local CLI binary. No server, no network, no distributed system. The TUI library needs to be appropriate for this scale.

**Quality attributes** (from System Architecture §3):
1. Startup-to-first-render < 500 ms
2. Terminal portability (crossterm-compatible terminals: xterm-256color, tmux, Windows Terminal, iTerm2, kitty)
3. Correct terminal state restoration on all exit paths
4. Memory footprint < 50 MB RSS at 10 000 rows

---

## Decision

**Use `ratatui 0.29` as the TUI rendering library with `crossterm 0.28` as the terminal backend.**

ratatui is selected as the single rendering layer. crossterm is selected as the terminal I/O adapter. They are used together as a standard pair; ratatui is backend-agnostic but the crossterm backend is the de-facto standard for portable cross-platform Rust TUIs.

---

## Alternatives Considered

### Alternative 1: Raw crossterm (no rendering library)

**Description**: Use `crossterm` directly, without ratatui. Write terminal escape sequences manually for table layout, colour, and cursor positioning.

**Evaluation**:
- Pro: Zero rendering-library dependency; smallest possible binary size increase.
- Con: Table widget, scroll state, border drawing, and status bar all must be implemented from scratch. This is a significant amount of infrastructure code for a feature whose primary value is in the user interaction, not the renderer.
- Con: No layout engine — terminal resize handling requires manual column-width recalculation.
- Con: Substantially higher implementation risk for the walking skeleton. Slice 01 would take 2–3× longer.

**Rejection rationale**: The implementation cost of a correct table renderer from raw crossterm exceeds the marginal benefit of avoiding the ratatui dependency. ratatui provides a proven, tested table widget, scroll state, and layout engine. This is not resume-driven development; it is the correct tool for the problem.

### Alternative 2: `cursive`

**Description**: `cursive` is a Rust TUI framework with a widget model similar to a GUI toolkit (dialog boxes, buttons, text views). It supports multiple backends including crossterm.

**Evaluation**:
- Pro: High-level widget model; dialogs and menus are easy to compose.
- Con: `cursive`'s widget model is designed for dialog-driven UIs, not for high-performance streaming table displays. Rendering a virtualized table of 10 000 rows is not its primary use case.
- Con: `cursive`'s event handling model is callback-based (via closures stored in the widget tree), which conflicts with the clean `AppState` owned-state model designed for this feature. Reasoning about state mutations becomes harder.
- Con: Less widely adopted than ratatui for data-display TUIs. Community momentum and example corpus are weaker for table-centric applications.
- Con: Introduces a heavier widget framework for a feature that needs a simple table, a status bar, and a text input. Overkill.

**Rejection rationale**: `cursive` is appropriate for menu-driven, dialog-based TUIs. This feature is a data-browsing interface, not a dialog-driven UI. ratatui's immediate-mode rendering model is a better fit.

### Alternative 3: `termion`

**Description**: `termion` is a lower-level terminal I/O library for Unix systems. It provides raw mode, alternate screen, and key event reading without a rendering layer.

**Evaluation**:
- Pro: Lightweight; minimal dependency footprint.
- Con: Unix-only. The existing `Cargo.toml` already targets cross-platform (the `open` command is dispatched per OS); adding a Unix-only TUI library would break the portability constraint.
- Con: No rendering layer. Same hand-rolled table problem as Alternative 1, plus the platform portability regression.
- Con: Less actively maintained than ratatui + crossterm.

**Rejection rationale**: Unix-only constraint is a hard veto. The existing codebase supports Linux and macOS at minimum; Windows Terminal support (a crossterm capability) should not be sacrificed.

---

## Consequences

### Positive
- `ratatui` provides a `Table` widget with virtual scroll state, `Block` borders, and `Paragraph` for the status bar out of the box. The walking skeleton (Slice 01) needs ~100–150 lines of rendering code, not ~400.
- `crossterm` is already present as a transitive dependency via `skim`. Adding it as a direct dependency with an explicit version pin has zero net new external code fetched for the crossterm crate itself.
- Immediate-mode rendering model aligns with the owned `AppState` design: each frame, `ui::render(frame, &state)` reads state and draws. No retained widget tree, no closure-stored callbacks, no hidden mutable state.
- Strong community: ratatui has 10 000+ GitHub stars, multiple active maintainers, and is the recommended successor to the archived `tui-rs` crate.
- MIT license — compatible with the project's OSS posture.

### Negative / Trade-offs
- Binary size increases. ratatui + crossterm add approximately 300–400 KB to the compiled release binary. This is acceptable; the system architect's constraint is zero new _system binary_ dependencies (external executables), not zero new Rust crate dependencies.
- ratatui's immediate-mode model requires the application to explicitly track "dirty" state and drive redraws. This is a small amount of bookkeeping (`state.dirty: bool`) but it is the application's responsibility, not the library's.
- ratatui's API is evolving; minor breaking changes between minor versions are possible. Pinning to `0.29` mitigates this for the feature's delivery timeline.

### Constraints This Decision Establishes
- All TUI rendering code lives behind `ratatui` abstractions (`Frame`, `Widget` implementations). No direct ANSI escape sequence strings outside of crossterm.
- crossterm version is pinned to `0.28` to match ratatui 0.29's declared backend compatibility.
- If a future feature requires async event handling (e.g., background DB refresh), the escalation path is `std::thread` + `std::sync::mpsc` channel, not an async runtime. The library choice does not preclude this.
