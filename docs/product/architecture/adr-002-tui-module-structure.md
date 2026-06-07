# ADR-002: `src/tui/` Internal Module Structure

**Status**: Accepted
**Date**: 2026-06-06
**Feature**: `tui-job-applications`
**Deciders**: Morgan (Solution Architect)

---

## Context

The feature `tui-job-applications` adds a new interactive terminal UI to `rusty_cv_creator`. The binary is a modular monolith; the TUI code is a new Rust module at `src/tui/`. The decision here concerns how that module is internally structured: how many files, what each file owns, and how they relate.

**Forces acting on this decision**:

1. **Correct terminal state restoration is a safety invariant.** `crossterm::terminal::enable_raw_mode()` and `crossterm::execute!(stdout, EnterAlternateScreen)` must be undone on _every_ exit path: normal quit, `q`/Esc, panic, SIGINT, SIGTERM. Any structure that makes it possible to bypass teardown is unsafe.

2. **The rendering function must be independently testable.** ratatui's `Frame` is an in-process struct that can be constructed in tests without a real terminal. If rendering logic is co-located with event-handling or state-mutation logic, testing the render output requires simulating the event loop. Separation enables direct `render(&state)` unit tests.

3. **State mutation must be traceable.** The table cursor, filter string, filter mode, scroll offset, and status message are all mutable state. If mutations are scattered across multiple files with no clear ownership, debugging incorrect cursor behaviour requires reading multiple files simultaneously.

4. **The TUI module must not grow into the write path.** `src/tui/` must never import `src/cv_insert.rs`. An accidental import would create a coupling between the display context and the write path that the domain model explicitly forbids. Module structure should make the correct thing easy and the wrong thing hard.

5. **The codebase is OOP-leaning (structs + `impl`).** The module structure should produce natural `impl` blocks, not free-function files with deeply nested types.

---

## Decision

Structure `src/tui/` as six files with single-responsibility ownership:

```
src/tui/
├── mod.rs            — module root, exports pub fn run()
├── app.rs            — App struct: composition root, event loop driver
├── state.rs          — AppState struct, ApplicationRow read model, Cv→ApplicationRow projection
├── ui.rs             — pub fn render(frame, &AppState): pure rendering, no side effects
├── events.rs         — pub fn handle_key(event, &mut AppState), open_pdf()
├── terminal_guard.rs — TerminalGuard RAII struct, impl Drop
└── probe.rs          — pub fn run_startup_probe(): Earned Trust substrate verification
```

### Responsibility assignments

**`mod.rs`** — Entry point only. Declares the six sub-modules. Exports `pub fn run(filters: FilterArgs) -> Result<(), Box<dyn std::error::Error>>`. No logic; delegates to `App::new(filters)?.run()`. This keeps the public interface of the module to a single function, making the CLI integration point clear.

**`app.rs`** — The composition root. `App::new()` creates the `TerminalGuard`, runs `probe::run_startup_probe()`, calls `database::load_all_applications()`, constructs `AppState`, and then enters the event loop. The event loop calls `ui::render()` when `state.dirty` is true and dispatches `crossterm::event::read()` results to `events::handle_key()`. Registers the `ctrlc` SIGINT handler (sets an `AtomicBool` flag checked each tick). `App` owns the `ratatui::Terminal` instance and the `TerminalGuard`. Lifetime of the terminal is coextensive with `App`.

**`state.rs`** — Owns all mutable TUI state. `AppState` contains: `rows: Vec<ApplicationRow>` (full loaded dataset), `filtered: Vec<usize>` (indices into `rows` matching the current filter), `cursor: usize` (position in `filtered`), `scroll_offset: usize`, `filter_string: String`, `mode: TuiMode` (enum: `Normal` | `Filter`), `status_message: Option<(String, u8)>` (message + remaining ticks before clear). `ApplicationRow` is defined here alongside its projection function `fn project(cv: Cv) -> ApplicationRow`. Filter logic (`fn apply_filter(&mut self)`) lives here because it reads and writes `AppState` fields atomically. No I/O in this file.

