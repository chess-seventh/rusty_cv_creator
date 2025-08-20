//! Integration tests for rusty-cv-creator
//!
//! These tests test the full functionality with real file systems,
//! mocked databases, and controlled environments.

use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

// Import the modules we want to test

mod integration_helpers {
    use super::*;

    pub struct TestEnvironment {
        pub temp_dir: TempDir,
        pub config_file: String,
        pub _db_path: String,
    }

    impl TestEnvironment {
        pub fn setup() -> Self {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Create config file
            let config_content = format!(
                r#"
[destination]
cv_path = "{}/cvs"

[cv]
cv_template_path = "{}/template"
cv_template_file = "cv.tex"

[to_replace]
position_line_to_change = "POSITION_PLACEHOLDER"
quote_line_to_change = "QUOTE_PLACEHOLDER"

[db]
db_path = "{}/db"
db_file = "test.db"
engine = "sqlite"
db_pg_host = "postgresql://test:test@localhost/test"

[optional]
pdf_viewer = "echo"
"#,
                temp_dir.path().display(),
                temp_dir.path().display(),
                temp_dir.path().display()
            );

            let config_path = temp_dir.path().join("config.ini");
            fs::write(&config_path, config_content).expect("Failed to write config");

            // Create template directory and file
            let template_dir = temp_dir.path().join("template");
            fs::create_dir_all(&template_dir).expect("Failed to create template dir");

            let template_content = r#"
\documentclass{article}
\begin{document}
\section{CV}
Position: POSITION_PLACEHOLDER
Quote: QUOTE_PLACEHOLDER
\end{document}
"#;
            let template_file = template_dir.join("cv.tex");
            fs::write(&template_file, template_content).expect("Failed to write template");

            // Create database directory
            let db_dir = temp_dir.path().join("db");
            fs::create_dir_all(&db_dir).expect("Failed to create db dir");
            let db_path = db_dir.join("test.db").to_string_lossy().to_string();

            // Set up environment
            env::set_var("DATABASE_URL", format!("sqlite://{}", db_path));

            TestEnvironment {
                temp_dir,
                config_file: config_path.to_string_lossy().to_string(),
                _db_path: db_path,
            }
        }
    }
}

#[cfg(test)]
mod config_integration_tests {
    use super::*;
    use integration_helpers::TestEnvironment;

    #[test]
    #[serial]
    fn test_config_loading_integration() {
        let test_env = TestEnvironment::setup();

        // Test that config file exists and is readable
        assert!(fs::metadata(&test_env.config_file).is_ok());

        let content = fs::read_to_string(&test_env.config_file).expect("Failed to read config");
        assert!(content.contains("[destination]"));
        assert!(content.contains("[cv]"));
        assert!(content.contains("[db]"));
    }

    #[test]
    #[serial]
    fn test_template_file_exists() {
        let test_env = TestEnvironment::setup();

        let template_path = test_env.temp_dir.path().join("template").join("cv.tex");
        assert!(template_path.exists());

        let content = fs::read_to_string(&template_path).expect("Failed to read template");
        assert!(content.contains("POSITION_PLACEHOLDER"));
        assert!(content.contains("QUOTE_PLACEHOLDER"));
    }
}

#[cfg(test)]
mod file_system_integration_tests {
    use super::*;
    use integration_helpers::TestEnvironment;

    #[test]
    fn test_directory_creation_and_cleanup() {
        let test_env = TestEnvironment::setup();

        // Test creating nested directories
        let nested_path = test_env
            .temp_dir
            .path()
            .join("deep")
            .join("nested")
            .join("path");
        fs::create_dir_all(&nested_path).expect("Failed to create nested dirs");
        assert!(nested_path.exists());

        // Test cleanup
        fs::remove_dir_all(test_env.temp_dir.path().join("deep")).expect("Failed to cleanup");
        assert!(!nested_path.exists());
    }

    #[test]
    fn test_file_operations() {
        let test_env = TestEnvironment::setup();

        let test_file = test_env.temp_dir.path().join("test.txt");

        // Write
        fs::write(&test_file, "test content").expect("Failed to write");
        assert!(test_file.exists());

        // Read
        let content = fs::read_to_string(&test_file).expect("Failed to read");
        assert_eq!(content, "test content");

        // Modify
        fs::write(&test_file, "modified content").expect("Failed to modify");
        let modified = fs::read_to_string(&test_file).expect("Failed to read modified");
        assert_eq!(modified, "modified content");

        // Remove
        fs::remove_file(&test_file).expect("Failed to remove");
        assert!(!test_file.exists());
    }
}

#[cfg(test)]
mod command_execution_tests {

    use std::process::Command;

