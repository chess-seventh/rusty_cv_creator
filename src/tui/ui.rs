// SCAFFOLD: true
// Pure render function stub — no ratatui import yet.
// DELIVER: add ratatui to Cargo.toml and replace () with &mut ratatui::Frame.
// DESIGN constraint: state parameter must be &AppState (immutable), never &mut.

use crate::tui::state::AppState;

pub fn render(_frame: &mut (), _state: &AppState) {
    panic!("Not yet implemented — RED scaffold");
}
