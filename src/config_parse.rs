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