**`ui.rs`** — Pure function. `pub fn render(frame: &mut Frame, state: &AppState)`. Reads `AppState`, draws the ratatui `Table` widget, `Block` borders, status bar `Paragraph`, and filter bar `Paragraph`. No writes to `AppState`. No I/O. This function can be called from a unit test by constructing a `TestBackend` terminal and asserting on the rendered buffer. The purity constraint is enforced by the function signature: the parameter is `&AppState`, not `&mut AppState`.

**`events.rs`** — Keyboard event handler and PDF open adapter. `pub fn handle_key(event: KeyEvent, state: &mut AppState)` is a match tree over `KeyCode`. It is the only place that writes to `AppState` from outside `state.rs` itself (via `&mut`). `pub fn open_pdf(path: &str) -> Result<(), String>` performs the filesystem existence check (`std::fs::metadata`) and spawns the OS viewer process (`std::process::Command`). Platform dispatch is contained here.

**`terminal_guard.rs`** — The RAII terminal lifecycle guard.

```
pub struct TerminalGuard {
    _private: ()  // zero-size; prevents external construction without new()
}

impl TerminalGuard {
    pub fn new() -> Result<Self, crossterm::ErrorKind> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        Ok(Self { _private: () })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        let _ = crossterm::terminal::disable_raw_mode();
    }
}
```

The `Drop` implementation uses `let _ = ...` (ignores errors) because `drop` cannot return a `Result`. This is intentional: teardown must be attempted unconditionally even if it partially fails. The result of partial teardown (e.g., only one of the two calls succeeds) is better than no teardown.

`TerminalGuard` is created by `App::new()` and held as a field. Because Rust drops struct fields in declaration order (reverse), `TerminalGuard` should be declared after `Terminal` in `App` to ensure the terminal is dropped before raw mode is disabled. The correct declaration order is documented here as a maintenance constraint.

**`probe.rs`** — Startup probe (Earned Trust). Runs before the event loop. Verifies:
1. `crossterm::terminal::is_raw_mode_enabled()` returns `true` after `TerminalGuard::new()`. Fails with `StartupError::NotATty` if false (stdout is piped, not a TTY).
2. `crossterm::terminal::size()` returns `Ok((w, h))` with `w > 0` and `h > 0`. Fails with `StartupError::TerminalSizeUnavailable`.
3. A DB reachability check: executes `SELECT COUNT(*) FROM cv LIMIT 1`. Fails with `StartupError::DbUnreachable` on any error.

On failure, emits a structured log event `health.startup.refused: <reason>` via `log::error!` and returns `Err`. The caller (`App::new()`) propagates the error to `main`, which prints a user-facing error message and exits with a non-zero code.

---

## Alternatives Considered

### Alternative 1: Single-file `src/tui.rs`

**Description**: Place all TUI logic — state, rendering, events, guard, probe — in a single `src/tui.rs` file.

**Evaluation**:
- Pro: Simplest possible file layout. No module tree to navigate.
- Con: The file would be 600–900 lines for a complete four-slice implementation. Navigation and review become difficult.
- Con: The rendering function cannot be tested independently without importing the entire event-handling namespace. Test isolation degrades.
- Con: The `TerminalGuard` RAII pattern is harder to review in a large file where it is buried among rendering and event-handling code. The teardown safety invariant is less visible.
- Con: Future features (status column, detail panel, export) would require modifying a single large file, increasing merge conflict probability.

**Rejection rationale**: A single file satisfies the walking skeleton (Slice 01) but does not scale to the full four-slice feature without significant internal disorganisation. The six-file structure costs one additional file-navigation step but pays compound maintainability dividends.

### Alternative 2: `app.rs` + `ui.rs` only (two-file structure)

**Description**: `app.rs` owns state, event loop, probe, and guard. `ui.rs` owns rendering.

**Evaluation**:
- Pro: Simpler than six files. Fewer files to navigate for Slice 01.
- Con: `app.rs` becomes a 400+ line file mixing composition root concerns, state mutation, signal handling, and probe logic. The separation of concerns is nominal (split into two files) but the larger file still has too many reasons to change.
- Con: `TerminalGuard` is not a distinct type with its own file — its `Drop` contract is buried in `app.rs`, making the AST pre-commit hook (which verifies `impl Drop for TerminalGuard` is present) harder to implement reliably.
- Con: The `state.rs` ownership model is lost. Filtering logic and `ApplicationRow` projection end up in `app.rs`, mixing data model concerns with the composition root.

