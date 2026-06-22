//! Template sourcing (D7) — resolve the CV template directory from either a
//! local path or a git URL.
//!
//! The OO shape locked in DISCUSS: a [`TemplateSource`] trait with two concrete
//! implementors — [`LocalDirectory`] (passthrough of an existing local path,
//! today's behaviour and the backward-compat guarantee) and
//! [`GitHubRepository`] (clones a git URL into the cache dir). The git mechanism
//! is a shell-out routed through the existing [`CommandRunner`] port (ADR-0002),
//! so no git Rust crate is needed and the clone is testable with `FakeRunner`.
//!
//! Walking-skeleton scope (slice 01 / TS-01): only the public-clone happy path
//! and the local-dir passthrough. Private auth (TS-02), ref pinning (TS-03) and
//! cache reuse / offline fallback (TS-04) are later slices and intentionally
//! absent here.

use crate::command_runner::CommandRunner;
use crate::helpers::ensure_tools_available;
use log::info;
use std::fs;
use std::path::Path;

/// A source of the CV template that resolves to a local directory which the
/// existing `copy_dir` flow consumes unchanged (D5).
pub trait TemplateSource {
    /// Resolve this source to a local template directory path.
    fn resolve(&self, runner: &dyn CommandRunner) -> Result<String, Box<dyn std::error::Error>>;
}

/// Passthrough source: the configured value already points at a local template
/// directory. Resolving returns the path unchanged — this is the pre-feature
/// behaviour and the backward-compatibility guarantee (TS-01/AC2).
pub struct LocalDirectory {
    path: String,
}

impl LocalDirectory {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl TemplateSource for LocalDirectory {
    fn resolve(&self, _runner: &dyn CommandRunner) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.path.clone())
    }
}

/// Git-backed source: clones the repository's default branch into the cache
/// directory and resolves to that local clone (TS-01/AC1).
pub struct GitHubRepository {
    url: String,
    cache_dir: String,
}

impl GitHubRepository {
    pub fn new(url: String, cache_dir: String) -> Self {
        Self { url, cache_dir }
    }

    /// Deterministic per-repository sub-directory of the cache dir. Cache keying
    /// by ref and reuse-on-failure are later slices (TS-03 / TS-04); the skeleton
    /// always clones fresh into this location.
    fn clone_destination(&self) -> String {
        let sanitized: String = self
            .url
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        format!("{}/{sanitized}", self.cache_dir)
    }
}

impl TemplateSource for GitHubRepository {
    fn resolve(&self, runner: &dyn CommandRunner) -> Result<String, Box<dyn std::error::Error>> {
        // ADR-0004 pre-usage check: fail fast with a devenv hint if git is absent.
        ensure_tools_available(&["git"])?;

        let destination = self.clone_destination();

        // Skeleton: no cache reuse yet (TS-04) — always clone fresh.
        if Path::new(&destination).exists() {
            fs::remove_dir_all(&destination)?;
        }
        if let Some(parent) = Path::new(&destination).parent() {
            fs::create_dir_all(parent)?;
        }

        info!("✅ Cloning template from {} (default branch)", self.url);
        if !runner.status("git", &["clone", &self.url, &destination], None)? {
            return Err(format!("git clone failed for {}", self.url).into());
        }

        info!("✅ Template cloned to {destination}");
        Ok(destination)
    }
}

/// Auto-detect (D1) which [`TemplateSource`] to use from the configured
/// `cv_template_path` value: an existing readable directory → [`LocalDirectory`];
/// a git URL → [`GitHubRepository`]. Anything else is rejected with a message
/// naming the offending value and the accepted forms (TS-01/AC3).
pub fn detect_template_source(
    value: &str,
    cache_dir: &str,
) -> Result<Box<dyn TemplateSource>, Box<dyn std::error::Error>> {
    if Path::new(value).is_dir() {
        return Ok(Box::new(LocalDirectory::new(value.to_string())));
    }
    if is_git_url(value) {
        return Ok(Box::new(GitHubRepository::new(
            value.to_string(),
            cache_dir.to_string(),
        )));
    }
    Err(format!(
        "cv_template_path '{value}' is neither a readable local directory nor a git URL \
         (expected an existing directory, a 'git@…' SSH URL, or an 'https://….git' URL)"
    )
    .into())
}

