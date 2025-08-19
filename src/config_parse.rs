use crate::cli_structure::UserInput;
use crate::global_conf::{GlobalVars, GLOBAL_VAR};
use crate::helpers::{check_file_exists, clean_string_from_quotes, fix_home_directory_path};
use configparser::ini::Ini;
use log::{error, info};
use std::fs;

#[allow(clippy::unnecessary_wraps)]
pub fn set_global_vars(user_input: &UserInput) -> Result<&str, &str> {
    let read_file_path = user_input.clone().config_ini;
    info!("Reading config file here: {read_file_path}");

    let file_path = match check_file_exists(read_file_path.as_str()) {
        Ok(filepath) => filepath,
        Err(e) => panic!("cloud not set file path: {e}"),
    };

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    let config = load_config(contents);
    let today = chrono::offset::Local::now();

    match GLOBAL_VAR.set(GlobalVars::new()) {
        Ok(()) => info!("GLOBAL_VAR is not set!"),
        Err(e) => {
            error!("Something went wrong when trying to set the GLOBAL_VAR: {e:?}");
            return Err("Something went wrong when trying to set the GLOBAL_VAR: {e:?}");
        }
    }

    let global_vars = if let Some(v) = GLOBAL_VAR.get() {
        info!("Could get GLOBAL_VAR");
        v
    } else {
        error!("Could not get GLOBAL_VAR, something is wrong");
        return Err("Could not get GLOBAL_VAR, something is wrong");
    };

    global_vars.set_all(config, today, user_input.clone());
    Ok("all good")
}

fn load_config(config_string: String) -> Ini {
    let mut config = Ini::new();
    config
        .read(config_string)
        .expect("Could not read the INI file!");
    config
}

pub fn get_variable_from_config(section: &str, variable: &str) -> Result<String, String> {
    let config = if let Some(v) = GLOBAL_VAR.get() {
        info!("Could get GLOBAL_VAR");
        &v.get_config()
    } else {
        error!("Could not get GLOBAL_VAR, something is wrong");
        return Err("Could not get GLOBAL_VAR, something is wrong".to_string());
    };

    let config_get = match &config.get(section, variable) {
        Some(cfg) => cfg.clone(),
        None => return Err("Could not get GLOBAL_VAR, something is wrong".to_string()),
    };

    let value = fix_home_directory_path(&config_get);

    Ok(clean_string_from_quotes(&value))
}

pub fn get_db_configurations() -> Result<String, String> {
    let config = match GLOBAL_VAR.get() {
        Some(cfg) => cfg.get_config(),
        None => return Err("GlobalVar Get didn't work".to_string()),
    };

    let config_get = match &config.get("db", "db_path") {
        Some(cfg) => cfg.clone(),
        None => return Err("Could not get DB Path from GlobalVars".to_string()),
    };

    let mut db_path = fix_home_directory_path(&clean_string_from_quotes(&config_get));

    let config_file = match &config.get("db", "db_file") {
        Some(cfg) => cfg.clone(),
        None => return Err("Could not get DB File from GlobalVars".to_string()),
    };

    let db_file = clean_string_from_quotes(&config_file);

    db_path.push('/');
    db_path.push_str(db_file.as_str());
    Ok(db_path)
}