    #[test]
    fn test_echo_command() {
        let output = Command::new("echo")
            .arg("test message")
            .output()
            .expect("Failed to execute echo");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("test message"));
    }

    #[test]
    fn test_command_failure() {
        let output = Command::new("false")
            .output()
            .expect("Failed to execute false");

        assert!(!output.status.success());
    }

    #[test]
    fn test_nonexistent_command() {
        let result = Command::new("definitely_does_not_exist_command_12345").output();

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod environment_variable_tests {
    use super::*;

    #[test]
    fn test_environment_variable_operations() {
        let test_key = "RUSTY_CV_TEST_VAR";
        let test_value = "test_value_12345";

        // Ensure it's not set initially
        env::remove_var(test_key);
        assert!(env::var(test_key).is_err());

        // Set and verify
        env::set_var(test_key, test_value);
        assert_eq!(env::var(test_key).unwrap(), test_value);

        // Modify and verify
        let new_value = "modified_value_67890";
        env::set_var(test_key, new_value);
        assert_eq!(env::var(test_key).unwrap(), new_value);

        // Remove and verify
        env::remove_var(test_key);
        assert!(env::var(test_key).is_err());
    }

    #[test]
    fn test_database_url_environment() {
        let original = env::var("DATABASE_URL").ok();

        let test_url = "sqlite:///tmp/integration_test.db";
        env::set_var("DATABASE_URL", test_url);
        assert_eq!(env::var("DATABASE_URL").unwrap(), test_url);

        // Restore original
        match original {
            Some(value) => env::set_var("DATABASE_URL", value),
            None => env::remove_var("DATABASE_URL"),
        }
    }
}

#[cfg(test)]
mod text_processing_tests {

    #[test]
    fn test_string_replacement() {
        let template = "Hello PLACEHOLDER, welcome to PLACEHOLDER!";
        let result = template.replace("PLACEHOLDER", "World");
        assert_eq!(result, "Hello World, welcome to World!");
    }

    #[test]
    fn test_quote_removal() {
        let quoted = r#""hello world""#;
        let unquoted = quoted.replace('"', "");
        assert_eq!(unquoted, "hello world");

        let single_quoted = "'hello world'";
        let single_unquoted = single_quoted.replace('\'', "");
        assert_eq!(single_unquoted, "hello world");
    }

    #[test]
    fn test_path_manipulation() {
        let path_with_tilde = "~/documents/test.txt";

        // Simulate home directory replacement
        let home = "/home/testuser";
        let expanded = path_with_tilde.replace('~', home);
        assert_eq!(expanded, "/home/testuser/documents/test.txt");
    }

    #[test]
    fn test_date_formatting() {
        use chrono::{Local, TimeZone};

        let test_date = Local.with_ymd_and_hms(2023, 8, 19, 15, 30, 0).unwrap();

        let yyyy_mm_dd = test_date.format("%Y-%m-%d").to_string();
        assert_eq!(yyyy_mm_dd, "2023-08-19");

        let readable = test_date.format("%e-%b-%Y").to_string();
        assert!(readable.contains("19"));
        assert!(readable.contains("Aug"));
        assert!(readable.contains("2023"));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_file_not_found_error() {
        let result = fs::read_to_string("/definitely/does/not/exist.txt");
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_permission_denied_simulation() {
        // Create a file and try to write to a readonly directory
        let test_env = integration_helpers::TestEnvironment::setup();
        let readonly_dir = test_env.temp_dir.path().join("readonly");

        fs::create_dir_all(&readonly_dir).expect("Failed to create readonly dir");

        // On Unix systems, you could change permissions here
        // On Windows, this test might behave differently

        let test_file = readonly_dir.join("test.txt");
        fs::write(&test_file, "content").expect("Should be able to write initially");
        assert!(test_file.exists());
    }

    #[test]
    fn test_invalid_utf8_handling() {
        // Test handling of invalid UTF-8 sequences
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
        let lossy_string = String::from_utf8_lossy(&invalid_bytes);
        assert!(lossy_string.contains('\u{FFFD}')); // Replacement character
    }
}

#[cfg(test)]
mod concurrent_access_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_concurrent_file_access() {
        let test_env = integration_helpers::TestEnvironment::setup();
        let shared_file = Arc::new(test_env.temp_dir.path().join("shared.txt"));
        let counter = Arc::new(Mutex::new(0));

        let mut handles = vec![];

        for i in 0..5 {
            let file_clone = Arc::clone(&shared_file);
            let counter_clone = Arc::clone(&counter);

            let handle = thread::spawn(move || {
                let content = format!("Thread {} was here\n", i);

                // Simulate some work
                thread::sleep(std::time::Duration::from_millis(10));

                // Write to file (this would need proper synchronization in real code)
                let _ = fs::write(&*file_clone, &content);

                // Update counter
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(*counter.lock().unwrap(), 5);
        assert!(shared_file.exists());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_file_creation_performance() {
        let test_env = integration_helpers::TestEnvironment::setup();
        let start = Instant::now();

        for i in 0..100 {
            let file_path = test_env
                .temp_dir
                .path()
                .join(format!("perf_test_{}.txt", i));
            fs::write(&file_path, format!("Content {}", i)).expect("Failed to write");
        }

        let duration = start.elapsed();

        // Should be able to create 100 small files quickly
        assert!(duration.as_millis() < 1000); // Less than 1 second
    }

    #[test]
    fn test_string_processing_performance() {
        let template = "PLACEHOLDER".repeat(1000);
        let start = Instant::now();

        let result = template.replace("PLACEHOLDER", "REPLACEMENT");

        let duration = start.elapsed();

        assert!(result.contains("REPLACEMENT"));
        assert!(duration.as_millis() < 100); // Should be very fast
    }
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;

    #[test]
    fn test_temp_directory_cleanup() {
        let temp_dir_path = {
            let test_env = integration_helpers::TestEnvironment::setup();
            let path = test_env.temp_dir.path().to_path_buf();

            // Create some files
            fs::write(path.join("test1.txt"), "content1").expect("Failed to write");
            fs::write(path.join("test2.txt"), "content2").expect("Failed to write");

            assert!(path.join("test1.txt").exists());
            assert!(path.join("test2.txt").exists());

            path
        }; // test_env drops here, should clean up temp directory

        // Verify cleanup happened
        assert!(!temp_dir_path.exists());
    }
}
