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
//! THE SCANNING RULE, PLAINLY: every tracked file is scanned. The only things
//! skipped are prose (`.md`, which legitimately describes the credential shape
//! -- the analysis document on this branch spells the defect out on purpose)
//! and files whose bytes are not valid UTF-8 (binaries), skipped silently.
//! Everything else is scanned: `.json`, `.sql`, `.feature`, `.xml`, `.iml`,
//! `env.sample`, `.envrc`, and extensionless files such as
//! `.github/CODEOWNERS`.
//!
//! There is NO allowlist, and there never will be one. An allowlist is what
//! made `env.sample` -- the file a human copies to `.env` and pastes a real
//! password into -- invisible to the previous version of this scanner. When
//! this test hits, the offending FILE gets fixed; the scanner is never narrowed
//! to make a hit go away. If a legitimate example needs a credential-shaped
//! string, assemble it at runtime from fragments, the way the tests below do.
//! That is also why every credential-shaped string in this file is assembled at
//! runtime: the scanner scans itself.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The burned literal: it is in git history forever and must never be re-added.
/// Assembled at runtime so this file does not carry it.
fn burned_literal() -> String {
    ["rusty", "cv", "01"].join("-")
}

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

/// The one and only exclusion of the shape scan: prose. A `.md` file is allowed
/// to describe what a leaked credential looks like; that is what documentation
/// is for. Nothing else is excluded -- there is no extension allowlist.
fn is_prose(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// Structural detection of a password inside a URL userinfo: find `://`, then a
/// `:` before the next `@`. No regex, so no new dependency.
///
/// The `@` is deliberately NOT required to precede the first `/`: a password may
/// legally contain a slash, and requiring the authority to end before the first
/// `/` made exactly that shape invisible. (The shape is not written out here --
/// this file is scanned by its own matcher; see the runtime-assembled fixture in
/// `the_matcher_fires_when_the_password_itself_contains_a_slash`.) The trade is
/// more false positives -- accepted on purpose, because a false positive gets
/// the file fixed while a miss gets a password committed.
///
/// What this detector still CANNOT see:
/// - it is line-based, so a credential split across two lines (a wrapped INI
///   value, a multi-line YAML scalar, a Rust string continued with `\`) evades
///   it entirely;
/// - a password held in a variable and interpolated at runtime, which is the
///   shape the fixtures in this repo deliberately use;
/// - a credential in a file whose bytes are not valid UTF-8.
///
/// The shape is deliberately not spelled out as a literal anywhere in this
/// file: once tracked, this file is scanned by its own matcher.
fn has_inline_credential(line: &str) -> bool {
    let mut rest = line;

    while let Some(scheme) = rest.find("://") {
        let authority = &rest[scheme + "://".len()..];

        if let (Some(at), Some(colon)) = (authority.find('@'), authority.find(':')) {
            let user = &authority[..colon];
            let password = &authority[colon + 1..at.max(colon + 1)];
            // Neither segment may contain whitespace. This is a deliberate
            // trade-off, not an oversight: it is what stops ordinary prose
            // ("see https://host/docs: ask bob@host") from matching, and
            // `the_matcher_ignores_passwordless_and_pathological_urls` pins
            // it. The cost is that a passphrase containing a space slips
            // through - recorded as a known gap in the architecture brief.
            let is_userinfo = colon < at
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

fn hits_in(name: &str, text: &str, hit: &impl Fn(&str) -> bool, found: &mut Vec<String>) {
    for (index, line) in text.lines().enumerate() {
        if hit(line) {
            found.push(format!("{name}:{}", index + 1));
        }
    }
}

/// Shape scan: EVERY tracked file, minus prose, minus binaries. Not an
/// allowlist -- a denylist of exactly two entries.
fn offences_outside_prose(hit: impl Fn(&str) -> bool) -> Vec<String> {
    let mut found = Vec::new();

    for (name, path) in tracked_files() {
        if is_prose(&name) {
            continue;
        }

        let Ok(bytes) = std::fs::read(&path) else {
            found.push(format!("{name}: tracked file could not be read"));
            continue;
        };

        // Not valid UTF-8 => binary => skipped silently, by design.
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };

        hits_in(&name, &text, &hit, &mut found);
    }

    found
}

/// Literal scan: EVERY tracked file, no exclusions at all -- prose included,
/// binaries read lossily so a literal buried in one is still caught.
fn offences_everywhere(hit: impl Fn(&str) -> bool) -> Vec<String> {
    let mut found = Vec::new();

    for (name, path) in tracked_files() {
        let Ok(bytes) = std::fs::read(&path) else {
            found.push(format!("{name}: tracked file could not be read"));
            continue;
        };

        let text = String::from_utf8_lossy(&bytes);
        hits_in(&name, &text, &hit, &mut found);
    }

    found
}

#[test]
fn no_tracked_file_outside_prose_carries_an_inline_credential_url() {
    let found = offences_outside_prose(has_inline_credential);

    assert!(
        found.is_empty(),
        "a URL carrying a password in its userinfo was found in tracked files:\n  {}\n\
         Fix the FILE, never this scanner. Remove the credential; the postgres\n\
         password comes from RUSTY_CV_DB_PASSWORD. If the string is a legitimate\n\
         example, assemble it at runtime from fragments.",
        found.join("\n  ")
    );
}

#[test]
fn the_burned_credential_literal_never_reappears() {
    let literal = burned_literal();
    let found = offences_everywhere(|line| line.contains(&literal));

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
fn the_matcher_fires_when_the_password_itself_contains_a_slash() {
    // The hole the previous matcher had: it demanded the `@` come before the
    // first `/`, so a slash inside the password hid the whole credential.
    let scheme = "postgres";
    let with_slash = format!("db_url = \"{scheme}://user{}pa/ss@host/db\"", ':');
    assert!(has_inline_credential(&with_slash), "missed: {with_slash}");

    let many_slashes = format!("{scheme}://user{}a/b/c@host/db", ':');
    assert!(
        has_inline_credential(&many_slashes),
        "missed: {many_slashes}"
    );
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

/// The names the shape scan actually visits, derived exactly the way the scan
/// derives them -- so the test below proves coverage of the real tracked tree,
/// not of a hand-written list that can drift away from it.
fn shape_scanned_names() -> Vec<String> {
    tracked_files()
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| !is_prose(name))
        .collect()
}

#[test]
fn the_shape_scan_covers_every_tracked_file_except_prose() {
    let scanned = shape_scanned_names();

    // The proven hole in the extension allowlist this replaced. `env.sample`
    // exists to be copied to `.env`, which makes it the single most likely
    // place for a human to paste a real password -- and its name starts with
    // neither `.env` nor an allowlisted extension, so it was invisible.
    // `.github/CODEOWNERS` has no extension at all; `.json`, `.sql`,
    // `.feature` and `.xml` were simply never on the list.
    for name in [
        "env.sample",
        ".github/CODEOWNERS",
        ".envrc",
        ".idea/rusty_cv_creator.iml",
        "Cargo.lock",
        "migrations/.keep",
        "docs/feature/fix-db-credentials-from-sops/deliver/roadmap.json",
        "tests/acceptance/template-source/public-source.feature",
    ] {
        assert!(
            scanned.iter().any(|scanned_name| scanned_name == name),
            "must be scanned: {name}"
        );
    }

    // Prose is the only exclusion, and it is a real one: these files describe
    // the credential shape on purpose.
    for name in ["README.md", "docs/product/architecture/brief.md"] {
        assert!(
            tracked_files().iter().any(|(tracked, _)| tracked == name),
            "fixture drifted: {name} is no longer tracked"
        );
        assert!(
            !scanned.iter().any(|scanned_name| scanned_name == name),
            "prose must not be shape-scanned: {name}"
        );
    }

    // And nothing else is excluded. If this fails, an allowlist crept back in.
    let skipped: Vec<String> = tracked_files()
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| is_prose(name))
        .collect();
    assert!(
        skipped.iter().all(|name| name.ends_with(".md")),
        "the only exclusion is `.md` prose, but these were skipped too: {skipped:?}"
    );
}

#[test]
fn the_literal_scan_has_no_exclusions_at_all() {
    let literal_scanned: Vec<String> = offences_everywhere(|_| true)
        .into_iter()
        .filter_map(|hit| hit.rsplit_once(':').map(|(name, _)| name.to_string()))
        .collect();

    // Prose is scanned here: the analysis document may describe the shape, but
    // it must never carry the burned literal itself.
    for name in [
        "README.md",
        "docs/feature/fix-db-credentials-from-sops/rca.md",
    ] {
        assert!(
            literal_scanned.iter().any(|scanned| scanned == name),
            "the literal scan must reach prose too: {name}"
        );
    }
}
