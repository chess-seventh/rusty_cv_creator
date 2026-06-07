// SCAFFOLD: true
// Event handling stubs — no crossterm import yet.
// DELIVER: add crossterm to Cargo.toml and replace KeyEventStub with crossterm::event::KeyEvent.

use crate::tui::state::AppState;

/// Stub for crossterm::event::KeyEvent — replaced with real type in DELIVER.
pub struct KeyEventStub;

pub fn handle_key_event(
    _state: &mut AppState,
    _key: KeyEventStub,
) -> Result<bool, Box<dyn std::error::Error>> {
    panic!("Not yet implemented — RED scaffold");
}

/// Open the PDF at `path` in the OS default viewer (non-blocking spawn).
/// Returns Err with "File not found: <path>" if the file does not exist.
/// DESIGN: no imports from crate::cv_insert.
pub fn open_pdf(_path: &str) -> Result<(), String> {
    panic!("Not yet implemented — RED scaffold");
}
