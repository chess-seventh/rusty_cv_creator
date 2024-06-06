use configparser::ini::Ini;
use std::fs;
use once_cell::sync::OnceCell;
use crate::helpers;

pub static CONFIG: OnceCell<Ini> = OnceCell::new();

pub fn read_config_file(read_file_path: &str) -> Ini {
    println!("Reading config file here: {read_file_path}");
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

pub fn get_cv_template_directory() -> String {
    let config = crate::CONFIG.get().unwrap().clone();

    let cv_template_path: String = helpers::fix_home_directory_path(&config.get("cv", "cv_template_path").unwrap().clone());
    helpers::clean_string_from_quotes(&cv_template_path)
}

pub fn get_destination_folder() -> String {
    let config = crate::CONFIG.get().unwrap().clone();

    let destination_folder = helpers::fix_home_directory_path(&config.get("destination", "cv_path").unwrap());
    helpers::clean_string_from_quotes(&destination_folder.clone())
}

pub fn get_cv_template_file() -> String {
    let config = CONFIG.get().unwrap().clone();
    let file = config.get("cv", "cv_template_file").unwrap().clone();
    helpers::clean_string_from_quotes(&file)
}

pub fn get_position_value_to_change() -> String {
    let config = CONFIG.get().unwrap().clone();
    let mut replace_position: String = config.get("to_replace", "position_line_to_change").unwrap().clone();
    replace_position = helpers::clean_string_from_quotes(&replace_position.clone());
    replace_position
}

pub fn get_quote_value_to_change() -> String {
    let config = CONFIG.get().unwrap().clone();
    let mut replace_quote: String = config.get("to_replace", "quote_line_to_change").unwrap().clone();
    replace_quote = helpers::clean_string_from_quotes(&replace_quote.clone());
    replace_quote
}

pub fn get_db_configurations() -> String {
        let config = crate::CONFIG.get().unwrap().clone();

        let mut db_path = config.get("db", "db_path").unwrap().clone();
        let db_file = config.get("db", "db_file").unwrap();
        let file: &str = db_file.as_str();

        db_path.push('/');
        db_path.push_str(file);
        helpers::clean_string_from_quotes(&db_path)
}

pub fn get_optional_configurations() -> String {
        let config = crate::CONFIG.get().unwrap().clone();

        let pdf_viewer = config.get("optional", "pdf_viewer").unwrap().clone();

        helpers::clean_string_from_quotes(&pdf_viewer)
}
