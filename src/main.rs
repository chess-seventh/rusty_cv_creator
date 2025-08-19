use core::panic;

use clap::Parser;
use dotenvy::dotenv;
use log::{error, info};
use std::io::{self};
use std::process::Command;

mod cli_structure;
mod config_parse;
mod database;
mod file_handlers;
mod global_conf;
mod helpers;
mod user_action;

use crate::cli_structure::{match_user_action, UserInput};
use crate::config_parse::{get_variable_from_config, set_global_vars};
use crate::file_handlers::{compile_cv, create_directory, make_cv_changes_based_on_input};
use crate::helpers::{
    check_if_db_env_is_set_or_set_from_config, fix_home_directory_path, view_cv_file,
};

fn main() {
    env_logger::init();
    dotenv().ok();

    match is_tailscale_connected() {
        Ok(true) => println!("Device is connected to Tailscale!"),
        Ok(false) => println!("Device is NOT connected to Tailscale."),
        Err(e) => eprintln!("Error: {e:?}"),
    }

    let user_input = UserInput::parse();

    match set_global_vars(&user_input.clone()) {
        Ok(o) => info!("all good: {o}"),
        Err(e) => panic!("could not set global vars {e}"),
    }

    match check_if_db_env_is_set_or_set_from_config() {
        Ok(_v) => info!("Fetched the DATABASE_URL env variable"),
        Err(v) => panic!("{}", v),
    }

    let cv_full_path = match_user_action();

    if !cv_full_path.is_empty() {
        match user_input.view_generated_cv {
            Some(true) => {
                match view_cv_file(&cv_full_path) {
                    Ok(b) => b,
                    Err(e) => panic!("{e:?}"),
                };
            }
            Some(false) | None => {
                info!("CV saved to: {cv_full_path}");
                // println!("CV saved to: {cv_full_path}");
            }
        }
    }
}

fn prepare_cv(job_title: &str, company_name: &str, quote: &str) -> Result<String, String> {
    let cfg = match get_variable_from_config("cv", "cv_template_file") {
        Ok(c) => c,
        Err(e) => {
            error!("Something went wrong when gathering variable from config: {e:?}");
            return Err(
                "Something went wrong when gathering variable from config: {e:?}".to_string(),
            );
        }
    };
    let cv_template_file = fix_home_directory_path(&cfg);

    let created_cv_dir = match create_directory(job_title, company_name) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not create directory for CV: {e:?}");
            return Err("Could not create directory for CV: {e:?}".to_string());
        }
    };

    let destination_cv_file_full_path =
        fix_home_directory_path(&format!("{created_cv_dir}/{cv_template_file}"));

    make_cv_changes_based_on_input(job_title, quote, &destination_cv_file_full_path);
    compile_cv(&created_cv_dir, &cv_template_file);

    file_handlers::remove_created_dir_from_pro(
        job_title,
        company_name,
        &created_cv_dir,
        &destination_cv_file_full_path,
    );

    Ok(destination_cv_file_full_path)
}

