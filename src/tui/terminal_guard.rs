// SCAFFOLD: true
// RAII terminal state guard stub — no crossterm import yet.
// DELIVER: add crossterm to Cargo.toml and implement enable_raw_mode + LeaveAlternateScreen.
// DESIGN constraint: `terminal` field MUST be declared before `_guard` field in App struct.

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        panic!("Not yet implemented — RED scaffold");
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // DELIVER: call crossterm::terminal::disable_raw_mode() + LeaveAlternateScreen here.
        // Must not panic in Drop — use let _ = ... to swallow errors.
    }
}
