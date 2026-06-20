use std::io;
use std::process::{Command, Stdio};

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
}

#[cfg(test)]
pub mod testing {
    use super::{CommandRunner, io};
    use std::cell::RefCell;

    /// A configurable fake runner for tests. Records the programs invoked and
    /// returns canned results.
    pub struct FakeRunner {
        pub succeed: bool,
        pub stdout: String,
        pub fail_io: bool,
        pub calls: RefCell<Vec<String>>,
    }

    impl FakeRunner {
        pub fn ok() -> Self {
            Self {
                succeed: true,
                stdout: String::new(),
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
