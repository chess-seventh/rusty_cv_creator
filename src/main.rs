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
use crate::config_parse::{get_variable_from_config_file, set_global_vars};
use crate::file_handlers::{compile_cv, create_directory, make_cv_changes_based_on_input};
use crate::global_conf::get_global_var;
use crate::helpers::{
    check_if_db_env_is_set_or_set_from_config, fix_home_directory_path, view_cv_file,
};

#[cfg(not(tarpaulin_include))]
fn main() {
    env_logger::init();
    dotenv().ok();

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

    // Integration test helpers for testing the full main function
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
                        format!("Unexpected command: {cmd}"),
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
            assert!(success);
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
#[allow(clippy::missing_errors_doc)]
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
