//! The database password must not survive into a child process.
//!
//! A child inherits the whole environment by default. The PDF viewer, the
//! browser `xdg-open` starts, `git`, `sudo` and `tailscale` would each hold the
//! password in `/proc/<pid>/environ` for their entire lifetime - and a desktop
//! browser's lifetime is the whole session. `command_without_db_password` is
//! the one constructor that strips it; these tests are its proof.

use rusty_cv_creator::child_env::{DB_PASSWORD_ENV, command_without_db_password};
use serial_test::serial;

/// Not a credential shape: no scheme, no userinfo. The scanner in
/// `db_credentials_regression.rs` reads this file like any other.
const SENTINEL: &str = "sentinel-value-that-is-not-a-real-password";

fn child_environment(mut command: std::process::Command) -> String {
    let out = command
        .args(["-c", "env"])
        .output()
        .expect("spawning `sh -c env` should work on any supported platform");
    assert!(out.status.success(), "`sh -c env` failed in the child");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
#[serial]
fn the_database_password_never_reaches_a_child_environment() {
    // Mutates process environment, hence `#[serial]`.
    set_password_in_this_process(SENTINEL);

    let scrubbed = child_environment(command_without_db_password("sh"));

    assert!(
        !scrubbed.contains(DB_PASSWORD_ENV),
        "the child still sees {DB_PASSWORD_ENV} in its environment"
    );
    assert!(
        !scrubbed.contains(SENTINEL),
        "the child still sees the password value in its environment"
    );

    // The parent keeps it: this program needs the password to connect. Only the
    // children are stripped.
    assert_eq!(
        std::env::var(DB_PASSWORD_ENV).ok().as_deref(),
        Some(SENTINEL),
        "scrubbing a child must not disturb this process's own environment"
    );

    clear_password();
}

#[test]
#[serial]
fn an_unscrubbed_child_does_inherit_it_so_the_assertion_is_not_vacuous() {
    // Control. Without the constructor the password is inherited - which is
    // exactly the defect. If this ever stops holding, the test above proves
    // nothing and must be re-examined rather than trusted.
    set_password_in_this_process(SENTINEL);

    let inherited = child_environment(std::process::Command::new("sh"));

    assert!(
        inherited.contains(SENTINEL),
        "a plain `Command::new` child is expected to inherit the environment; \
         if it no longer does, the scrubbing test above is vacuous"
    );

    clear_password();
}

fn set_password_in_this_process(value: &str) {
    std::env::set_var(DB_PASSWORD_ENV, value);
}

fn clear_password() {
    std::env::remove_var(DB_PASSWORD_ENV);
}