/// True when `value` looks like a clonable git URL. Besides the two GitHub forms
/// named in D1 (`git@…`, `https://….git`), a `file://…` local remote is also
/// recognised — it is a genuine clonable git URL used for local/offline remotes.
fn is_git_url(value: &str) -> bool {
    value.starts_with("git@")
        || value.starts_with("file://")
        || (value.starts_with("https://") && value.ends_with(".git"))
}

// ─────────────────────────────────────────────────────────────────────────────
// DISTILL RED scaffolds — slices TS-02 (auth), TS-03 (ref pinning),
// TS-04 (cache/offline) and TS-D1 (typed errors). ADDITIVE ONLY: the green
// skeleton above (`LocalDirectory`, `GitHubRepository::new`/`resolve`,
// `detect_template_source`, `is_git_url`) is untouched. Every body `panic!`s so
// the pending specs in `mod distill_specs` classify RED (not BROKEN). DELIVER
// replaces these bodies. Detect markers: `// SCAFFOLD: true`.
// ─────────────────────────────────────────────────────────────────────────────

/// Auth transport (TS-D3, TS-02). Inferred from the URL scheme by default;
/// `token` reads `GITHUB_TOKEN` from the environment and feeds git via
/// `core.askpass` — never the INI, the argv, or the cache `.git/config`.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Auto,
    Ssh,
    Token,
}

impl AuthMode {
    /// Parse the optional `[cv] cv_template_auth` value (`auto|ssh|token`).
    // SCAFFOLD: true
    #[allow(dead_code)]
    pub fn from_config(_value: &str) -> Result<AuthMode, TemplateSourceError> {
        panic!("not yet implemented — RED scaffold")
    }
}

/// Pure (TS-02/AC2): the extra `git -c` flags needed for `auth` against `url`.
/// For `Token` this is a `core.askpass` helper invocation; the secret value
/// itself never appears in the returned flags (asserted by the specs).
// SCAFFOLD: true
#[allow(dead_code)]
pub fn auth_invocation_flags(_auth: AuthMode, _url: &str) -> Vec<String> {
    panic!("not yet implemented — RED scaffold")
}

/// Typed failure classes (TS-D1). Distinct variants so the resolver emits a
/// distinct actionable hint per failure by `match`, never by string-compare.
#[allow(dead_code)]
#[derive(Debug)]
pub enum TemplateSourceError {
    /// Auth rejected (e.g. SSH publickey / bad token) — TS-02/AC3.
    Auth { url: String, mode: AuthMode },
    /// Remote unreachable / offline with no usable cache — TS-04.
    NetworkOffline { url: String },
    /// The pinned ref does not resolve in the repo — TS-03/AC3.
    BadRef { url: String, git_ref: String },
    /// No cache for `repo@ref` and the fetch failed — TS-04/AC2.
    NoCache { repo_ref: String },
    /// Value is neither a readable directory nor a git URL — TS-01/AC3.
    BadValue { value: String },
}

impl std::fmt::Display for TemplateSourceError {
    // SCAFFOLD: true
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("not yet implemented — RED scaffold")
    }
}

impl std::error::Error for TemplateSourceError {}

/// Pure (UC-1): map git's stderr to a typed failure class, so auth vs
/// network/offline vs bad-ref each get a distinct hint (TS-02/AC3, TS-03/AC3).
// SCAFFOLD: true
#[allow(dead_code)]
pub fn classify_git_stderr(
    _stderr: &str,
    _url: &str,
    _git_ref: Option<&str>,
) -> TemplateSourceError {
    panic!("not yet implemented — RED scaffold")
}

/// Pure reuse-vs-fetch-vs-abort decision (TS-D2, TS-04). A total function over
/// (cache-entry-present, remote-reachable) — never panics once implemented.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheAction {
    /// No entry yet — full clone.
    Clone,
    /// Entry present and remote reachable — fetch + checkout (the perf lever).
    FetchCheckout,
    /// Remote unreachable but a usable entry exists — reuse it offline (warn).
    ReuseStale,
    /// Remote unreachable and no entry — abort fast, no partial CV.
    Abort,
}

