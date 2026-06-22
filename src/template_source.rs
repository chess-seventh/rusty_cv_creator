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
    auth: AuthMode,
}

impl GitHubRepository {
    pub fn new(url: String, cache_dir: String) -> Self {
        Self {
            url,
            cache_dir,
            auth: AuthMode::Auto,
        }
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
    /// Parse the optional `[cv] cv_template_auth` value (`auto|ssh|token`). An
    /// unknown value fails fast as a typed `TemplateSourceError::BadValue`
    /// naming the offending value — never a silent default (TS-02/AC3).
    #[allow(dead_code)]
    pub fn from_config(value: &str) -> Result<AuthMode, TemplateSourceError> {
        match value {
            "auto" => Ok(AuthMode::Auto),
            "ssh" => Ok(AuthMode::Ssh),
            "token" => Ok(AuthMode::Token),
            other => Err(TemplateSourceError::BadValue {
                value: other.to_string(),
            }),
        }
    }

    /// Human-facing name used in error hints (never the secret token value).
    #[allow(dead_code)]
    fn hint_name(self) -> &'static str {
        match self {
            AuthMode::Auto => "auto",
            AuthMode::Ssh => "ssh",
            AuthMode::Token => "token",
        }
    }

    /// Infer the transport from the URL scheme: an `git@…`/`ssh://…` remote uses
    /// the SSH agent, anything else authenticates with a token.
    fn inferred_from_url(url: &str) -> AuthMode {
        if url.starts_with("git@") || url.starts_with("ssh://") {
            return AuthMode::Ssh;
        }
        AuthMode::Token
    }
}

/// Name of the askpass helper git is pointed at for token auth. The helper reads
/// `GITHUB_TOKEN` from the environment and prints it on stdout when git asks for
/// a password, so the secret value never reaches the argv, the INI, or the cache
/// repo's `.git/config` (TS-D3 / ADR-0008). The resolver materialises this
/// executable on disk (step 04-02); only the indirection is named here.
const ASKPASS_HELPER: &str = "git-askpass-from-env";

/// Pure total fn (TS-02/AC2) over (`AuthMode` × url-scheme): the extra `git -c`
/// flags needed for `auth` against `url`.
///
/// * `Auto`  infers the transport from the url scheme (a `git@…`/`ssh://…`
///   remote inherits the SSH agent; anything else uses a token).
/// * `Ssh`   adds no flags — the agent is inherited from the ambient environment
///   (SPIKE-proven against the real private repo).
/// * `Token` routes through a `core.askpass` helper that reads `GITHUB_TOKEN`
///   from the environment; the secret VALUE never appears in the returned flags.
#[allow(dead_code)]
pub fn auth_invocation_flags(auth: AuthMode, url: &str) -> Vec<String> {
    match auth {
        AuthMode::Auto => auth_invocation_flags(AuthMode::inferred_from_url(url), url),
        AuthMode::Ssh => Vec::new(),
        AuthMode::Token => vec!["-c".to_string(), format!("core.askpass={ASKPASS_HELPER}")],
    }
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
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateSourceError::Auth { url, mode } => write!(
                formatter,
                "authentication rejected for {url} (auth mode: {}) — \
                 check your SSH key or that GITHUB_TOKEN is set and authorised",
                mode.hint_name()
            ),
            TemplateSourceError::NetworkOffline { url } => write!(
                formatter,
                "could not reach {url} — the network appears offline and \
                 no usable cached template is available"
            ),
            TemplateSourceError::BadRef { url, git_ref } => write!(
                formatter,
                "ref '{git_ref}' does not resolve in {url} — aborting \
                 (no silent fallback to the default branch)"
            ),
            TemplateSourceError::NoCache { repo_ref } => write!(
                formatter,
                "no cached template for '{repo_ref}' and the fetch failed — \
                 cannot proceed offline"
            ),
            TemplateSourceError::BadValue { value } => write!(
                formatter,
                "cv_template_path '{value}' is neither a readable local directory \
                 nor a git URL"
            ),
        }
    }
}

impl std::error::Error for TemplateSourceError {}