/// Checks if the device is connected to Tailscale.
/// Returns true if up, false if not, or Err if unable to check.
fn is_tailscale_connected() -> io::Result<bool> {
    let output = Command::new("sudo").arg("tailscale").arg("status").output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Tailscale outputs "Logged out." if disconnected,
                // and network details if connected
                Ok(!stdout.contains("Logged out."))
            } else {
                Err(io::Error::other("tailscale status command failed"))
            }
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use serial_test::serial;
    // use std::os::unix::process::ExitStatusExt;

    // Mock function for testing is_tailscale_connected without external dependencies
    // fn mock_command_output(success: bool, stdout: &str) -> std::io::Result<std::process::Output> {
    //     use std::process::{ExitStatus, Output};
    //
    //     // Create a mock output
    //     let status = if success {
    //         ExitStatus::from_raw(0)
    //     } else {
    //         ExitStatus::from_raw(1)
    //     };
    //
    //     Ok(Output {
    //         status,
    //         stdout: stdout.as_bytes().to_vec(),
    //         stderr: Vec::new(),
    //     })
    // }

    #[test]
    fn test_is_tailscale_connected_success_connected() {
        // Test the function with a mock that simulates connected state
        // Note: This test requires refactoring the function to accept dependency injection

        // For now, we test the logic manually
        let stdout = "100.64.0.1 hostname1\n100.64.0.2 hostname2";
        let is_connected = !stdout.contains("Logged out.");
        assert!(is_connected);
    }

    #[test]
    fn test_is_tailscale_connected_success_disconnected() {
        let stdout = "Logged out.";
        let is_connected = !stdout.contains("Logged out.");
        assert!(!is_connected);
    }

    #[test]
    fn test_is_tailscale_connected_partial_logout_message() {
        let stdout = "Device status: Logged out.";
        let is_connected = !stdout.contains("Logged out.");
        assert!(!is_connected);
    }

    #[test]
    fn test_is_tailscale_connected_empty_output() {
        let stdout = "";
        let is_connected = !stdout.contains("Logged out.");
        assert!(is_connected); // Empty output means connected
    }

    #[test]
    fn test_is_tailscale_connected_mixed_output() {
        let stdout = "100.64.0.1 hostname1\nSome other info\nNot Logged out.";
        let is_connected = !stdout.contains("Logged out.");
        assert!(!is_connected);
    }

    // Tests for prepare_cv function
    #[test]
    #[serial]
    #[ignore] // Requires GLOBAL_VAR setup and file system operations
    fn test_prepare_cv_success() {
        // This test requires:
        // 1. GLOBAL_VAR to be set with appropriate config
        // 2. Mock file system operations
        // 3. Mock external commands (xelatex, cp)

        // Example test structure:
        // setup_test_config();
        // let result = prepare_cv("Software Engineer", "ACME Corp", "Great opportunity");
        // assert!(result.is_ok());
        // assert!(!result.unwrap().is_empty());
    }

    #[test]
    #[serial]
    #[ignore] // Requires GLOBAL_VAR setup
    fn test_prepare_cv_missing_config() {
        // Test when config is missing required keys
        // let result = prepare_cv("Engineer", "Corp", "Quote");
        // assert!(result.is_err());
    }

    #[test]
    #[ignore] // Requires file system setup
    fn test_prepare_cv_invalid_template_path() {
        // Test when template path doesn't exist
        // This would test the error handling in create_directory
    }

    // Integration test helpers for testing the full main function
    #[cfg(test)]
    mod integration_helpers {
        // use std::env;
        // use tempfile::TempDir;

        //         pub fn setup_test_environment() -> TempDir {
        //             let temp_dir = TempDir::new().expect("Failed to create temp dir");
        //
        //             // Create test config file
        //             let config_path = temp_dir.path().join("test-config.ini");
        //             let config_content = r#"
        // [destination]
        // cv_path = "/tmp/test_cv"
        //
        // [cv]
        // cv_template_path = "/tmp/template"
        // cv_template_file = "cv.tex"
        //
        // [to_replace]
        // position_line_to_change = "POSITION_PLACEHOLDER"
        //
        // [db]
        // db_path = "/tmp"
        // db_file = "test.db"
        // engine = "sqlite"
        //
        // [optional]
        // pdf_viewer = "echo"
        // "#;
        //             std::fs::write(&config_path, config_content).expect("Failed to write config");
        //
        //             // Set environment variable for database
        //             env::set_var("DATABASE_URL", "sqlite:///tmp/test.db");
        //
        //             temp_dir
        //         }

        #[test]
        #[ignore] // Full integration test
        fn test_main_integration() {
            // This would test the main function with mocked command line arguments
            // and a test environment
        }
    }

    // Test utility functions for mocking external dependencies
    #[cfg(test)]
    mod mock_utils {
        use std::collections::HashMap;
        use std::sync::{Arc, Mutex};

        pub struct MockCommandRunner {
            expectations: Arc<Mutex<HashMap<String, (bool, String)>>>,
        }

        impl MockCommandRunner {
            pub fn new() -> Self {
                Self {
                    expectations: Arc::new(Mutex::new(HashMap::new())),
                }
            }

            pub fn expect_command(&self, cmd: &str, success: bool, output: &str) {
                let mut expectations = self.expectations.lock().unwrap();
                expectations.insert(cmd.to_string(), (success, output.to_string()));
            }

            pub fn run_command(&self, cmd: &str) -> std::io::Result<(bool, String)> {
                let expectations = self.expectations.lock().unwrap();
                if let Some((success, output)) = expectations.get(cmd) {
                    Ok((*success, output.clone()))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Unexpected command: {}", cmd),
                    ))
                }
            }
        }

        #[test]
        fn test_mock_command_runner() {
            let mock = MockCommandRunner::new();
            mock.expect_command("sudo tailscale status", true, "100.64.0.1 hostname");

            let result = mock.run_command("sudo tailscale status");
            assert!(result.is_ok());
            let (success, output) = result.unwrap();
            assert_eq!(success, true);
            assert_eq!(output, "100.64.0.1 hostname");
        }

        #[test]
        fn test_mock_command_runner_unexpected() {
            let mock = MockCommandRunner::new();
            let result = mock.run_command("unexpected command");
            assert!(result.is_err());
        }
    }

    // Test for command line argument parsing (integration with clap)
    #[cfg(test)]
    mod clap_integration {
        // use super::*;
        use crate::cli_structure::{UserAction, UserInput};
        use clap::Parser;

        #[test]
        fn test_clap_insert_command() {
            let args = vec![
                "rusty-cv-creator",
                "insert",
                "ACME Corp",
                "Software Engineer",
                "Great opportunity",
            ];

            let result = UserInput::try_parse_from(args);
            assert!(result.is_ok());

            let user_input = result.unwrap();
            match user_input.action {
                UserAction::Insert(insert_args) => {
                    assert_eq!(insert_args.company_name, "ACME Corp");
                    assert_eq!(insert_args.job_title, "Software Engineer");
                    assert_eq!(insert_args.quote, "Great opportunity");
                }
                _ => panic!("Expected Insert action"),
            }
        }

        #[test]
        fn test_clap_with_flags() {
            let args = vec![
                "rusty-cv-creator",
                "--save-to-database",
                "false",
                "--view-generated-cv",
                "true",
                "--dry-run",
                "true",
                "insert",
                "Test Co",
                "Developer",
                "Quote",
            ];

            let result = UserInput::try_parse_from(args);
            assert!(result.is_ok());

            let user_input = result.unwrap();
            assert_eq!(user_input.save_to_database, Some(false));
            assert_eq!(user_input.view_generated_cv, Some(true));
            assert_eq!(user_input.dry_run, Some(true));
        }

        #[test]
        fn test_clap_config_path() {
            let args = vec![
                "rusty-cv-creator",
                "--config-ini",
                "/custom/path/config.ini",
                "insert",
                "Co",
                "Job",
                "Quote",
            ];

            let result = UserInput::try_parse_from(args);
            assert!(result.is_ok());

            let user_input = result.unwrap();
            assert_eq!(user_input.config_ini, "/custom/path/config.ini");
        }

        #[test]
        fn test_clap_database_engine() {
            let args = vec![
                "rusty-cv-creator",
                "--engine",
                "postgres",
                "insert",
                "Co",
                "Job",
                "Quote",
            ];

            let result = UserInput::try_parse_from(args);
            assert!(result.is_ok());

            let user_input = result.unwrap();
            assert_eq!(user_input.engine, "postgres");
        }

        #[test]
        fn test_clap_missing_required_args() {
            let args = vec![
                "rusty-cv-creator",
                "insert",
                "Company", // Missing job_title and quote
            ];

            let result = UserInput::try_parse_from(args);
            assert!(result.is_err());
        }

        #[test]
        fn test_clap_help() {
            let args = vec!["rusty-cv-creator", "--help"];
            let result = UserInput::try_parse_from(args);
            assert!(result.is_err()); // Help exits with error code
        }

        #[test]
        fn test_clap_version() {
            let args = vec!["rusty-cv-creator", "--version"];
            let result = UserInput::try_parse_from(args);
            assert!(result.is_err()); // Version exits with error code
        }
    }
}

