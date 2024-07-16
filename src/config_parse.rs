use log::info;
use configparser::ini::Ini;
use std::fs;
use once_cell::sync::OnceCell;
use crate::helpers;

pub static CONFIG: OnceCell<Ini> = OnceCell::new();

pub fn get_config_once_cell() -> Ini {
    CONFIG.get().unwrap().clone()
}

pub fn read_config_file(read_file_path: &str) -> Ini {
    info!("Reading config file here: {read_file_path}");
    let file_path = helpers::fix_home_directory_path(&crate::helpers::check_file_exists(read_file_path));

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    load_config(contents)
}

fn load_config(config_string: String) -> Ini {
    let mut config = Ini::new();
    config.read(config_string).unwrap();
    config
}

pub fn get_variable_from_config(section: &str, variable: &str) -> String {
    let config = get_config_once_cell();
    let value = helpers::fix_home_directory_path(&config.get(section, variable).unwrap().clone());
    helpers::clean_string_from_quotes(&value)
}

pub fn get_db_configurations() -> String {
    let config = get_config_once_cell();

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

