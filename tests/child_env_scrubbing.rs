//! The database credentials must not survive into a child process.
//!
//! A child inherits the whole environment by default. The PDF viewer, the
//! browser `xdg-open` starts, `git`, `sudo` and `tailscale` would each hold the
//! credentials in `/proc/<pid>/environ` for their entire lifetime - and a
//! desktop browser's lifetime is the whole session.
//! `command_without_db_credentials` is the one constructor that strips them;
//! these tests are its proof.
//!
//! Two variables carry a secret, not one. `RUSTY_CV_DB_PASSWORD` is the
//! deliberate delivery mechanism. `DATABASE_URL` is written into this process's
//! own environment at startup from the configured `db_pg_host`, and a config
//! whose URL still carries inline userinfo puts a password there too - which is
//! exactly the state of a config that has not been migrated yet.

use rusty_cv_creator::child_env::{DB_PASSWORD_ENV, DB_URL_ENV, command_without_db_credentials};
use serial_test::serial;

/// Not a credential shape: no scheme, no userinfo. The scanner in
/// `db_credentials_regression.rs` reads this file like any other.
const SENTINEL: &str = "sentinel-value-that-is-not-a-real-password";

/// A URL carrying inline userinfo, assembled at runtime so this source file
/// holds no credential-shaped literal for the scanner to flag.
fn url_with_inline_credentials() -> String {
    format!("postgres://sentinel-user{}{SENTINEL}@localhost/db", ':')
}

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
fn neither_credential_variable_reaches_a_child_environment() {
    // Mutates process environment, hence `#[serial]`.
    let _guard = CredentialEnvGuard::set(SENTINEL, &url_with_inline_credentials());

    let scrubbed = child_environment(command_without_db_credentials("sh"));

    assert!(
        !scrubbed.contains(DB_PASSWORD_ENV),
        "the child still sees {DB_PASSWORD_ENV} in its environment"
    );
    assert!(
        !scrubbed.contains(DB_URL_ENV),
        "the child still sees {DB_URL_ENV} in its environment"
    );
    assert!(
        !scrubbed.contains(SENTINEL),
        "the child still sees a password value in its environment"
    );

    // The parent keeps both: this program needs them to connect. Only the
    // children are stripped.
    assert_eq!(
        std::env::var(DB_PASSWORD_ENV).ok().as_deref(),
        Some(SENTINEL),
        "scrubbing a child must not disturb this process's own environment"
    );
    assert_eq!(
        std::env::var(DB_URL_ENV).ok(),
        Some(url_with_inline_credentials()),
        "scrubbing a child must not disturb this process's own environment"
    );
}

#[test]
#[serial]
fn an_unscrubbed_child_inherits_them_so_the_assertions_are_not_vacuous() {
    // Control. Without the constructor both variables are inherited - which is
    // exactly the defect. If this ever stops holding, the test above proves
    // nothing and must be re-examined rather than trusted.
    let _guard = CredentialEnvGuard::set(SENTINEL, &url_with_inline_credentials());

    let inherited = child_environment(std::process::Command::new("sh"));

    assert!(
        inherited.contains(DB_PASSWORD_ENV),
        "a plain `Command::new` child is expected to inherit {DB_PASSWORD_ENV}; \
         if it no longer does, the scrubbing test above is vacuous"
    );
    assert!(
        inherited.contains(DB_URL_ENV),
        "a plain `Command::new` child is expected to inherit {DB_URL_ENV}; \
         if it no longer does, the scrubbing test above is vacuous"
    );
}

/// Restores both variables on drop, so a panicking assertion cannot leak the
/// sentinel into a sibling test under threaded `cargo test`.
struct CredentialEnvGuard {
    password: Option<String>,
    url: Option<String>,
}

impl CredentialEnvGuard {
    fn set(password: &str, url: &str) -> Self {
        let guard = Self {
            password: std::env::var(DB_PASSWORD_ENV).ok(),
            url: std::env::var(DB_URL_ENV).ok(),
        };
        std::env::set_var(DB_PASSWORD_ENV, password);
        std::env::set_var(DB_URL_ENV, url);
        guard
    }
}

impl Drop for CredentialEnvGuard {
    fn drop(&mut self) {
        match &self.password {
            Some(value) => std::env::set_var(DB_PASSWORD_ENV, value),
            None => std::env::remove_var(DB_PASSWORD_ENV),
        }
        match &self.url {
            Some(value) => std::env::set_var(DB_URL_ENV, value),
            None => std::env::remove_var(DB_URL_ENV),
        }
    }
}
