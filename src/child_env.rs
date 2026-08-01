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

/// The connection URL this program writes into its own environment at startup.
/// It is scrubbed from children too: a URL configured with inline userinfo
/// carries a password just as surely as `DB_PASSWORD_ENV` does, and the
/// startup write happens before the first child is spawned.
pub const DB_URL_ENV: &str = "DATABASE_URL";

/// Build a `Command` whose child cannot see the database credentials: they are
/// delivered to this process only, and any child - a PDF viewer, `xdg-open`
/// and the browser it starts, `git`, `sudo`, `tailscale` - would otherwise
/// carry them in its own environment, readable from /proc, for its whole
/// lifetime. A desktop browser's lifetime is the whole session.
pub fn command_without_db_credentials(program: &str) -> Command {
    let mut command = Command::new(program);
    command.env_remove(DB_PASSWORD_ENV);
    command.env_remove(DB_URL_ENV);
    command
}
