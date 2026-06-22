//! template-source — subprocess driving-adapter scenarios.
//!
//! Driving adapter: the existing `insert` CLI entry (`rusty_cv_creator insert`),
//! spawned as a real process. Sourcing is transparent inside the command (D6),
//! so this exercises the user's real invocation path end-to-end through the CLI.
//!
//! The `@real-io` git happy path (a real `git clone` from a `file://` bare-repo
//! fixture) is covered at the `create_directory` seam by the in-crate walking
//! skeleton (`src/file_handlers.rs::walking_skeleton_github_source_resolves_template_dir`)
//! and the in-crate `distill_specs`, where the binary-private `TemplateSource` /
//! `FakeRunner` symbols are reachable — they are NOT on the library facade
//! (`lib.rs` exposes only `database`/`models`/`schema`/`tui`), so an external
//! integration crate cannot drive the git adapter directly.
//!
//! Run under devenv so `git`/`just`/`tectonic` (ADR-0004 pre-usage checks) are on
//! PATH: `devenv shell -- cargo test --test template_source_scenarios`.

use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rusty_cv_creator"))
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Write a minimal, self-contained INI whose `[cv] cv_template_path` is `value`,
/// with a real tmp destination so the run reaches template detection.
fn config_with_template_value(dir: &Path, value: &str) -> String {
    let dest = dir.join("dest");
    let out = dir.join("out");
    std::fs::create_dir_all(&dest).unwrap();
    let cfg = dir.join("conf.ini");
    std::fs::write(
        &cfg,
        format!(
            "[cv]\ncv_template_path = \"{value}\"\ncv_file_prefix = \"TestCV\"\n\
             [variant]\ndefault = \"senior-devops\"\n\
             [build]\nbuilder = \"just\"\nrecipe = \"build\"\n\
             [destination]\ncv_path = \"{dest}\"\noutput_pdf = \"{out}\"\n\
             [db]\nengine = \"sqlite\"\ndb_file = \"x.db\"\n",
            dest = dest.display(),
            out = out.display(),
        ),
    )
    .unwrap();
    cfg.display().to_string()
}

/// @driving_adapter @real-io @us-01 @error @contract-shape:pure-function
/// TS-01/AC3: a `cv_template_path` that is neither a readable directory nor a
/// recognisable git URL fails the run fast, naming the offending value — at the
/// real CLI entry point, not only the unit resolver. No clone, no build, no DB.
#[test]
fn ts01_ac3_bad_template_value_fails_fast_naming_the_value() {
    let td = TempDir::new().unwrap();
    let bad = "definitely-not-a-dir-nor-a-git-url";
    let cfg = config_with_template_value(td.path(), bad);

    let output = binary()
        .args([
            "--config-ini",
            &cfg,
            "insert",
            "--job-title",
            "SRE",
            "--company-name",
            "Acme",
        ])
        .output()
        .expect("failed to run insert");

    assert!(
        !output.status.success(),
        "a bad template value must not exit 0"
    );
    let out = combined(&output);
    assert!(
        out.contains(bad),
        "the failure should name the offending value, got: {out}"
    );
}
