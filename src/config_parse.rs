use crate::cli_structure::UserInput;
use crate::global_conf::{GlobalVars, GLOBAL_VAR};
use crate::helpers::{check_file_exists, clean_string_from_quotes, fix_home_directory_path};
use configparser::ini::Ini;
use log::info;
use std::fs;

pub fn set_global_vars(user_input: &UserInput) {
    let read_file_path = user_input.clone().config_ini;
    info!("Reading config file here: {read_file_path}");

    let file_path = &check_file_exists(read_file_path.as_str());

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
    let config = load_config(contents);
    let today = chrono::offset::Local::now();

    GLOBAL_VAR.set(GlobalVars::new()).unwrap();

    let global_vars = GLOBAL_VAR.get().unwrap();

    global_vars.set_all(config, today, user_input.clone());
}

fn load_config(config_string: String) -> Ini {
    let mut config = Ini::new();
    config.read(config_string).unwrap();
    config
}

pub fn get_variable_from_config(section: &str, variable: &str) -> String {
    let config = GLOBAL_VAR.get().unwrap().get_config();
    let value = fix_home_directory_path(&config.get(section, variable).unwrap().clone());
    clean_string_from_quotes(&value)
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
