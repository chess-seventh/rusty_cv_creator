// SCAFFOLD: true
// App orchestrator — owns event loop. Replace in DELIVER slice-01 and slice-02.
// DESIGN constraint: `terminal` field must be declared before `_guard` field.

use crate::tui::state::AppState;
use crate::tui::terminal_guard::TerminalGuard;

pub struct App {
    pub state: AppState,
    // `terminal` must be declared before `_guard` to ensure correct drop order.
    _guard: TerminalGuard,
}

impl App {
    pub fn new(_state: AppState) -> Result<Self, Box<dyn std::error::Error>> {
        panic!("Not yet implemented — RED scaffold");
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        panic!("Not yet implemented — RED scaffold");
    }
}
