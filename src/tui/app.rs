use crate::tui::events::handle_key_event;
use crate::tui::state::AppState;
use crate::tui::terminal_guard::TerminalGuard;
use crossterm::event::{self, Event};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;

pub struct App {
    pub state: AppState,
    // `terminal` declared before `_guard` — terminal drops before guard restores raw mode.
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    _guard: TerminalGuard,
}

impl App {
    pub fn new(state: AppState) -> Result<Self, Box<dyn std::error::Error>> {
        let guard = TerminalGuard::new()?;
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            state,
            terminal,
            _guard: guard,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let App {
                state, terminal, ..
            } = self;
            terminal.draw(|frame| {
                crate::tui::ui::render(frame, state);
            })?;

            if let Event::Key(key) = event::read()? {
                if handle_key_event(&mut self.state, key)? {
                    return Ok(());
                }
            }
        }
    }
}
