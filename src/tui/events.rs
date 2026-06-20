use crate::tui::state::AppState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::Path;
use std::process::Command;

/// Returns Ok(true) to signal quit, Ok(false) to continue.
pub fn handle_key_event(
    state: &mut AppState,
    key: KeyEvent,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::tui::state::Mode;

    match state.mode {
        Mode::Normal => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return Ok(true)
            }
            KeyCode::Down | KeyCode::Char('j') => state.move_down(),
            KeyCode::Up | KeyCode::Char('k') => state.move_up(),
            KeyCode::Home => state.move_to_first(),
            KeyCode::End => state.move_to_last(),
            KeyCode::Char('/') => state.mode = Mode::Filter,
            KeyCode::Enter => {
                if let Some(row) = state.selected_row() {
                    let path = row.pdf_path.clone();
                    let _ = open_pdf(&path);
                }
            }
            _ => {}
        },
        Mode::Filter => match key.code {
            KeyCode::Esc => {
                state.clear_filter();
                state.mode = Mode::Normal;
            }
            KeyCode::Enter => state.mode = Mode::Normal,
            KeyCode::Backspace => {
                let mut text = state.filter_text.clone();
                text.pop();
                state.set_filter(&text);
            }
            KeyCode::Char(c) => {
                let mut text = state.filter_text.clone();
                text.push(c);
                state.set_filter(&text);
            }
            _ => {}
        },
    }
    Ok(false)
}

/// Open `path` in the OS default viewer (non-blocking).
/// Returns Err("File not found: {path}") when the file does not exist, is empty, or is a directory.
pub fn open_pdf(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err(format!("File not found: {path}"));
    }
    let p = Path::new(path);
    if !p.exists() || p.is_dir() {
        return Err(format!("File not found: {path}"));
    }
    #[cfg(target_os = "macos")]
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to open {path}: {e}"))?;
    #[cfg(target_os = "linux")]
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|e| format!("Failed to open {path}: {e}"))?;
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err(format!("open_pdf not supported on this platform: {path}"));
    Ok(())
}
