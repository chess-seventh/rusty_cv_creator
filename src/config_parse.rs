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

    // TODO:
    // remove unwrap
    let value = fix_home_directory_path(&config.get(section, variable).unwrap().clone());

    Ok(clean_string_from_quotes(&value))
}

pub fn get_db_configurations() -> String {
    let config = GLOBAL_VAR.get().unwrap().get_config();

    let mut db_path = fix_home_directory_path(&clean_string_from_quotes(
        &config.get("db", "db_path").unwrap().clone(),
    ));

    let db_file = clean_string_from_quotes(&config.get("db", "db_file").unwrap());

    db_path.push('/');
    db_path.push_str(db_file.as_str());
    db_path
}
// ---------------------------------------------------------------------------
// The following tests use dummy implementations of `UserInput` and `GlobalVars`
// to allow testing without external dependencies. In your actual code, use the
// real implementations from `cli_structure` and `global_conf`.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use configparser::ini::Ini;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Dummy UserInput for testing purposes.
    #[derive(Clone, Default)]
    struct DummyUserInput {
        pub config_ini: String,
        // Other fields as needed.
    }
    impl From<DummyUserInput> for UserInput {
        fn from(input: DummyUserInput) -> Self {
            // In real code, fill in all the fields appropriately.
            UserInput {
                config_ini: input.config_ini,
                ..Default::default()
            }
        }
    }
    type UserInput = DummyUserInput;

    // Dummy GlobalVars for testing purposes.
    #[derive(Default, Clone)]
    pub struct TestGlobalVars {
        config: Ini,
    }
    impl TestGlobalVars {
        pub fn new() -> Self {
            Self { config: Ini::new() }
        }
        pub fn set_all(
            &mut self,
            config: Ini,
            _today: chrono::DateTime<Local>,
            _user_input: UserInput,
        ) {
            self.config = config;
        }
        pub fn get_config(&self) -> &Ini {
            &self.config
        }
    }
    // For testing, we alias GlobalVars to our dummy implementation.
    type GlobalVars = TestGlobalVars;

    #[test]
    fn test_load_config_success() {
        let config_str = r#"
            [section]
            key = "value"
        "#;
        let ini = load_config(config_str).expect("Failed to load config");
        let value = ini.get("section", "key").expect("Key not found");
        // Assuming that `clean_string_from_quotes` strips the quotes.
        assert_eq!(clean_string_from_quotes(&value), "value");
    }

    #[test]
    fn test_get_variable_from_config_success() {
        let mut ini = Ini::new();
        ini.set("test", "var", Some("/home/user".to_string()));
        let mut global_vars = GlobalVars::new();
        global_vars.set_all(ini, Local::now(), UserInput::default());

        let var =
            get_variable_from_config(&global_vars, "test", "var").expect("Failed to get variable");
        // Assuming that the helper functions return the input unchanged in tests.
        assert_eq!(var, "/home/user");
    }

    #[test]
    fn test_get_db_configurations_success() {
        let mut ini = Ini::new();
        ini.set("db", "db_path", Some("/home/user".to_string()));
        ini.set("db", "db_file", Some("database.sqlite".to_string()));
        let mut global_vars = GlobalVars::new();
        global_vars.set_all(ini, Local::now(), UserInput::default());

        let db_config =
            get_db_configurations(&global_vars).expect("Failed to get db configurations");
        // Expected: "/home/user/database.sqlite"
        assert_eq!(db_config, "/home/user/database.sqlite");
    }

    #[test]
    fn test_load_global_vars_success() {
        // Create a temporary configuration file.
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = r#"
            [section]
            key = "value"
        "#;
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap().to_string();

        let user_input = UserInput {
            config_ini: file_path,
            ..Default::default()
        };

        // For testing we assume `check_file_exists` returns the given path.
        let global_vars = load_global_vars(&user_input).expect("Failed to load global vars");
        let value = get_variable_from_config(&global_vars, "section", "key")
            .expect("Failed to get variable");
        assert_eq!(value, "value");
    }
}
