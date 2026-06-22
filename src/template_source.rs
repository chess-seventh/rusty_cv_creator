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

use crate::command_runner::{CommandOutcome, CommandRunner};
use crate::helpers::ensure_tools_available;
use log::{info, warn};
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
    git_ref: Option<String>,
}

impl GitHubRepository {
    pub fn new(url: String, cache_dir: String) -> Self {
        Self {
            url,
            cache_dir,
            auth: AuthMode::Auto,
            git_ref: None,
        }
    }

    /// Deterministic per-`repo@ref` sub-directory of the cache dir, shared with
    /// [`TemplateCache::cache_key`] so a cache probe and a clone address the same
    /// entry. An unset ref folds to the shared default-branch slot.
    fn clone_destination(&self) -> String {
        TemplateCache::new(self.cache_dir.clone()).entry_path(&self.url, self.git_ref.as_deref())
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

/// Production resolver entry (step 04-02): resolve the configured
/// `cv_template_path` to a local template directory, threading the optional
/// `cv_template_ref` (TS-03), `cv_template_auth` (TS-02) and the cache dir
/// (TS-04) for git sources. A local directory is an unchanged passthrough
/// (TS-01/AC2); a git URL resolves through the cache executor (probe → decide →
/// clone / fetch / reuse-stale / abort). Anything else is rejected by
/// [`detect_template_source`], naming the offending value (TS-01/AC3).
pub fn resolve_template_for_config(
    value: &str,
    cache_dir: &str,
    git_ref: Option<&str>,
    auth: AuthMode,
    runner: &dyn CommandRunner,
) -> Result<String, Box<dyn std::error::Error>> {
    // Local-dir precedence matches `detect_template_source`: only a non-directory
    // value that parses as a git URL takes the enriched (ref/auth/cache) path.
    if !Path::new(value).is_dir() && is_git_url(value) {
        // ADR-0004 pre-usage check: fail fast with a devenv hint if git is absent.
        ensure_tools_available(&["git"])?;
        let mut repository =
            GitHubRepository::new(value.to_string(), cache_dir.to_string()).with_auth(auth);
        if let Some(reference) = git_ref {
            repository = repository.with_ref(reference);
        }
        let cache = TemplateCache::new(cache_dir.to_string());
        return Ok(repository.resolve_cached(runner, &cache)?);
    }
    // Local-dir passthrough and the BadValue error stay with the classifier.
    detect_template_source(value, cache_dir)?.resolve(runner)
}

// ─────────────────────────────────────────────────────────────────────────────
// Auth (TS-02), ref pinning (TS-03), cache / offline (TS-04) and typed errors
// (TS-D1) — wired into the production resolver `resolve_template_for_config`
// above (consumed by `file_handlers::prepare_path_for_new_cv`). The skeleton
// happy path (`LocalDirectory`, `GitHubRepository::new`/`resolve`,
// `detect_template_source`, `is_git_url`) is unchanged.
// ─────────────────────────────────────────────────────────────────────────────

/// Auth transport (TS-D3, TS-02). Inferred from the URL scheme by default;
/// `token` reads `GITHUB_TOKEN` from the environment and feeds git via
/// `core.askpass` — never the INI, the argv, or the cache `.git/config`.
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
pub fn auth_invocation_flags(auth: AuthMode, url: &str) -> Vec<String> {
    match auth {
        AuthMode::Auto => auth_invocation_flags(AuthMode::inferred_from_url(url), url),
        AuthMode::Ssh => Vec::new(),
        AuthMode::Token => vec!["-c".to_string(), format!("core.askpass={ASKPASS_HELPER}")],
    }
}

/// Typed failure classes (TS-D1). Distinct variants so the resolver emits a
/// distinct actionable hint per failure by `match`, never by string-compare.
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
    /// A local filesystem operation on the cache entry failed (permissions,
    /// full disk, …) — distinct from a network failure so the hint is accurate.
    Io { url: String, detail: String },
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
            TemplateSourceError::Io { url, detail } => write!(
                formatter,
                "could not prepare the local cache for {url}: {detail} — \
                 check directory permissions and available disk space"
            ),
        }
    }
}

impl std::error::Error for TemplateSourceError {}

