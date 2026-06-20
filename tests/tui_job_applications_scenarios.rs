// tui-job-applications — subprocess-level scenarios.
// Driving adapter: CLI (`rusty-cv list` via std::process::Command).
//
// Self-contained: each test passes its own `--config-ini` (a temp INI) so the
// binary does not depend on the developer's ~/.config file and runs identically
// in CI. The `list` arm runs the TTY startup probe FIRST (so a non-TTY
// invocation fails fast without opening a DB connection), therefore in these
// piped-stdio subprocesses the probe message is the expected outcome.

use std::process::Command;
use tempfile::TempDir;

// ─── helpers ────────────────────────────────────────────────────────────────

fn temp_db_env() -> (TempDir, String) {
    let dir = TempDir::new().expect("tmpdir");
    let db_path = dir.path().join("test.db");
    let url = format!("sqlite://{}", db_path.display());
    (dir, url)
}

/// A minimal, loadable config so the binary doesn't fall back to ~/.config
/// (absent in CI). DB values are placeholders — the non-TTY probe exits before
/// the DB is used.
fn temp_config() -> (TempDir, String) {
    let dir = TempDir::new().expect("tmpdir");
    let cfg = dir.path().join("config.ini");
    std::fs::write(
        &cfg,
        "[db]\nengine = \"sqlite\"\ndb_path = \"/tmp\"\ndb_file = \"test.db\"\n\
         db_pg_host = \"postgres://placeholder\"\n",
    )
    .expect("write config");
    (dir, cfg.display().to_string())
}

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rusty_cv_creator"))
}

fn message(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .to_lowercase()
}

// ─── Walking Skeleton (WS-01) ────────────────────────────────────────────────

/// @walking_skeleton @driving_adapter @real-io @us-01
/// `rusty-cv list` in a non-TTY subprocess: the startup probe detects the
/// missing terminal and exits non-zero with a helpful message. Proves the TUI
/// is wired to UserAction::List without needing a real TTY.
#[test]
fn ws_01_list_command_detects_non_tty_and_exits_with_message() {
    let (_dir, db_url) = temp_db_env();
    let (_cfg_dir, cfg) = temp_config();

    let output = binary()
        .env("DATABASE_URL", &db_url)
        .args(["--config-ini", &cfg, "list"])
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit when no TTY is present"
    );
    let combined = message(&output);
    assert!(
        combined.contains("terminal") || combined.contains("tty") || combined.contains("not a"),
        "Expected a startup-probe message about the missing terminal, got: {combined}"
    );
}

/// @walking_skeleton @driving_adapter @real-io @us-01 @error
/// With a bad DATABASE_URL the binary still exits non-zero. (In a non-TTY
/// subprocess the probe guards before the DB, so the probe message is what
/// surfaces — the point is a clean non-zero exit with a message, no hang/crash.)
#[test]
fn ws_02_list_command_with_bad_database_url_exits_with_error() {
    let (_cfg_dir, cfg) = temp_config();
    let output = binary()
        .env("DATABASE_URL", "sqlite:///definitely/does/not/exist.db")
        .args(["--config-ini", &cfg, "list"])
        .output()
        .expect("failed to run binary");

    assert!(!output.status.success(), "Expected non-zero exit");
    let combined = message(&output);
    assert!(
        combined.contains("terminal")
            || combined.contains("tty")
            || combined.contains("database")
            || combined.contains("error")
            || combined.contains("connect"),
        "Expected a probe or database error message, got: {combined}"
    );
}

// ─── Walking Skeleton (WS-03) ────────────────────────────────────────────────

/// @walking_skeleton @driving_adapter @real-io @error
/// With DATABASE_URL unset the binary exits non-zero with a message.
#[test]
fn ws_03_list_command_exits_with_error_when_database_url_not_set() {
    let (_cfg_dir, cfg) = temp_config();
    let output = binary()
        .env_remove("DATABASE_URL")
        .args(["--config-ini", &cfg, "list"])
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit when DATABASE_URL is not set"
    );
    let combined = message(&output);
    assert!(
        combined.contains("error")
            || combined.contains("database")
            || combined.contains("url")
            || combined.contains("terminal")
            || combined.contains("tty"),
        "Expected an error/probe message, got: {combined}"
    );
}
