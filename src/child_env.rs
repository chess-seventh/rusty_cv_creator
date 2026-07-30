//! The single place where this program builds a child process.
//!
//! Lives in the library crate rather than next to `CommandRunner` (which is
//! binary-private) because the TUI's `xdg-open` / `open` spawn is a library
//! call site: the choke point has to be reachable from both crates, and there
//! must be exactly one of it.

use std::process::Command;

/// The environment variable carrying the PostgreSQL password. Canonical here so
/// the name that is scrubbed and the name that is read are one literal.
pub const DB_PASSWORD_ENV: &str = "RUSTY_CV_DB_PASSWORD";

/// Build a `Command` whose child cannot see the database password: the password
/// is delivered to this process only, and any child - a PDF viewer, `xdg-open`
/// and the browser it starts, `git`, `sudo`, `tailscale` - would otherwise carry
/// it in its own environment, readable from /proc, for its whole lifetime.
pub fn command_without_db_password(program: &str) -> Command {
    let mut command = Command::new(program);
    command.env_remove(DB_PASSWORD_ENV);
    command
}
