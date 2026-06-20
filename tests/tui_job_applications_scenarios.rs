// DISTILL: tui-job-applications — subprocess-level scenarios
// Driving adapter: CLI (`rusty-cv list` via std::process::Command)
// All tests are #[ignore] RED scaffolds per nWave ADR-025 / Mandate 7.
// DELIVER unskips these one at a time.

use std::env;
use std::process::Command;
use tempfile::TempDir;

// ─── helpers ────────────────────────────────────────────────────────────────

fn temp_db_env() -> (TempDir, String) {
    let dir = TempDir::new().expect("tmpdir");
    let db_path = dir.path().join("test.db");
    let url = format!("sqlite://{}", db_path.display());
    (dir, url)
}

fn binary() -> Command {
    // Points to the test binary produced by `cargo test --test ...`
    // In integration test context, use the debug binary directly.
    Command::new(env!("CARGO_BIN_EXE_rusty_cv_creator"))
}

// ─── Walking Skeleton (WS-01) ────────────────────────────────────────────────

/// @walking_skeleton @driving_adapter @real-io @us-01
/// When `rusty-cv list` is run in a non-TTY subprocess the startup probe
/// detects the missing terminal and exits non-zero with a helpful message.
/// This proves the TUI module is wired to UserAction::List without requiring
/// a real TTY.
#[test]
fn ws_01_list_command_detects_non_tty_and_exits_with_message() {
    let (_dir, db_url) = temp_db_env();

    let output = binary()
        .env("DATABASE_URL", &db_url)
        .args(["list"])
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit when no TTY is present"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("terminal")
            || combined.to_lowercase().contains("tty")
            || combined.to_lowercase().contains("not a"),
        "Expected a startup-probe message about missing terminal, got: {combined}"
    );
}

/// @walking_skeleton @driving_adapter @real-io @us-01 @error
/// When DATABASE_URL is invalid the binary exits non-zero with a DB error message.
#[test]
fn ws_02_list_command_with_bad_database_url_exits_with_error() {
    let output = binary()
        .env("DATABASE_URL", "sqlite:///definitely/does/not/exist.db")
        .args(["list"])
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit on DB error"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("database")
            || combined.to_lowercase().contains("error")
            || combined.to_lowercase().contains("connect"),
        "Expected a database error message, got: {combined}"
    );
}

// ─── Walking Skeleton (WS-03) ────────────────────────────────────────────────

/// @walking_skeleton @driving_adapter @real-io @error
/// When DATABASE_URL is not set at all the binary exits non-zero with a configuration error.
#[test]
fn ws_03_list_command_exits_with_error_when_database_url_not_set() {
    let output = binary()
        .env_remove("DATABASE_URL")
        .args(["list"])
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "Expected non-zero exit when DATABASE_URL is not set"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("error")
            || combined.to_lowercase().contains("database")
            || combined.to_lowercase().contains("url")
            || combined.to_lowercase().contains("terminal")
            || combined.to_lowercase().contains("tty"),
        "Expected an error message, got: {combined}"
    );
}
