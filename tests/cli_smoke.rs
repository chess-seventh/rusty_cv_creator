//! Subprocess smoke tests for the CLI driving port.
//!
//! These spawn the actually-built binary and assert it parses arguments and
//! exits correctly. `--help` short-circuits in clap before any config/DB/build,
//! so these are safe to run without the CV template repo, just/tectonic, or a DB.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rusty_cv_creator"))
}

#[test]
fn test_help_exits_zero_and_shows_about() {
    let out = bin().arg("--help").output().expect("failed to run --help");
    assert!(out.status.success(), "--help should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Generate and save CV"),
        "help should show the program description, got: {stdout}"
    );
}

#[test]
fn test_insert_help_lists_variant_flag() {
    let out = bin()
        .args(["insert", "--help"])
        .output()
        .expect("failed to run insert --help");
    assert!(out.status.success(), "insert --help should exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("--variant"),
        "insert --help should expose the --variant flag, got: {stdout}"
    );
}

#[test]
fn test_no_subcommand_is_an_error() {
    // A subcommand is required; clap should reject invocation with no action
    // and exit non-zero before any side effects.
    let out = bin().output().expect("failed to run with no args");
    assert!(
        !out.status.success(),
        "running with no subcommand should be a usage error"
    );
}
