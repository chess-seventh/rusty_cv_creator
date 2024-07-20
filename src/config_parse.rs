use log::info;
use configparser::ini::Ini;
use std::fs;
use once_cell::sync::OnceCell;
use chrono::{DateTime, Local};
use crate::helpers;

pub static CONFIG: OnceCell<Ini> = OnceCell::new();
pub static TODAY: OnceCell<DateTime<Local>> = OnceCell::new();

pub struct GlobalVars { }


impl GlobalVars {
    pub fn get_config() -> Ini {
        CONFIG.get().expect("Config.ini file not initialized").clone()
    }

    pub fn get_today() -> DateTime<Local> {
        *TODAY.get().expect("Date not initialized")
    }
}


pub fn set_global_vars(read_file_path: &str) {
    info!("Reading config file here: {read_file_path}");
    let file_path = helpers::fix_home_directory_path(&crate::helpers::check_file_exists(read_file_path));
    let now = chrono::offset::Local::now();

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    // load_config(contents)
    CONFIG.set(load_config(contents)).unwrap();
    TODAY.set(now).unwrap();
}

fn load_config(config_string: String) -> Ini {
    let mut config = Ini::new();
    config.read(config_string).unwrap();
    config
}

pub fn get_variable_from_config(section: &str, variable: &str) -> String {
    let config = GlobalVars::get_config();
    let value = helpers::fix_home_directory_path(&config.get(section, variable).unwrap().clone());
    helpers::clean_string_from_quotes(&value)
}

pub fn get_db_configurations() -> String {
    let config = GlobalVars::get_config();

    let mut db_path = helpers::fix_home_directory_path(
        &helpers::clean_string_from_quotes(
            &config.get("db", "db_path").unwrap().clone()
        )
    );

    let db_file = helpers::clean_string_from_quotes(&config.get("db", "db_file").unwrap());

    db_path.push('/');
    db_path.push_str(db_file.as_str());
    db_path
}

