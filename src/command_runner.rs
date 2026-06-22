use std::io;
use std::process::{Command, Stdio};

/// Captured outcome of a subprocess run (UC-1, feature `template-source`):
/// success plus both streams, so a caller can classify a git failure from its
/// stderr (auth vs network/offline vs bad-ref). Additive to the port; existing
/// `status`/`output`/`spawn` call sites are unaffected.
#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Abstraction over running external programs so the build / view / Tailscale
/// logic can be unit-tested with a fake instead of really shelling out.
pub trait CommandRunner {
    /// Run `program args...` (optionally in `cwd`) to completion; return whether
    /// it exited successfully.
    fn status(&self, program: &str, args: &[&str], cwd: Option<&str>) -> io::Result<bool>;

    /// Run `program args...` and capture `(success, stdout)`.
    fn output(&self, program: &str, args: &[&str]) -> io::Result<(bool, String)>;

    /// Launch `program args...` detached (used for the PDF viewer).
    fn spawn(&self, program: &str, args: &[&str]) -> io::Result<()>;

    /// UC-1 (feature `template-source`): run `program args...` (optionally in
    /// `cwd`) capturing success + stdout + stderr, so a git failure can be
    /// classified by inspecting stderr. Additive and backward-compatible —
    /// implementors that capture stderr (e.g. `SystemRunner`) override this;
    /// the default is a best-effort delegation to `output` with empty stderr,
    /// so existing implementors keep compiling without change.
    fn run_capturing(
        &self,
        program: &str,
        args: &[&str],
        _cwd: Option<&str>,
    ) -> io::Result<CommandOutcome> {
        let (success, stdout) = self.output(program, args)?;
        Ok(CommandOutcome {
            success,
            stdout,
            stderr: String::new(),
        })
    }
}

/// The real runner, backed by `std::process::Command`.
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn status(&self, program: &str, args: &[&str], cwd: Option<&str>) -> io::Result<bool> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        Ok(cmd.status()?.success())
    }

    fn output(&self, program: &str, args: &[&str]) -> io::Result<(bool, String)> {
        let out = Command::new(program).args(args).output()?;
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        Ok((out.status.success(), stdout))
    }

    fn spawn(&self, program: &str, args: &[&str]) -> io::Result<()> {
        Command::new(program)
            .args(args)
            .stdout(Stdio::null())
            .spawn()?;
        Ok(())
    }

    fn run_capturing(
        &self,
        program: &str,
        args: &[&str],
        cwd: Option<&str>,
    ) -> io::Result<CommandOutcome> {
        let mut cmd = Command::new(program);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let out = cmd.output()?;
        Ok(CommandOutcome {
            success: out.status.success(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
pub mod testing {
    use super::{CommandOutcome, CommandRunner, io};
    use std::cell::RefCell;

    /// A configurable fake runner for tests. Records the programs invoked and
    /// returns canned results.
    pub struct FakeRunner {
        pub succeed: bool,
        pub stdout: String,
        pub stderr: String,
        pub fail_io: bool,
        pub calls: RefCell<Vec<String>>,
    }

    impl FakeRunner {
        pub fn ok() -> Self {
            Self {
                succeed: true,
                stdout: String::new(),
                stderr: String::new(),
                fail_io: false,
                calls: RefCell::new(Vec::new()),
            }
        }

        pub fn failing() -> Self {
            Self {
                succeed: false,
                ..Self::ok()
            }
        }

        /// A failing runner whose captured `stderr` is the given string, so a
        /// caller can assert the failure is classified by its stderr content
        /// (auth vs bad-ref vs network) rather than by the single canned default.
        pub fn failing_with_stderr(stderr: &str) -> Self {
            Self {
                succeed: false,
                stderr: stderr.to_string(),
                ..Self::ok()
            }
        }

        pub fn io_error() -> Self {
            Self {
                fail_io: true,
                ..Self::ok()
            }
        }

        pub fn with_stdout(stdout: &str) -> Self {
            Self {
                stdout: stdout.to_string(),
                ..Self::ok()
            }
        }

        fn record(&self, program: &str, args: &[&str]) {
            self.calls
                .borrow_mut()
                .push(format!("{program} {}", args.join(" ")));
        }

        fn maybe_err<T>(&self, ok: T) -> io::Result<T> {
            if self.fail_io {
                Err(io::Error::new(io::ErrorKind::NotFound, "fake io error"))
            } else {
                Ok(ok)
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn status(&self, program: &str, args: &[&str], _cwd: Option<&str>) -> io::Result<bool> {
            self.record(program, args);
            self.maybe_err(self.succeed)
        }

        fn output(&self, program: &str, args: &[&str]) -> io::Result<(bool, String)> {
            self.record(program, args);
            self.maybe_err((self.succeed, self.stdout.clone()))
        }

        fn spawn(&self, program: &str, args: &[&str]) -> io::Result<()> {
            self.record(program, args);
            self.maybe_err(())
        }

        fn run_capturing(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&str>,
        ) -> io::Result<CommandOutcome> {
            self.record(program, args);
            let stderr = if self.succeed {
                String::new()
            } else if self.stderr.is_empty() {
                "fatal: could not read from remote repository.".to_string()
            } else {
                self.stderr.clone()
            };
            self.maybe_err(CommandOutcome {
                success: self.succeed,
                stdout: self.stdout.clone(),
                stderr,
            })
        }
    }

    #[test]
    fn test_fake_runner_records_and_returns() {
        let fake = FakeRunner::ok();
        assert!(fake.status("just", &["build", "x"], None).unwrap());
        assert_eq!(fake.calls.borrow().len(), 1);
        assert_eq!(fake.calls.borrow()[0], "just build x");
    }

    #[test]
    fn test_fake_runner_io_error() {
        let fake = FakeRunner::io_error();
        assert!(fake.spawn("zathura", &["a.pdf"]).is_err());
    }
}

#[cfg(test)]
mod uc1_specs {
    //! DISTILL spec for UC-1 (feature `template-source`): the additive
    //! stderr-capturing run. Pending DELIVER — calls the RED scaffold default
    //! `run_capturing`, so it fails by `panic!` (RED), not by a compile error.
    use super::CommandRunner;
    use super::testing::FakeRunner;

    /// @us-02 @in-memory
    /// UC-1: a captured run exposes stderr so a git failure can be classified
    /// into a distinct `TemplateSourceError` (auth vs network vs bad-ref).
    #[test]
    fn uc1_run_capturing_exposes_stderr() {
        let runner = FakeRunner::failing();
        let outcome = runner
            .run_capturing(
                "git",
                &["clone", "git@github.com:chess-seventh/cv.git"],
                None,
            )
            .expect("captured run should return an outcome, not an io error");
        assert!(!outcome.success, "a failing clone reports success=false");
        assert!(
            !outcome.stderr.is_empty(),
            "stderr must be captured so the failure can be classified"
        );
    }
}