/// Owns cache-key derivation and the pure cache decision (TS-D2). Only the
/// executor writes, and its write universe is bounded to the cache dir.
#[allow(dead_code)]
pub struct TemplateCache {
    cache_dir: String,
}

impl TemplateCache {
    #[allow(dead_code)]
    pub fn new(cache_dir: String) -> Self {
        Self { cache_dir }
    }

    /// Deterministic sanitised cache key for `repo@ref`. Pure — same input maps
    /// to the same key (PBT in `mod distill_specs`).
    // SCAFFOLD: true
    #[allow(dead_code)]
    pub fn cache_key(&self, _url: &str, _git_ref: Option<&str>) -> String {
        panic!("not yet implemented — RED scaffold")
    }

    /// Pure decision over (entry-exists, remote-reachable) — the TS-04 matrix.
    // SCAFFOLD: true
    #[allow(dead_code)]
    pub fn decide(&self, _entry_exists: bool, _remote_reachable: bool) -> CacheAction {
        panic!("not yet implemented — RED scaffold")
    }
}

impl GitHubRepository {
    /// Pin the template to a branch/tag/SHA (TS-03). Additive builder; the
    /// skeleton's default-branch `new`/`resolve` are unchanged.
    // SCAFFOLD: true
    #[allow(dead_code)]
    pub fn with_ref(self, _git_ref: &str) -> Self {
        panic!("not yet implemented — RED scaffold")
    }

    /// Select the auth transport (TS-02). Additive.
    // SCAFFOLD: true
    #[allow(dead_code)]
    pub fn with_auth(self, _auth: AuthMode) -> Self {
        panic!("not yet implemented — RED scaffold")
    }

    /// Resolve with cache reuse, ref pinning, auth, and typed failure
    /// classification (TS-02/03/04). Distinct from the skeleton's `resolve`,
    /// which stays the green default-branch happy path (TS-01/AC1).
    // SCAFFOLD: true
    #[allow(dead_code)]
    pub fn resolve_classified(
        &self,
        _runner: &dyn CommandRunner,
    ) -> Result<String, TemplateSourceError> {
        panic!("not yet implemented — RED scaffold")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_runner::testing::FakeRunner;
    use tempfile::TempDir;

    #[test]
    fn test_local_directory_resolves_to_passthrough_path() {
        let source = LocalDirectory::new("/some/local/template".to_string());
        let runner = FakeRunner::ok();
        assert_eq!(source.resolve(&runner).unwrap(), "/some/local/template");
    }

    #[test]
    fn test_local_directory_does_not_invoke_runner() {
        let source = LocalDirectory::new("/some/local/template".to_string());
        let runner = FakeRunner::ok();
        source.resolve(&runner).unwrap();
        assert!(runner.calls.borrow().is_empty());
    }

    // Serialized: `resolve` reads the global PATH (ADR-0004 git check), which the
    // PATH-mutating test in `helpers` temporarily clobbers.
    #[test]
    #[serial_test::serial]
    fn test_github_repository_clones_via_runner() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let source =
            GitHubRepository::new("https://github.com/chess-seventh/cv.git".to_string(), cache);
        let runner = FakeRunner::ok();

        let resolved = source.resolve(&runner).unwrap();

        let recorded = &runner.calls.borrow()[0];
        assert!(recorded.starts_with("git clone https://github.com/chess-seventh/cv.git "));
        assert!(recorded.ends_with(&resolved));
    }