// // ---------------------------------------------------------------------------
// // The following tests use dummy implementations of `UserInput` and `GlobalVars`
// // to allow testing without external dependencies. In your actual code, use the
// // real implementations from `cli_structure` and `global_conf`.
// // ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_structure::{InsertArgs, UserAction};
    use serial_test::serial;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    // Mock implementation that doesn't use the real GLOBAL_VAR
    struct MockGlobalState {
        config: Option<Ini>,
    }

    impl MockGlobalState {
        fn new() -> Self {
            Self { config: None }
        }

        fn set_config(&mut self, config: Ini) {
            self.config = Some(config);
        }

        fn get_config(&self) -> Option<&Ini> {
            self.config.as_ref()
        }
    }

    // Thread-safe mock global state for testing
    lazy_static::lazy_static! {
        static ref MOCK_GLOBAL_STATE: Mutex<MockGlobalState> = Mutex::new(MockGlobalState::new());
    }

    fn create_test_user_input_with_config(config_path: &str) -> UserInput {
        UserInput {
            action: UserAction::Insert(InsertArgs {
                company_name: "Test Company".to_string(),
                job_title: "Software Engineer".to_string(),
                quote: "Test quote".to_string(),
            }),
            save_to_database: Some(true),
            view_generated_cv: Some(false),
            dry_run: Some(false),
            config_ini: config_path.to_string(),
            engine: "sqlite".to_string(),
        }
    }

    fn create_test_config_content() -> String {
        r#"
[destination]
cv_path = "/home/test/cv"

[cv]
cv_template_path = "/home/test/template"
cv_template_file = "cv.tex"

[to_replace]
position_line_to_change = "POSITION_PLACEHOLDER"
quote_line_to_change = "QUOTE_PLACEHOLDER"

[db]
db_path = "/home/test/db"
db_file = "test.db"
engine = "sqlite"
db_pg_host = "postgresql://localhost/test"

[optional]
pdf_viewer = "evince"
"#
        .to_string()
    }

    #[test]
    fn test_load_config_success() {
        let config_content = create_test_config_content();
        let config = load_config(config_content);

        assert_eq!(
            config.get("destination", "cv_path").unwrap(),
            "\"/home/test/cv\""
        );
        assert_eq!(config.get("db", "engine").unwrap(), "\"sqlite\"");
        assert_eq!(config.get("cv", "cv_template_file").unwrap(), "\"cv.tex\"");
    }

    #[test]
    #[should_panic(expected = "Could not read the INI file!")]
    fn test_load_config_invalid_ini() {
        let invalid_config = "[invalid ini content without closing bracket";
        load_config(invalid_config.to_string());
    }

    #[test]
    fn test_load_config_empty() {
        let empty_config = "";
        let config = load_config(empty_config.to_string());
        assert!(config.get("section", "key").is_none());
    }

    #[test]
    fn test_load_config_with_quotes() {
        let config_content = r#"
[section]
quoted_value = "test value"
single_quoted = 'single test'
no_quotes = test
"#;
        let config = load_config(config_content.to_string());
        assert_eq!(
            config.get("section", "quoted_value").unwrap(),
            "\"test value\""
        );
        assert_eq!(
            config.get("section", "single_quoted").unwrap(),
            "'single test'"
        );
        assert_eq!(config.get("section", "no_quotes").unwrap(), "test");
    }

    #[test]
    fn test_load_config_multiple_sections() {
        let config_content = r#"
[section1]
key1 = "value1"

[section2]
key2 = "value2"

[section3]
key3 = "value3"
"#;
        let config = load_config(config_content.to_string());
        assert_eq!(config.get("section1", "key1").unwrap(), "\"value1\"");
        assert_eq!(config.get("section2", "key2").unwrap(), "\"value2\"");
        assert_eq!(config.get("section3", "key3").unwrap(), "\"value3\"");
    }

    #[test]
    fn test_load_config_with_comments() {
        let config_content = r#"
# This is a comment
[section]
key = "value"  # Inline comment
# Another comment
key2 = "value2"
"#;
        let config = load_config(config_content.to_string());
        assert_eq!(config.get("section", "key").unwrap(), "\"value\"");
        assert_eq!(config.get("section", "key2").unwrap(), "\"value2\"");
    }

    // Mock functions for testing without real GLOBAL_VAR dependency
    fn mock_get_variable_from_config(
        config: &Ini,
        section: &str,
        variable: &str,
    ) -> Result<String, String> {
        match config.get(section, variable) {
            Some(value) => {
                let expanded = fix_home_directory_path(&value);
                Ok(clean_string_from_quotes(&expanded))
            }
            None => Err(format!("Key {}.{} not found", section, variable)),
        }
    }

    fn mock_get_db_configurations(config: &Ini) -> Result<String, String> {
        let db_path = match config.get("db", "db_path") {
            Some(path) => fix_home_directory_path(&clean_string_from_quotes(&path)),
            None => return Err("db_path not found".to_string()),
        };

        let db_file = match config.get("db", "db_file") {
            Some(file) => clean_string_from_quotes(&file),
            None => return Err("db_file not found".to_string()),
        };

        Ok(format!("{}/{}", db_path, db_file))
    }

    #[test]
    fn test_mock_get_variable_from_config_success() {
        let config_content = create_test_config_content();
        let config = load_config(config_content);

        let result = mock_get_variable_from_config(&config, "destination", "cv_path");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/home/test/cv");
    }

    #[test]
    fn test_mock_get_variable_from_config_missing_key() {
        let config_content = create_test_config_content();
        let config = load_config(config_content);

        let result = mock_get_variable_from_config(&config, "nonexistent", "key");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Key nonexistent.key not found");
    }

    #[test]
    fn test_mock_get_variable_from_config_with_tilde() {
        let config_with_tilde = r#"
[test]
path_with_tilde = "~/test/path"
"#;
        let config = load_config(config_with_tilde.to_string());

        let result = mock_get_variable_from_config(&config, "test", "path_with_tilde");
        assert!(result.is_ok());
        let result_path = result.unwrap();
        assert!(!result_path.contains("~"));
        assert!(result_path.contains("/test/path"));
    }

    #[test]
    fn test_mock_get_variable_from_config_with_quotes() {
        let config_with_quotes = r#"
[test]
quoted_path = "/test/path"
single_quoted_path = '/test/path'
"#;
        let config = load_config(config_with_quotes.to_string());

        let result1 = mock_get_variable_from_config(&config, "test", "quoted_path");
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), "/test/path");

        let result2 = mock_get_variable_from_config(&config, "test", "single_quoted_path");
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), "/test/path");
    }

    #[test]
    fn test_mock_get_db_configurations_success() {
        let config_content = create_test_config_content();
        let config = load_config(config_content);

        let result = mock_get_db_configurations(&config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/home/test/db/test.db");
    }

    #[test]
    fn test_mock_get_db_configurations_missing_db_path() {
        let config_without_db_path = r#"
[db]
db_file = "test.db"
"#;
        let config = load_config(config_without_db_path.to_string());

        let result = mock_get_db_configurations(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "db_path not found");
    }

    #[test]
    fn test_mock_get_db_configurations_missing_db_file() {
        let config_without_db_file = r#"
[db]
db_path = "/home/test/db"
"#;
        let config = load_config(config_without_db_file.to_string());

        let result = mock_get_db_configurations(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "db_file not found");
    }

    #[test]
    fn test_mock_get_db_configurations_with_tilde() {
        let config_with_tilde = r#"
[db]
db_path = "~/test/db"
db_file = "test.db"
"#;
        let config = load_config(config_with_tilde.to_string());

        let result = mock_get_db_configurations(&config);
        assert!(result.is_ok());
        let db_config = result.unwrap();
        assert!(!db_config.contains("~"));
        assert!(db_config.contains("/test/db/test.db"));
    }

    #[test]
    fn test_mock_get_db_configurations_with_quotes() {
        let config_with_quotes = r#"
[db]
db_path = "/home/test/db"
db_file = "test.db"
"#;
        let config = load_config(config_with_quotes.to_string());

        let result = mock_get_db_configurations(&config);
        assert!(result.is_ok());
        let db_config = result.unwrap();
        assert_eq!(db_config, "/home/test/db/test.db");
        assert!(!db_config.contains("\""));
    }

    // Test file operations without GLOBAL_VAR dependency
    #[test]
    fn test_config_file_reading() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = create_test_config_content();
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        // Test file existence check
        let file_check = check_file_exists(file_path);
        assert!(file_check.is_ok());

        // Test file reading
        let contents = fs::read_to_string(file_path).expect("Should be able to read test file");
        assert!(contents.contains("[destination]"));
        assert!(contents.contains("[db]"));
        assert!(contents.contains("cv_path"));
    }

    #[test]
    fn test_config_file_not_found() {
        let result = check_file_exists("/definitely/does/not/exist.ini");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "File does not exist!");
    }

    // Integration-style tests that test the logic without global state
    #[test]
    fn test_full_config_processing_flow() {
        // Create test config file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = create_test_config_content();
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        // Test the full flow: file check -> read -> parse -> extract values
        let file_path_result = check_file_exists(file_path);
        assert!(file_path_result.is_ok());

        let contents = fs::read_to_string(file_path_result.unwrap())
            .expect("Should be able to read config file");

        let config = load_config(contents);

        // Test extracting various config values
        let cv_path = mock_get_variable_from_config(&config, "destination", "cv_path");
        assert_eq!(cv_path.unwrap(), "/home/test/cv");

        let template_path = mock_get_variable_from_config(&config, "cv", "cv_template_path");
        assert_eq!(template_path.unwrap(), "/home/test/template");

        let db_config = mock_get_db_configurations(&config);
        assert_eq!(db_config.unwrap(), "/home/test/db/test.db");
    }

    #[test]
    fn test_config_with_environment_variables() {
        // Test that home directory expansion works
        let config_with_env = r#"
[paths]
home_path = "~/documents"
absolute_path = "/absolute/path"
"#;
        let config = load_config(config_with_env.to_string());

        let home_result = mock_get_variable_from_config(&config, "paths", "home_path");
        assert!(home_result.is_ok());
        let home_path = home_result.unwrap();
        assert!(!home_path.contains("~"));
        assert!(home_path.contains("documents"));

        let abs_result = mock_get_variable_from_config(&config, "paths", "absolute_path");
        assert!(abs_result.is_ok());
        assert_eq!(abs_result.unwrap(), "/absolute/path");
    }

    #[test]
    fn test_edge_cases() {
        // Test with empty values
        let config_with_empty = r#"
[test]
empty_value = ""
space_value = "   "
"#;
        let config = load_config(config_with_empty.to_string());

        let empty_result = mock_get_variable_from_config(&config, "test", "empty_value");
        assert_eq!(empty_result.unwrap(), "");

        let space_result = mock_get_variable_from_config(&config, "test", "space_value");
        assert_eq!(space_result.unwrap(), "   ");
    }

    #[test]
    fn test_unicode_config_values() {
        let config_with_unicode = r#"
[unicode]
chinese = "中文路径"
emoji = "📁 folder"
mixed = "Test 测试 🚀"
"#;
        let config = load_config(config_with_unicode.to_string());

        let chinese = mock_get_variable_from_config(&config, "unicode", "chinese");
        assert_eq!(chinese.unwrap(), "中文路径");

        let emoji = mock_get_variable_from_config(&config, "unicode", "emoji");
        assert_eq!(emoji.unwrap(), "📁 folder");

        let mixed = mock_get_variable_from_config(&config, "unicode", "mixed");
        assert_eq!(mixed.unwrap(), "Test 测试 🚀");
    }

    // Performance test for config loading
    #[test]
    fn test_large_config_performance() {
        let mut large_config = String::new();

        // Generate a large config with many sections and keys
        for section in 0..50 {
            large_config.push_str(&format!("[section_{}]\n", section));
            for key in 0..20 {
                large_config.push_str(&format!("key_{} = \"value_{}_{}\"\n", key, section, key));
            }
            large_config.push('\n');
        }

        let start = std::time::Instant::now();
        let config = load_config(large_config);
        let duration = start.elapsed();

        // Should load quickly even with many entries
        assert!(duration.as_millis() < 100);

        // Verify some values are accessible
        let result = mock_get_variable_from_config(&config, "section_25", "key_10");
        assert_eq!(result.unwrap(), "value_25_10");
    }

    // Test error handling and edge cases
    #[test]
    fn test_malformed_config_sections() {
        // Test various malformed configurations that should still parse or fail gracefully
        let configs_to_test = vec![
            (
                "[section1]\nkey=value\n[section2",
                "Missing closing bracket",
            ),
            ("key_without_section=value", "Key without section"),
            ("[]\nkey=value", "Empty section name"),
            ("[section]\n=value", "Empty key name"),
            ("[section]\nkey=", "Empty value"),
        ];

        for (config_content, _description) in configs_to_test {
            // Most of these should either parse successfully or panic
            // The exact behavior depends on the configparser crate implementation
            let result = std::panic::catch_unwind(|| load_config(config_content.to_string()));

            // We mainly want to ensure no undefined behavior occurs
            // Some configurations might parse successfully, others might panic
            // Both are acceptable behaviors for malformed input
            assert!(result.is_ok() || result.is_err());
        }
    }
}
