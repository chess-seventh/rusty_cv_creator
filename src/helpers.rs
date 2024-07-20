use chrono::NaiveDateTime;
use log::info;
use log::error;
use std::fs;
use std::process::{Command, Stdio};
use crate::config_parse;
use crate::helpers;


pub fn clean_string_from_quotes(cv_template_path: &str) -> String {
    cv_template_path.replace(['\"', '\''], "")
}

pub fn fix_home_directory_path(file_path: &str) -> String {
    if file_path.contains('~') {
        let home_dir = dirs::home_dir().unwrap();
        file_path.replace('~', home_dir.to_str().unwrap())
    } else {
        file_path.to_string()
    }
}

pub fn check_file_exists(file_path: &str) -> String {
    let fixed_file_path= fix_home_directory_path(file_path);

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
    let cv_file_content: Vec<u8> = fs::read(helpers::fix_home_directory_path(cv_path).replace(".tex", ".pdf"))
        .expect("Could not read the file");
    cv_file_content
}

pub fn view_cv_file(cv_path: &str) {
    let file_name = crate::config_parse::get_variable_from_config("cv", "cv_template_file").to_string();
    let cv_dir = cv_path.to_string().replace(&file_name, "");
    let pdf_viewer = config_parse::get_variable_from_config("optional", "pdf_viewer");
    let pdf_file = cv_path.replace(".tex", ".pdf");

    match Command::new(pdf_viewer)
        .current_dir(cv_dir)
        .stdout(Stdio::null())
        .arg(pdf_file)
        .spawn() {
        Ok(_) => info!("CV compiled successfully"),
        Err(e) => error!("Error compiling CV: {e}")
    }
}

pub fn parse_date(filter_date: Option<String>) -> NaiveDateTime {
    // TODO make sure that we parse all possibilities, else return to use the proper date formats
    // https://docs.rs/chrono/latest/chrono/format/strftime/index.html
    //
    match chrono::NaiveDateTime::parse_from_str(&filter_date.unwrap(), "%B") {
        Ok(d) => d,
        Err(_) => todo!(),
    }

}