/// Pure (UC-1): map git's stderr to a typed failure class, so auth vs
/// network/offline vs bad-ref each get a distinct hint (TS-02/AC3, TS-03/AC3).
pub fn classify_git_stderr(stderr: &str, url: &str, git_ref: Option<&str>) -> TemplateSourceError {
    if is_auth_failure(stderr) {
        return TemplateSourceError::Auth {
            url: url.to_string(),
            mode: AuthMode::inferred_from_url(url),
        };
    }
    // A bad-ref stderr only classifies as `BadRef` when a ref was actually
    // pinned. A no-ref clone failing with a ref-ish message (e.g. a missing repo
    // or access problem) is a bad-URL / network class, not a ref problem — so it
    // falls through to `NetworkOffline` rather than emitting a confusing "ref ''"
    // message (LOW 3).
    if let Some(reference) = git_ref {
        if is_bad_ref(stderr) {
            return TemplateSourceError::BadRef {
                url: url.to_string(),
                git_ref: reference.to_string(),
            };
        }
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
pub struct TemplateCache {
    cache_dir: String,
}

impl TemplateCache {
    pub fn new(cache_dir: String) -> Self {
        Self { cache_dir }
    }

    /// Deterministic sanitised cache key for `repo@ref`. Pure — same input maps
    /// to the same key (PBT in `mod distill_specs`). The url and ref are each
    /// sanitised (non-alphanumerics fold to `_`) and joined with `@`; an unset
    /// ref folds to one shared default-branch slot, so every default-branch
    /// resolve of a repository addresses the same entry.
    pub fn cache_key(&self, url: &str, git_ref: Option<&str>) -> String {
        let repository = sanitize_key_component(url);
        let reference = git_ref
            .map(sanitize_key_component)
            .unwrap_or_else(|| DEFAULT_REF_SLOT.to_string());
        format!("{repository}@{reference}")
    }

    /// Absolute cache entry directory for `repo@ref`: the cache dir joined with
    /// the deterministic [`cache_key`](Self::cache_key). A cache probe and a clone
    /// address the same entry through this single derivation.
    pub fn entry_path(&self, url: &str, git_ref: Option<&str>) -> String {
        format!("{}/{}", self.cache_dir, self.cache_key(url, git_ref))
    }

    /// Pure decision over (entry-exists, remote-reachable) — the TS-04 matrix.
    /// Total: every point of the 2×2 boolean universe maps to exactly one
    /// [`CacheAction`], so the decision is DATA (a value to act on later), never
    /// an in-line side effect.
    pub fn decide(&self, entry_exists: bool, remote_reachable: bool) -> CacheAction {
        match (entry_exists, remote_reachable) {
            (false, true) => CacheAction::Clone,
            (true, true) => CacheAction::FetchCheckout,
            (true, false) => CacheAction::ReuseStale,
            (false, false) => CacheAction::Abort,
        }
    }
}

/// Cache slot addressed when no explicit ref is pinned: every default-branch
/// resolve of a repository folds to this one deterministic entry (TS-04 key).
const DEFAULT_REF_SLOT: &str = "default";

/// Fold an arbitrary url/ref fragment to a filesystem-safe, deterministic slug
/// by replacing every non-alphanumeric character with `_`. Same input always
/// yields the same slug — this is what makes [`TemplateCache::cache_key`] pure.
fn sanitize_key_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect()
}

impl GitHubRepository {
    /// Pin the template to a branch/tag/SHA (TS-03). Additive builder; the
    /// skeleton's default-branch `new`/`resolve` are unchanged.
    pub fn with_ref(mut self, git_ref: &str) -> Self {
        self.git_ref = Some(git_ref.to_string());
        self
    }

    /// Select the auth transport (TS-02). Additive builder; the skeleton's
    /// default-branch `new`/`resolve` are unchanged.
    pub fn with_auth(mut self, auth: AuthMode) -> Self {
        self.auth = auth;
        self
    }

    /// Cache-aware resolution (TS-04, step 04-02 executor): probe remote
    /// reachability, decide via [`TemplateCache::decide`], then perform the chosen
    /// [`CacheAction`] against the cache entry:
    ///
    /// * `Clone`         — no entry yet: a fresh auth-routed clone + ref checkout.
    /// * `FetchCheckout` — entry present and remote reachable: fetch then
    ///   re-checkout the pinned ref (the perf lever).
    /// * `ReuseStale`    — remote unreachable but a usable entry exists: reuse it
    ///   offline with a warning.
    /// * `Abort`         — remote unreachable and no entry: fail fast as `NoCache`
    ///   (no partial CV).
    pub fn resolve_cached(
        &self,
        runner: &dyn CommandRunner,
        cache: &TemplateCache,
    ) -> Result<String, TemplateSourceError> {
        let entry = self.clone_destination();
        let entry_exists = Path::new(&entry).is_dir();
        let remote_reachable = self.remote_reachable(runner);

        match cache.decide(entry_exists, remote_reachable) {
            CacheAction::Clone => self.resolve_classified(runner),
            CacheAction::FetchCheckout => self.fetch_existing_entry(runner, &entry),
            CacheAction::ReuseStale => {
                warn!("⚠️  Offline — reusing the cached template at {entry} (it may be stale)");
                Ok(entry)
            }
            CacheAction::Abort => Err(TemplateSourceError::NoCache {
                repo_ref: cache.cache_key(&self.url, self.git_ref.as_deref()),
            }),
        }
    }

    /// Probe whether the remote is reachable under the configured auth by listing
    /// its refs. A captured io failure or a non-zero exit reads as unreachable, so
    /// the cache decision falls back to reuse-or-abort (TS-04).
    fn remote_reachable(&self, runner: &dyn CommandRunner) -> bool {
        let flags = auth_invocation_flags(self.auth, &self.url);
        let mut args: Vec<&str> = flags.iter().map(String::as_str).collect();
        args.push("ls-remote");
        args.push(&self.url);
        runner
            .run_capturing("git", &args, None)
            .map(|outcome| outcome.success)
            .unwrap_or(false)
    }

    /// Update an existing cache entry (remote reachable): fetch, then re-checkout
    /// the pinned ref. The fetch is auth-routed the same way as the clone/probe —
    /// an HTTPS+token cache refresh would otherwise fail auth (the SSH agent path
    /// adds no flags). A fetch or checkout failure is classified from its captured
    /// stderr — never a silent fallback (TS-03/AC3).
    fn fetch_existing_entry(
        &self,
        runner: &dyn CommandRunner,
        entry: &str,
    ) -> Result<String, TemplateSourceError> {
        let flags = auth_invocation_flags(self.auth, &self.url);
        let mut args: Vec<&str> = flags.iter().map(String::as_str).collect();
        args.extend_from_slice(&["fetch", "--prune", "--tags"]);
        let fetch = runner
            .run_capturing("git", &args, Some(entry))
            .map_err(|_| TemplateSourceError::NetworkOffline {
                url: self.url.clone(),
            })?;
        if !fetch.success {
            return Err(classify_git_stderr(
                &fetch.stderr,
                &self.url,
                self.git_ref.as_deref(),
            ));
        }
        if let Some(git_ref) = &self.git_ref {
            self.checkout_pinned_ref(runner, entry, git_ref)?;
        }
        Ok(entry.to_string())
    }

    /// Resolve with an auth-routed clone and typed failure classification (TS-02 /
    /// TS-03). Prepares the destination (fresh parent dir, no stale clone), clones
    /// with the auth flags through `CommandRunner`, classifies a failure from
    /// captured stderr, then — when a ref is pinned — checks it out and logs the
    /// resolved SHA, aborting via `BadRef` with NO silent default-branch fallback.
    /// The SSH path adds no askpass flag — the agent is inherited.
    pub fn resolve_classified(
        &self,
        runner: &dyn CommandRunner,
    ) -> Result<String, TemplateSourceError> {
        let destination = self.clone_destination();
        self.prepare_destination(&destination)?;

        let flags = auth_invocation_flags(self.auth, &self.url);
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
            return Err(classify_git_stderr(
                &outcome.stderr,
                &self.url,
                self.git_ref.as_deref(),
            ));
        }

        if let Some(git_ref) = &self.git_ref {
            self.checkout_pinned_ref(runner, &destination, git_ref)?;
        }

        Ok(destination)
    }

    /// Ensure the cache entry's parent exists and no stale clone occupies the
    /// destination, so a fresh `git clone` into it always succeeds. A filesystem
    /// failure (permissions, full disk, …) maps to a distinct `Io` error carrying
    /// the real io detail — never the misleading `NetworkOffline` hint (LOW 1).
    fn prepare_destination(&self, destination: &str) -> Result<(), TemplateSourceError> {
        let io = |error: std::io::Error| TemplateSourceError::Io {
            url: self.url.clone(),
            detail: error.to_string(),
        };
        if Path::new(destination).exists() {
            fs::remove_dir_all(destination).map_err(io)?;
        }
        if let Some(parent) = Path::new(destination).parent() {
            fs::create_dir_all(parent).map_err(io)?;
        }
        Ok(())
    }

    /// Explicitly check out `git_ref` in the freshly cloned `destination`, then
    /// log the resolved SHA via `git rev-parse HEAD` (detached-HEAD safe — a SHA
    /// checkout leaves a detached HEAD, so branch state is never relied upon). A
    /// failing checkout or rev-parse is classified from its captured stderr into
    /// `BadRef` and aborts — never a silent fallback to the default branch
    /// (TS-03/AC3).
    fn checkout_pinned_ref(
        &self,
        runner: &dyn CommandRunner,
        destination: &str,
        git_ref: &str,
    ) -> Result<(), TemplateSourceError> {
        self.run_in_clone_checked(runner, &["checkout", git_ref], destination, git_ref)?;
        let resolved =
            self.run_in_clone_checked(runner, &["rev-parse", "HEAD"], destination, git_ref)?;
        info!(
            "✅ Pinned template to ref '{git_ref}' (resolved SHA: {})",
            resolved.stdout.trim()
        );
        Ok(())
    }

    /// Run a git subcommand inside the cloned `destination` and require success,
    /// classifying a non-zero exit from its captured stderr (the pinned-ref
    /// context means a failure aborts as `BadRef` — never a silent default-branch
    /// fallback, TS-03/AC3). A captured io failure is mapped by [`run_in_clone`].
    fn run_in_clone_checked(
        &self,
        runner: &dyn CommandRunner,
        args: &[&str],
        destination: &str,
        git_ref: &str,
    ) -> Result<CommandOutcome, TemplateSourceError> {
        let outcome = self.run_in_clone(runner, args, destination, git_ref)?;
        if !outcome.success {
            return Err(classify_git_stderr(
                &outcome.stderr,
                &self.url,
                Some(git_ref),
            ));
        }
        Ok(outcome)
    }

    /// Run a git subcommand inside the cloned `destination`, mapping a captured
    /// io failure to `BadRef` so a pinned-ref resolve never silently degrades.
    fn run_in_clone(
        &self,
        runner: &dyn CommandRunner,
        args: &[&str],
        destination: &str,
        git_ref: &str,
    ) -> Result<CommandOutcome, TemplateSourceError> {
        runner
            .run_capturing("git", args, Some(destination))
            .map_err(|_| TemplateSourceError::BadRef {
                url: self.url.clone(),
                git_ref: git_ref.to_string(),
            })
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

    // ── Direct coverage of the PRODUCTION resolver `resolve_classified` ──────
    // (the skeleton `resolve` above is auth/cache/ref-UNAWARE). These assert the
    // auth-flag assembly and the stderr→TemplateSourceError classification that
    // the production git path actually exercises.

    /// Token auth assembles `-c core.askpass=…` into the recorded clone argv.
    #[test]
    #[serial_test::serial]
    fn test_resolve_classified_token_auth_wires_askpass_into_clone() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let source = GitHubRepository::new("https://github.com/chess-seventh/cv.git".into(), cache)
            .with_auth(AuthMode::Token);
        let runner = FakeRunner::ok();

        source.resolve_classified(&runner).unwrap();

        let recorded = &runner.calls.borrow()[0];
        assert!(
            recorded.contains("-c core.askpass=") && recorded.contains("clone"),
            "token clone must carry the askpass flag: {recorded}"
        );
    }

    /// An auth-failure stderr from the clone classifies as `Auth`.
    #[test]
    #[serial_test::serial]
    fn test_resolve_classified_auth_failure_stderr_maps_to_auth_error() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let source = GitHubRepository::new("https://github.com/chess-seventh/cv.git".into(), cache)
            .with_auth(AuthMode::Token);
        let runner = FakeRunner::failing_with_stderr(
            "remote: Invalid username or password.\nfatal: Authentication failed",
        );

        let err = source.resolve_classified(&runner).unwrap_err();
        assert!(
            matches!(err, TemplateSourceError::Auth { .. }),
            "auth-failure stderr must classify as Auth, got: {err:?}"
        );
    }

    /// A bad-ref stderr from the clone, WITH a pinned ref, classifies as `BadRef`
    /// (no silent fallback). Would regress to `NetworkOffline` if the clone-failure
    /// path passed `None` instead of the pinned ref.
    #[test]
    #[serial_test::serial]
    fn test_resolve_classified_bad_ref_stderr_with_pinned_ref_maps_to_bad_ref() {
        let td = TempDir::new().unwrap();
        let cache = td.path().to_str().unwrap().to_string();
        let source = GitHubRepository::new("https://github.com/chess-seventh/cv.git".into(), cache)
            .with_ref("does-not-exist");
        let runner =
            FakeRunner::failing_with_stderr("fatal: couldn't find remote ref does-not-exist");

        let err = source.resolve_classified(&runner).unwrap_err();
        assert!(
            matches!(err, TemplateSourceError::BadRef { .. }),
            "bad-ref stderr with a pinned ref must classify as BadRef, got: {err:?}"
        );
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
    /// TS-02/AC2 (falsifiable): for ANY https remote where token auth applies, the
    /// returned flags are non-empty AND contain `core.askpass=<fixed helper>`,
    /// where the helper name is the fixed [`ASKPASS_HELPER`] constant, NOT derived
    /// from the url input. This turns RED if `auth_invocation_flags` returned an
    /// empty vec (no askpass wiring) or embedded an input-derived value in place of
    /// the constant indirection.
    #[test]
    fn ts02_ac2_token_flags_route_through_fixed_askpass_helper() {
        use proptest::prelude::*;
        proptest!(|(host in "[a-z]{3,10}", tld in "[a-z]{2,4}", name in "[a-z]{3,10}")| {
            let url = format!("https://{host}.{tld}/team/{name}.git");
            let flags = auth_invocation_flags(AuthMode::Token, &url);
            prop_assert!(
                !flags.is_empty(),
                "token auth must wire at least the askpass flag — never an empty vec"
            );
            let expected = format!("core.askpass={ASKPASS_HELPER}");
            prop_assert!(
                flags.iter().any(|f| f == &expected),
                "token auth must point git at the fixed askpass helper constant"
            );
            prop_assert!(
                !flags.iter().any(|f| f.contains(host.as_str()) || f.contains(name.as_str())),
                "the helper name must be input-independent — no url fragment in the flags"
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
    fn ts04_cache_action_matrix() {
        let cache = TemplateCache::new("/unused/cache".into());
        assert_eq!(cache.decide(false, true), CacheAction::Clone); // AC3: fresh clone
        assert_eq!(cache.decide(true, true), CacheAction::FetchCheckout); // AC3: update
        assert_eq!(cache.decide(true, false), CacheAction::ReuseStale); // AC1: offline reuse
        assert_eq!(cache.decide(false, false), CacheAction::Abort); // AC2: hard abort
    }

    // ── TS-04 — cache executor arms (resolve_cached), not just decide() ──────

    /// @us-04 @in-memory
    /// TS-04/AC1: entry present + remote unreachable → ReuseStale → the existing
    /// cache entry path is returned offline (no clone, no fetch).
    #[test]
    #[serial_test::serial]
    fn ts04_reuse_stale_arm_returns_existing_entry_offline() {
        let td = TempDir::new().unwrap();
        let cache_dir = td.path().to_str().unwrap().to_string();
        let url = "https://github.com/chess-seventh/cv.git";
        let cache = TemplateCache::new(cache_dir.clone());
        let entry = cache.entry_path(url, None);
        fs::create_dir_all(&entry).unwrap();
        let source = GitHubRepository::new(url.into(), cache_dir);
        let runner = FakeRunner::failing(); // ls-remote probe fails → unreachable

        let resolved = source.resolve_cached(&runner, &cache).unwrap();
        assert_eq!(
            resolved, entry,
            "ReuseStale must return the cached entry path"
        );
    }

    /// @us-04 @error
    /// TS-04/AC2: no entry + remote unreachable → Abort → `NoCache` (fail fast,
    /// never a partial CV).
    #[test]
    #[serial_test::serial]
    fn ts04_abort_arm_no_entry_offline_errors_nocache() {
        let td = TempDir::new().unwrap();
        let cache_dir = td.path().to_str().unwrap().to_string();
        let url = "https://github.com/chess-seventh/cv.git";
        let cache = TemplateCache::new(cache_dir.clone());
        let source = GitHubRepository::new(url.into(), cache_dir);
        let runner = FakeRunner::failing(); // ls-remote probe fails → unreachable

        let err = source.resolve_cached(&runner, &cache).unwrap_err();
        assert!(
            matches!(err, TemplateSourceError::NoCache { .. }),
            "Abort must classify as NoCache, got: {err:?}"
        );
    }

    /// @us-04 @in-memory
    /// TS-04/AC3: entry present + remote reachable → FetchCheckout → a `fetch
    /// --prune --tags` is recorded against the entry (the perf lever, not a fresh
    /// clone).
    #[test]
    #[serial_test::serial]
    fn ts04_fetch_checkout_arm_records_fetch() {
        let td = TempDir::new().unwrap();
        let cache_dir = td.path().to_str().unwrap().to_string();
        let url = "https://github.com/chess-seventh/cv.git";
        let cache = TemplateCache::new(cache_dir.clone());
        let entry = cache.entry_path(url, None);
        fs::create_dir_all(&entry).unwrap();
        let source = GitHubRepository::new(url.into(), cache_dir);
        let runner = FakeRunner::ok(); // ls-remote probe + fetch succeed

        source.resolve_cached(&runner, &cache).unwrap();
        assert!(
            runner
                .calls
                .borrow()
                .iter()
                .any(|c| c.contains("fetch --prune --tags")),
            "FetchCheckout must record a fetch against the cache entry"
        );
    }

    /// @us-04 @property
    /// BLOCKER: on the FetchCheckout path the fetch is auth-routed exactly like the
    /// clone/probe — a Token-auth refresh of an HTTPS cache entry MUST carry the
    /// `core.askpass` flag, otherwise the refresh fails auth and the cache never
    /// updates. This test is RED if the fetch invocation drops the auth flags.
    #[test]
    #[serial_test::serial]
    fn ts04_fetch_path_is_auth_routed_with_askpass() {
        let td = TempDir::new().unwrap();
        let cache_dir = td.path().to_str().unwrap().to_string();
        let url = "https://github.com/chess-seventh/cv.git";
        let cache = TemplateCache::new(cache_dir.clone());
        let entry = cache.entry_path(url, None);
        fs::create_dir_all(&entry).unwrap();
        let source = GitHubRepository::new(url.into(), cache_dir).with_auth(AuthMode::Token);
        let runner = FakeRunner::ok();

        source.resolve_cached(&runner, &cache).unwrap();

        let calls = runner.calls.borrow();
        let fetch_call = calls
            .iter()
            .find(|c| c.contains("fetch --prune --tags"))
            .expect("FetchCheckout must record a fetch");
        assert!(
            fetch_call.contains("-c core.askpass="),
            "the token fetch must carry the askpass auth flag: {fetch_call}"
        );
    }

    /// @us-04 @property @edge
    /// The cache key is deterministic (same input → same key) AND injective enough
    /// to keep distinct repos and the pinned-vs-default slots apart: distinct repos
    /// map to distinct keys, and a non-default pinned ref addresses a different
    /// slot than the shared default-branch slot.
    #[test]
    fn ts04_cache_key_is_deterministic_and_injective() {
        use proptest::prelude::*;
        let cache = TemplateCache::new("/unused/cache".into());
        proptest!(|(name_a in "[a-z]{3,12}",
                    name_b in "[a-z]{3,12}",
                    git_ref in "[a-z0-9]{1,8}")| {
            prop_assume!(name_a != name_b);
            prop_assume!(git_ref != DEFAULT_REF_SLOT);
            let url_a = format!("https://github.com/team/{name_a}.git");
            let url_b = format!("https://github.com/team/{name_b}.git");
            // determinism: same input → same key
            prop_assert_eq!(cache.cache_key(&url_a, None), cache.cache_key(&url_a, None));
            // injectivity: distinct repos must not collide on one cache slot
            prop_assert_ne!(cache.cache_key(&url_a, None), cache.cache_key(&url_b, None));
            // a pinned ref addresses a different slot than the default-branch slot
            prop_assert_ne!(
                cache.cache_key(&url_a, Some(git_ref.as_str())),
                cache.cache_key(&url_a, None)
            );
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

    // ── Mutation hardening (Phase 5) ─────────────────────────────────────────
    // Targeted tests that distinguish each surviving predicate from its mutant.
    // The marker sets are finite + enumerable, so per the PBT falsifier-gate
    // these are table-style example tests, not proptest cases.

    /// @us-02 @error @mutation
    /// Each auth-failure marker, ALONE, classifies as `Auth`. Kills the
    /// `is_auth_failure` `||`→`&&` mutants: under `&&` a stderr carrying only ONE
    /// of the four markers would stop classifying as auth and fall through.
    #[test]
    fn ts02_each_auth_marker_alone_classifies_as_auth() {
        let url = "git@github.com:chess-seventh/cv.git";
        for marker in [
            "Permission denied (publickey)",
            "Authentication failed",
            "could not read Username",
            "Invalid username or password",
        ] {
            let err = classify_git_stderr(marker, url, None);
            assert!(
                matches!(err, TemplateSourceError::Auth { .. }),
                "auth marker {marker:?} alone must classify as Auth, got: {err:?}"
            );
        }
    }

    /// @us-03 @error @mutation
    /// Each bad-ref marker ALONE (with a pinned ref) classifies as `BadRef`,
    /// killing the `is_bad_ref` `||`→`&&` mutants; AND an UNRELATED stderr (still
    /// with a pinned ref) classifies as `NetworkOffline`, killing the
    /// `is_bad_ref -> true` mutant (which would coerce anything to `BadRef`).
    #[test]
    fn ts03_each_bad_ref_marker_alone_and_unrelated_stays_network() {
        let url = "https://github.com/chess-seventh/cv.git";
        for marker in [
            "couldn't find remote ref",
            "did not match any file(s) known to git",
            "Remote branch",
            "not found in upstream",
        ] {
            let err = classify_git_stderr(marker, url, Some("v9"));
            assert!(
                matches!(err, TemplateSourceError::BadRef { .. }),
                "bad-ref marker {marker:?} alone must classify as BadRef, got: {err:?}"
            );
        }
        let unrelated = classify_git_stderr(
            "fatal: unable to access: Could not resolve host",
            url,
            Some("v9"),
        );
        assert!(
            matches!(unrelated, TemplateSourceError::NetworkOffline { .. }),
            "an unrelated stderr must stay NetworkOffline, got: {unrelated:?}"
        );
    }

    /// @us-02 @mutation
    /// `AuthMode::Auto` infers the transport from the URL scheme: both `git@…` and
    /// `ssh://…` remotes infer SSH (→ NO askpass flags), anything else infers Token
    /// (→ askpass flags present). Asserting BOTH ssh-scheme forms yield empty flags
    /// kills the `inferred_from_url` `||`→`&&` mutant (under `&&` no url matches
    /// both schemes → always Token → non-empty flags).
    #[test]
    fn ts02_auto_infers_ssh_for_both_ssh_schemes() {
        assert!(
            auth_invocation_flags(AuthMode::Auto, "git@github.com:chess-seventh/cv.git").is_empty(),
            "a git@ remote infers SSH — no askpass flags"
        );
        assert!(
            auth_invocation_flags(AuthMode::Auto, "ssh://git@github.com/chess-seventh/cv.git")
                .is_empty(),
            "an ssh:// remote infers SSH — no askpass flags"
        );
        assert!(
            !auth_invocation_flags(AuthMode::Auto, "https://github.com/chess-seventh/cv.git")
                .is_empty(),
            "an https remote infers Token — askpass flags present"
        );
    }

    /// @us-02 @error @mutation
    /// The `Auth` error Display embeds the EXACT auth-mode hint name, so the
    /// operator hint is accurate. Kills the `hint_name -> "" / "xyzzy"` mutant: a
    /// replaced body would drop the real "auto"/"ssh"/"token" text.
    #[test]
    fn ts02_auth_error_display_contains_exact_mode_hint() {
        for (mode, hint) in [
            (AuthMode::Auto, "auto"),
            (AuthMode::Ssh, "ssh"),
            (AuthMode::Token, "token"),
        ] {
            let err = TemplateSourceError::Auth {
                url: "https://github.com/chess-seventh/cv.git".to_string(),
                mode,
            };
            let rendered = err.to_string();
            assert!(
                rendered.contains(&format!("auth mode: {hint}")),
                "Auth Display must name the {hint:?} mode, got: {rendered}"
            );
        }
    }

    /// @us-04 @mutation
    /// `entry_path` returns a real path UNDER the cache dir whose final component is
    /// the sanitised `repo@ref` cache key — not an arbitrary string. Kills the
    /// `entry_path -> "xyzzy".into()` mutant.
    #[test]
    fn ts04_entry_path_is_under_cache_dir_with_sanitised_key() {
        let cache = TemplateCache::new("/var/cache/cv".into());
        let url = "https://github.com/chess-seventh/cv.git";
        let path = cache.entry_path(url, Some("v2.1"));
        let key = cache.cache_key(url, Some("v2.1"));
        assert!(
            path.starts_with("/var/cache/cv/"),
            "entry_path must live under the cache dir, got: {path}"
        );
        assert!(
            path.ends_with(&key),
            "entry_path must end with the sanitised cache key, got: {path}"
        );
    }

    /// @us-02 @mutation
    /// `resolve_classified` must actually prepare the destination — creating the
    /// cache entry's parent directory so the clone can land. With a non-existent
    /// nested cache dir, that directory is created as an observable side effect.
    /// Kills the `prepare_destination -> Ok(())` mutant, which would skip the work.
    #[test]
    #[serial_test::serial]
    fn ts02_resolve_classified_prepares_missing_cache_parent_dir() {
        let td = TempDir::new().unwrap();
        let cache_dir = format!("{}/nested/cache", td.path().display());
        assert!(
            !Path::new(&cache_dir).exists(),
            "precondition: nested cache dir must be absent"
        );
        let source = GitHubRepository::new(
            "https://github.com/chess-seventh/cv.git".into(),
            cache_dir.clone(),
        );
        let runner = FakeRunner::ok();

        source.resolve_classified(&runner).unwrap();

        assert!(
            Path::new(&cache_dir).is_dir(),
            "prepare_destination must create the cache entry's parent dir"
        );
    }

    /// @us-01 @us-04 @mutation
    /// The production resolver routes a (non-dir) git URL through the cache-aware
    /// git path — which probes the remote with `ls-remote` before deciding — NOT
    /// through the skeleton passthrough. Kills the `resolve_template_for_config`
    /// `delete !` mutant, which would force EVERY value (git URL included) down the
    /// passthrough branch where no `ls-remote` probe happens.
    #[test]
    #[serial_test::serial]
    fn ts04_config_resolver_routes_git_url_through_cache_probe() {
        let td = TempDir::new().unwrap();
        let cache_dir = td.path().to_str().unwrap().to_string();
        let url = "https://github.com/chess-seventh/cv.git";
        let runner = FakeRunner::ok();

        resolve_template_for_config(url, &cache_dir, None, AuthMode::Token, &runner).unwrap();

        assert!(
            runner
                .calls
                .borrow()
                .iter()
                .any(|c| c.contains("ls-remote")),
            "a git URL must go through the cache-aware path (ls-remote probe), calls: {:?}",
            runner.calls.borrow()
        );
    }
}
