use crate::cli_structure::UserInput;
use crate::global_conf::{
    get_global_var, get_global_var_config_db_file, get_global_var_config_db_path, GlobalVars,
    GLOBAL_VAR,
};
use crate::helpers::{check_config_file_exists, clean_string_from_quotes, fix_home_directory_path};
use configparser::ini::Ini;
use log::{debug, info};
use std::fs;

#[allow(clippy::unnecessary_wraps)]
pub fn set_global_vars(user_input: &UserInput) {
    let read_file_path = user_input.clone().config_ini;
    info!("Reading config file here: {read_file_path:}");

    let file_path = match check_config_file_exists(read_file_path.as_str()) {
        Ok(filepath) => filepath,
        Err(e) => panic!("Error in checking that the file path exists: {e:}"),
    };

    let contents = fs::read_to_string(file_path.clone())
        .unwrap_or_else(|_| panic!("Should have been able to read the file: {file_path:}"));

    let config = load_config(contents);
    let today = chrono::offset::Local::now();

    match GLOBAL_VAR.set(GlobalVars::new()) {
        Ok(()) => info!("GLOBAL_VAR is not set, setting OnceCell now..."),
        Err(e) => {
            panic!("Something went wrong when trying to create a new GLOBAL_VAR: {e:?}");
        }
    }

    let Some(global_vars) = GLOBAL_VAR.get() else {
        panic!("Could not get GLOBAL_VAR, something is wrong");
    };

    global_vars.set_all(config.clone(), today, user_input.clone());
}

fn load_config(config_string: String) -> Ini {
    info!("Reading the config file");
    let mut config = Ini::new();
    config
        .read(config_string)
        .expect("Could not read the INI file!");
    config
}

pub fn get_variable_from_config_file(section: &str, variable: &str) -> Result<String, String> {
    debug!("Retrieving from config: {section:} {variable:}");
    let config_get = get_global_var().get_user_input_vars(section, variable)?;

    let value = fix_home_directory_path(&config_get);

    Ok(clean_string_from_quotes(&value))
}

pub fn get_db_configurations() -> Result<String, String> {
    debug!("Getting DB Configuration");
    let cfg_db_path = get_global_var_config_db_path()?;
    let cfg_db_file = get_global_var_config_db_file()?;

    let mut db_path = fix_home_directory_path(&clean_string_from_quotes(&cfg_db_path));
    let db_file = clean_string_from_quotes(&cfg_db_file);

    db_path.push('/');
    db_path.push_str(db_file.as_str());
    Ok(db_path)
}
