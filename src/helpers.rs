use std::fs;
use crate::config_parse;

// HELPERS
pub fn clean_string_from_quotes(cv_template_path: &str) -> String {
    cv_template_path.replace(['\"', '\''], "")
}

pub fn check_file_exists(file_path: &str) -> String {
    let fixed_file_path = if file_path.starts_with('~') {
        let home_dir = dirs::home_dir().unwrap();
        file_path.replace('~', home_dir.to_str().unwrap())
    } else {
        file_path.to_string()
    };

    // TODO if db file does not exist, create it
    // if fs::metadata(file_path).is_err() {
    //     panic!("File {} does not exist", file_path)
    // };
    assert!(fs::metadata(fixed_file_path.clone()).is_ok(), "File {file_path} does not exist");
    fixed_file_path
}

pub fn check_if_db_env_is_set_or_set_from_config() {
    let db_path = config_parse::get_db_configurations();

    if let Ok(val) = std::env::var("DATABASE_URL") { drop(val); } else {
        std::env::set_var("DATABASE_URL", format!("sqlite://{db_path}"));
    }
}

pub fn read_destination_cv_file(cv_path: &str) -> Vec<u8> {

    let cv_file_content: Vec<u8> = fs::read(cv_path.replace(".tex", ".pdf"))
        .expect("Could not read the file");
    cv_file_content
}

pub fn view_cv_file(cv_path: &str) {
    
    let cv_file_content = read_destination_cv_file(cv_path);
    let cv_file_content = std::str::from_utf8(&cv_file_content).unwrap();
    println!("{}", cv_file_content);
}