/// Pure (UC-1): map git's stderr to a typed failure class, so auth vs
/// network/offline vs bad-ref each get a distinct hint (TS-02/AC3, TS-03/AC3).
#[allow(dead_code)]
pub fn classify_git_stderr(stderr: &str, url: &str, git_ref: Option<&str>) -> TemplateSourceError {
    if is_auth_failure(stderr) {
        return TemplateSourceError::Auth {
            url: url.to_string(),
            mode: AuthMode::inferred_from_url(url),
        };
    }
    if is_bad_ref(stderr) {
        return TemplateSourceError::BadRef {
            url: url.to_string(),
            git_ref: git_ref.unwrap_or_default().to_string(),
        };
    }
    // Network/offline is the residual class: an unreachable host or an
    // otherwise-unrecognised transport failure means the remote is unusable.
    TemplateSourceError::NetworkOffline {
        url: url.to_string(),
    }
}

/// True when git's stderr signals an authentication rejection (SSH publickey or
/// token/credential failure) — TS-02/AC3.
fn is_auth_failure(stderr: &str) -> bool {
    stderr.contains("Permission denied (publickey)")
        || stderr.contains("Authentication failed")
        || stderr.contains("could not read Username")
        || stderr.contains("Invalid username or password")
}

/// True when git's stderr signals an unresolvable ref/pathspec — TS-03/AC3. The
/// resolver aborts on this; it must never silently fall back to the default
/// branch.
fn is_bad_ref(stderr: &str) -> bool {
    stderr.contains("couldn't find remote ref")
        || stderr.contains("did not match any file(s) known to git")
        || stderr.contains("Remote branch")
        || stderr.contains("not found in upstream")
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

    /// Select the auth transport (TS-02). Additive builder; the skeleton's
    /// default-branch `new`/`resolve` are unchanged.
    #[allow(dead_code)]
    pub fn with_auth(mut self, auth: AuthMode) -> Self {
        self.auth = auth;
        self
    }

    /// Resolve with auth-routed clone and typed failure classification
    /// (TS-02). Distinct from the skeleton's `resolve`, which stays the green
    /// default-branch happy path (TS-01/AC1).
    ///
    /// Partial: cache reuse (TS-04) and ref pinning (TS-03) wiring lands in
    /// later slices. Here it prepends the auth flags to the git invocation
    /// through `CommandRunner` and classifies a clone failure from its captured
    /// stderr. The SSH path adds no askpass flag — the agent is inherited.
    // SCAFFOLD: true
    #[allow(dead_code)]
    pub fn resolve_classified(
        &self,
        runner: &dyn CommandRunner,
    ) -> Result<String, TemplateSourceError> {
        let flags = auth_invocation_flags(self.auth, &self.url);
        let destination = self.clone_destination();

        let mut args: Vec<&str> = flags.iter().map(String::as_str).collect();
        args.push("clone");
        args.push(&self.url);
        args.push(&destination);

        let outcome = runner.run_capturing("git", &args, None).map_err(|_| {
            TemplateSourceError::NetworkOffline {
                url: self.url.clone(),
            }
        })?;

        if !outcome.success {
            return Err(classify_git_stderr(&outcome.stderr, &self.url, None));
        }

        Ok(destination)
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

    /// @us-02 @property
    /// TS-02/AC2 (strengthened): secret-absence invariant. For ANY GITHUB_TOKEN
    /// value and ANY https remote, the returned token-auth flags route through
    /// `core.askpass` yet never embed the secret — the flags are computed
    /// independently of the token, which stays in the environment and reaches
    /// git only via the askpass indirection.
    #[test]
    fn ts02_ac2_secret_value_never_embedded_in_flags() {
        use proptest::prelude::*;
        proptest!(|(token in "ghp_[A-Za-z0-9]{20,40}", name in "[a-z]{3,10}")| {
            let url = format!("https://github.com/chess-seventh/{name}.git");
            let flags = auth_invocation_flags(AuthMode::Token, &url);
            prop_assert!(
                flags.iter().any(|f| f.contains("core.askpass")),
                "token auth must route through core.askpass"
            );
            prop_assert!(
                !flags.iter().any(|f| f.contains(&token)),
                "the generated token value must never appear in the returned flags"
            );
        });
    }

    /// @us-02 @error
    /// TS-02/AC3: an auth-failure stderr is classified as a distinct `Auth` error
    /// (separate from a network/offline error).
    #[test]
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