**Rejection rationale**: The two-file structure defers the same problems it claims to avoid by one slice. The six-file structure is chosen now because the full feature scope (four slices) is known; over-indexing on simplicity for Slice 01 at the cost of Slice 03–04 restructuring is the wrong trade-off.

### Alternative 3: Trait-based port abstractions (`DatabasePort`, `PdfOpenPort`)

**Description**: Define Rust traits for the database adapter and the PDF open adapter. Pass them into `App` as generic parameters or `Box<dyn Trait>`. This enables mock injection in tests.

**Evaluation**:
- Pro: Textbook ports-and-adapters. Maximum testability.
- Con: The codebase has zero existing trait-based ports. The database layer uses concrete Diesel connections directly. Introducing traits here would create a design gap between the new module and the rest of the codebase.
- Con: For a local-only binary with a single DB adapter and a single PDF open mechanism, the trait indirection adds two files (`src/tui/ports.rs`, `src/tui/adapters.rs`) and generic type parameters throughout `App` for a testability benefit that can be achieved more simply: `load_all_applications` returns `Result<Vec<Cv>, ...>` — the function can be tested directly with a test DB fixture without a trait.
- Con: `Box<dyn Trait>` introduces dynamic dispatch overhead for a code path that runs once per TUI session (DB load) or on user keypress (PDF open). Irrelevant for performance, but adds cognitive overhead for no gain.

**Rejection rationale**: Trait-based ports are the correct pattern for systems that need multiple adapter implementations (e.g., test mock vs production). This system has one of each. The concrete function call is simpler, testable via integration test with a fixture DB, and consistent with the rest of the codebase. If a future feature needs a mock DB, the refactor to introduce a trait at that point is straightforward.

---

## Consequences

### Positive
- `ui::render()` is a pure function with `&AppState` input. It can be unit-tested with ratatui's `TestBackend` without a real terminal.
- `TerminalGuard` is a dedicated type in a dedicated file. The pre-commit AST hook that verifies `impl Drop for TerminalGuard` exists is trivial to implement: check that `terminal_guard.rs` contains the string `impl Drop for TerminalGuard`.
- `state.rs` is the single file that owns all mutable TUI state. A bug in cursor positioning or filter logic is always found and fixed in one file.
- `probe.rs` is independently loadable and testable. CI can invoke `run_startup_probe()` in a controlled environment (piped stdout, test DB) to verify all three probe assertions without running the full event loop.
- The six-file boundary makes the import constraint enforced by inspection: `src/tui/` files import from `src/database.rs` and `src/models.rs` only. A grep for `cv_insert` in `src/tui/` yields zero results by construction.

### Negative / Trade-offs
- Six files for a feature that could ship as two is over-engineered for Slice 01 alone. The software-crafter implementing Slice 01 should treat the full six-file structure as the target and scaffold all files (even if initially mostly empty stubs) to avoid a disruptive restructure in Slice 03.
- `App` in `app.rs` owns both the event loop and the composition root setup (`TerminalGuard`, probe, DB load). A future concern (e.g., background DB refresh thread) would require extracting the event loop into a separate struct. This is acceptable for the current feature scope; the architecture is designed to accommodate that extraction without affecting other files.

### Constraints This Decision Establishes for the Software-Crafter
- `TerminalGuard` MUST be a distinct named struct with a `Drop` impl. It must not be inlined as an anonymous RAII block.
- `App` struct field declaration order: `_guard: TerminalGuard` must appear AFTER `terminal: Terminal<CrosstermBackend<Stdout>>` to ensure drop order is correct (terminal dropped before raw mode disabled).
- `ui::render()` MUST take `&AppState` (immutable). Any design that requires `&mut AppState` in the render function is a violation of the separation-of-concerns constraint and must be escalated.
- No `unwrap()` or `expect()` in `src/tui/*.rs`. All error paths return `Result` or update `AppState.status_message`. Enforced via `clippy::unwrap_used` lint applied to the `tui` module.