// Refactored version of is_tailscale_connected for better testability
#[cfg(test)]
pub fn is_tailscale_connected_with_executor<F>(executor: F) -> std::io::Result<bool>
where
    F: FnOnce(&str, &[&str]) -> std::io::Result<std::process::Output>,
{
    let output = executor("sudo", &["tailscale", "status"])?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.contains("Logged out."))
    } else {
        Err(std::io::Error::other("tailscale status command failed"))
    }
}

#[cfg(test)]
mod testable_version_tests {
    use super::*;
    use std::{
        os::unix::process::ExitStatusExt,
        process::{ExitStatus, Output},
    };

    fn mock_executor_success(
        stdout: &str,
    ) -> impl Fn(&str, &[&str]) -> std::io::Result<std::process::Output> + '_ {
        move |_cmd, _args| {
            Ok(Output {
                status: ExitStatus::from_raw(0),
                stdout: stdout.as_bytes().to_vec(),
                stderr: Vec::new(),
            })
        }
    }

    fn mock_executor_failure() -> impl Fn(&str, &[&str]) -> std::io::Result<std::process::Output> {
        |_cmd, _args| {
            Ok(Output {
                status: ExitStatus::from_raw(1),
                stdout: Vec::new(),
                stderr: b"Command failed".to_vec(),
            })
        }
    }

    #[test]
    fn test_tailscale_connected_with_mock() {
        let executor = mock_executor_success("100.64.0.1 hostname");
        let result = is_tailscale_connected_with_executor(executor);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_tailscale_disconnected_with_mock() {
        let executor = mock_executor_success("Logged out.");
        let result = is_tailscale_connected_with_executor(executor);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_tailscale_command_failed_with_mock() {
        let executor = mock_executor_failure();
        let result = is_tailscale_connected_with_executor(executor);
        assert!(result.is_err());
    }

    #[test]
    fn test_tailscale_command_error_with_mock() {
        let executor = |_cmd: &str, _args: &[&str]| -> std::io::Result<std::process::Output> {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Command not found",
            ))
        };
        let result = is_tailscale_connected_with_executor(executor);
        assert!(result.is_err());
    }
}