    #[test]
    #[serial_test::serial]
    fn test_github_repository_errors_when_clone_fails() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let source =
            GitHubRepository::new("https://github.com/chess-seventh/cv.git".to_string(), cache);
        let runner = FakeRunner::failing();
        assert!(source.resolve(&runner).is_err());
    }

    #[test]
    fn test_detect_existing_dir_is_local() {
        let td = TempDir::new().unwrap();
        let dir = td.path().to_str().unwrap();
        let runner = FakeRunner::ok();
        // A local dir resolves to itself and does not shell out.
        let resolved = detect_template_source(dir, "/unused/cache")
            .unwrap()
            .resolve(&runner)
            .unwrap();
        assert_eq!(resolved, dir);
        assert!(runner.calls.borrow().is_empty());
    }

    #[test]
    #[serial_test::serial]
    fn test_detect_ssh_url_is_github() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let runner = FakeRunner::ok();
        detect_template_source("git@github.com:chess-seventh/cv.git", &cache)
            .unwrap()
            .resolve(&runner)
            .unwrap();
        assert!(
            runner.calls.borrow()[0].starts_with("git clone git@github.com:chess-seventh/cv.git")
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_detect_https_git_url_is_github() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let runner = FakeRunner::ok();
        detect_template_source("https://github.com/chess-seventh/cv.git", &cache)
            .unwrap()
            .resolve(&runner)
            .unwrap();
        assert!(
            runner.calls.borrow()[0]
                .starts_with("git clone https://github.com/chess-seventh/cv.git")
        );
    }

    #[test]
    fn test_detect_unrecognised_value_errors_naming_value() {
        match detect_template_source("not-a-dir-nor-url", "/unused/cache") {
            Ok(_) => panic!("unrecognised value should not resolve to a source"),
            Err(err) => assert!(err.to_string().contains("not-a-dir-nor-url")),
        }
    }
}

#[cfg(test)]
mod distill_specs {
    //! DISTILL specifications for slices TS-02/03/04, mapped from the `.feature`
    //! SSOT under `tests/acceptance/template-source/`. These live in-crate (not in
    //! an external `tests/` file) because `TemplateSource`, the scaffolds, and
    //! `command_runner::testing::FakeRunner` are binary-private — they are NOT on
    //! the `lib.rs` facade (only `database`/`models`/`schema`/`tui` are), so an
    //! external integration crate cannot reach them. This matches the existing
    //! green precedent (`mod tests` above). Pending specs are `#[ignore]`d
    //! one-at-a-time for DELIVER; un-ignored they fail by `panic!` (RED), not by a
    //! compile/import error.
    use super::*;
    use crate::command_runner::testing::FakeRunner;
    use tempfile::TempDir;

    // ── TS-02 — private repo / auth ──────────────────────────────────────────

    /// @us-02 @in-memory
    /// TS-02/AC1: an SSH source clones over its `git@…` URL using the agent — no
    /// askpass / token machinery on the SSH path.
    #[test]
    #[serial_test::serial]
    #[ignore = "pending DELIVER — TS-02"]
    fn ts02_ac1_ssh_source_clones_via_git_at_url() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let runner = FakeRunner::ok();
        let source = GitHubRepository::new("git@github.com:chess-seventh/cv.git".into(), cache)
            .with_auth(AuthMode::Ssh);

        source.resolve_classified(&runner).unwrap();

