//! Regression scanner: no credential ever lands in a tracked file again.
//!
//! A plaintext PostgreSQL password lived in two tracked files for two years
//! because nothing in the build, the test suite or the pre-commit hooks could
//! recognise a URL userinfo password. This test is that missing control.
//!
//! Placed at the top level of `tests/` on purpose: cargo auto-discovers test
//! targets from `tests/*.rs` only, so `tests/regression/db_credentials.rs`
//! would never be compiled or run.
//!
//! There is NO allowlist. If a scan hit is legitimate, the fixture gets fixed,
//! not the scanner. That is also why every credential-shaped string in this
//! file is assembled at runtime from fragments: the scanner scans itself.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The burned literal: it is in git history forever and must never be re-added.
/// Assembled at runtime so this file does not carry it.
fn burned_literal() -> String {
    ["rusty", "cv", "01"].join("-")
}

/// Machine-read file kinds where a credential is dangerous. Markdown and other
/// prose are deliberately excluded from the shape scan: the analysis document
/// legitimately describes the defect shape.
const SCANNED_EXTENSIONS: [&str; 7] = ["nix", "ini", "toml", "rs", "yml", "yaml", "sh"];

/// Every file tracked by git, as absolute paths, paired with their repo-relative
/// name for reporting. Fails loudly when git is unavailable: a scanner that
/// silently passes is worse than no scanner.
fn tracked_files() -> Vec<(String, PathBuf)> {
    let root = env!("CARGO_MANIFEST_DIR");

    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("cannot run `git ls-files` in {root}: {e}"));

    assert!(
        output.status.success(),
        "`git ls-files` failed in {root}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let listing = String::from_utf8(output.stdout).expect("git ls-files emitted invalid UTF-8");
    let files: Vec<(String, PathBuf)> = listing
        .split('\0')
        .filter(|name| !name.is_empty())
        .map(|name| (name.to_string(), Path::new(root).join(name)))
        .collect();

    assert!(
        !files.is_empty(),
        "`git ls-files` returned nothing in {root}: the scan would pass vacuously"
    );

    files
}

fn read_text(path: &Path) -> Option<String> {
    std::fs::read(path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

fn is_machine_read_file(name: &str) -> bool {
    let path = Path::new(name);

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if file_name.starts_with(".env") {
        return true;
    }

    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| SCANNED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

/// Structural detection of a password inside a URL userinfo: find `://`, then a
/// `:` before the next `@`, with that `@` before the next `/`. No regex, so no
/// new dependency.
///
/// The shape is deliberately not spelled out as a literal anywhere in this
/// file: once tracked, this file is scanned by its own matcher.
fn has_inline_credential(line: &str) -> bool {
    let mut rest = line;

    while let Some(scheme) = rest.find("://") {
        let authority = &rest[scheme + "://".len()..];

        if let (Some(at), Some(colon)) = (authority.find('@'), authority.find(':')) {
            let ends_before_path = authority.find('/').is_none_or(|slash| at < slash);
            let user = &authority[..colon];
            let password = &authority[colon + 1..at.max(colon + 1)];
            let is_userinfo = colon < at
                && ends_before_path
                && !user.is_empty()
                && !password.is_empty()
                && !user.contains(char::is_whitespace)
                && !password.contains(char::is_whitespace);

            if is_userinfo {
                return true;
            }
        }

        rest = authority;
    }

    false
}

fn offences(hit: impl Fn(&str) -> bool, only_machine_read: bool) -> Vec<String> {
    let mut found = Vec::new();

    for (name, path) in tracked_files() {
        if only_machine_read && !is_machine_read_file(&name) {
            continue;
        }

        let Some(text) = read_text(&path) else {
            found.push(format!("{name}: tracked file could not be read"));
            continue;
        };

        for (index, line) in text.lines().enumerate() {
            if hit(line) {
                found.push(format!("{name}:{}", index + 1));
            }
        }
    }

    found
}

#[test]
fn no_tracked_machine_read_file_carries_an_inline_credential_url() {
    let found = offences(has_inline_credential, true);

    assert!(
        found.is_empty(),
        "a URL carrying a password in its userinfo was found in tracked files:\n  {}\n\
         Remove the credential; the postgres password comes from RUSTY_CV_DB_PASSWORD.",
        found.join("\n  ")
    );
}

#[test]
fn the_burned_credential_literal_never_reappears() {
    let literal = burned_literal();
    let found = offences(|line| line.contains(&literal), false);

    assert!(
        found.is_empty(),
        "the burned credential literal reappeared in tracked files:\n  {}\n\
         That value is in git history forever and must never be re-added.",
        found.join("\n  ")
    );
}

#[test]
fn the_matcher_fires_on_a_known_bad_url() {
    // Assembled from fragments so this file stays scanner-clean.
    let scheme = "postgres";
    let bad = format!("db_url = \"{scheme}://user{}pass@host/db\"", ':');
    assert!(has_inline_credential(&bad), "matcher missed: {bad}");

    let with_encoded_password = format!("{scheme}://user{}p%40ss@host/db", ':');
    assert!(has_inline_credential(&with_encoded_password));
}

#[test]
fn the_matcher_ignores_passwordless_and_pathological_urls() {
    for clean in [
        "postgres://rusty_cv@nixos-02.caracara-palermo.ts.net/rusty_cv",
        "https://github.com/chess-seventh/cv.git",
        "sqlite:///home/user/applications.db",
        "see https://example.invalid/docs: ask bob@example.invalid",
        "no scheme here at all",
    ] {
        assert!(!has_inline_credential(clean), "false positive on: {clean}");
    }
}

#[test]
fn the_scanned_set_covers_the_machine_read_file_kinds() {
    for name in [
        "devenv.nix",
        "rusty-cv-config-example.ini",
        "Cargo.toml",
        "src/config_parse.rs",
        ".github/workflows/build.yml",
        "scripts/thing.yaml",
        "scripts/thing.sh",
        ".env",
        ".env.testing",
    ] {
        assert!(is_machine_read_file(name), "should be scanned: {name}");
    }

    for name in ["README.md", "docs/product/architecture/brief.md", "LICENSE"] {
        assert!(!is_machine_read_file(name), "should not be scanned: {name}");
    }
}
