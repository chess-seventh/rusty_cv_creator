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