        let recorded = &runner.calls.borrow()[0];
        assert!(recorded.starts_with("git clone git@github.com:chess-seventh/cv.git"));
        assert!(
            !recorded.contains("askpass"),
            "the SSH path must not use askpass"
        );
    }

    /// @us-02 @in-memory @error
    /// TS-02/AC2: the token is taken from the environment and fed via
    /// `core.askpass`; it never appears on the git command line.
    #[test]
    #[ignore = "pending DELIVER — TS-02"]
    fn ts02_ac2_token_uses_askpass_and_never_on_argv() {
        let flags =
            auth_invocation_flags(AuthMode::Token, "https://github.com/chess-seventh/cv.git");
        assert!(
            flags.iter().any(|f| f.contains("core.askpass")),
            "token auth must route through core.askpass"
        );
        assert!(
            !flags
                .iter()
                .any(|f| f.contains("x-access-token") || f.contains("ghp_")),
            "the token value must never appear in the git argv"
        );
    }

    /// @us-02 @error
    /// TS-02/AC3: an auth-failure stderr is classified as a distinct `Auth` error
    /// (separate from a network/offline error).
    #[test]
    #[ignore = "pending DELIVER — TS-02"]
    fn ts02_ac3_auth_failure_stderr_classified_as_auth() {
        let err = classify_git_stderr(
            "git@github.com: Permission denied (publickey).",
            "git@github.com:chess-seventh/cv.git",
            None,
        );
        assert!(matches!(err, TemplateSourceError::Auth { .. }));
    }

    // ── TS-03 — ref pinning ──────────────────────────────────────────────────

    /// @us-03 @in-memory
    /// TS-03/AC1: a pinned ref is explicitly checked out (its resolved SHA is then
    /// logged — detached-HEAD safe).
    #[test]
    #[serial_test::serial]
    #[ignore = "pending DELIVER — TS-03"]
    fn ts03_ac1_pinned_ref_is_checked_out() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let runner = FakeRunner::ok();
        let source = GitHubRepository::new("https://github.com/chess-seventh/cv.git".into(), cache)
            .with_ref("v2.1");

        source.resolve_classified(&runner).unwrap();

        assert!(
            runner
                .calls
                .borrow()
                .iter()
                .any(|c| c.contains("checkout v2.1")),
            "the pinned ref must be checked out explicitly"
        );
    }

    /// @us-03 @error
    /// TS-03/AC3: an unknown ref is classified as `BadRef` — the resolver aborts,
    /// it does NOT silently fall back to the default branch.
    #[test]
    #[ignore = "pending DELIVER — TS-03"]
    fn ts03_ac3_bad_ref_classified_no_silent_fallback() {
        let err = classify_git_stderr(
            "fatal: couldn't find remote ref does-not-exist",
            "https://github.com/chess-seventh/cv.git",
            Some("does-not-exist"),
        );
        assert!(matches!(err, TemplateSourceError::BadRef { .. }));
    }

    // ── TS-04 — cache / offline (the CacheAction matrix) ─────────────────────

    /// @us-04 @property
    /// TS-04/AC1+AC2+AC3: the reuse-vs-fetch-vs-abort decision is total over
    /// (cache-entry-present, remote-reachable).
    #[test]
    #[ignore = "pending DELIVER — TS-04"]
    fn ts04_cache_action_matrix() {
        let cache = TemplateCache::new("/unused/cache".into());
        assert_eq!(cache.decide(false, true), CacheAction::Clone); // AC3: fresh clone
        assert_eq!(cache.decide(true, true), CacheAction::FetchCheckout); // AC3: update
        assert_eq!(cache.decide(true, false), CacheAction::ReuseStale); // AC1: offline reuse
        assert_eq!(cache.decide(false, false), CacheAction::Abort); // AC2: hard abort
    }

    /// @us-04 @property @edge
    /// A repository and ref map to one deterministic cache entry (same input →
    /// same key).
    #[test]
    #[ignore = "pending DELIVER — TS-04"]
    fn ts04_cache_key_is_deterministic() {
        use proptest::prelude::*;
        let cache = TemplateCache::new("/unused/cache".into());
        proptest!(|(url in "[a-z]{3,12}",
                    git_ref in proptest::option::of("[a-z0-9]{1,8}"))| {
            let r = git_ref.as_deref();
            prop_assert_eq!(cache.cache_key(&url, r), cache.cache_key(&url, r));
        });
    }

    // ── TS-01 — URL detection property (GREEN: `is_git_url` is implemented) ───

    /// @us-01 @property @edge
    /// Every recognised URL form (the two D1 GitHub forms plus the `file://`
    /// superset) classifies as a git source; a bare token or a bare local path
    /// does not.
    #[test]
    fn ts01_is_git_url_classifies_known_forms() {
        use proptest::prelude::*;
        proptest!(|(name in "[a-z]{3,10}")| {
            let ssh = format!("git@github.com:chess-seventh/{}.git", name);
            let https = format!("https://github.com/chess-seventh/{}.git", name);
            let file_remote = format!("file:///tmp/{}", name);
            let local_path = format!("/home/{}/templates/cv", name);
            prop_assert!(is_git_url(&ssh));
            prop_assert!(is_git_url(&https));
            prop_assert!(is_git_url(&file_remote));
            prop_assert!(!is_git_url(&name)); // bare token
            prop_assert!(!is_git_url(&local_path)); // local path
        });
    }
}
