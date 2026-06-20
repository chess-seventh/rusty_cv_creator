use crossterm::tty::IsTty;
use std::io;

pub fn run_startup_probe() -> Result<(), String> {
    if !io::stdin().is_tty() {
        return Err("Error: terminal required — not a tty".to_string());
    }
    Ok(())
}
