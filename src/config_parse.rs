use configparser::ini::Ini;
use std::fs;
use once_cell::sync::OnceCell;
pub static CONFIG: OnceCell<Ini> = OnceCell::new();

pub fn read_config_file(file_path: &str) -> Ini {
    crate::check_file_exists(file_path);

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    load_config(contents)
}

fn load_config(config_string: String) -> Ini {
    let mut config = Ini::new();
    config.read(config_string).unwrap();
    config
}

pub fn get_configurations(save_to_database: bool) -> (String, Option<String>) {
    let cv_path: String = get_cv_template_directory();
    crate::check_file_exists(&cv_path);

    if save_to_database {
        let db_path: String = get_db_configurations();
        crate::check_file_exists(&db_path);
        return (cv_path, Some(db_path));
    }

    (cv_path, None)
}

fn get_db_configurations() -> String {
    let config = crate::CONFIG.get().unwrap().clone();

    let mut db_path = config.get("db", "db_path").unwrap().clone();
    let db_file = config.get("db", "db_file").unwrap();
    let file: &str = db_file.as_str();

    db_path.push('/');
    db_path.push_str(file);
    clean_string_from_quotes(&db_path)
}

fn get_cv_template_directory() -> String {
    let config = crate::CONFIG.get().unwrap().clone();

    let mut cv_template_path: String = config.get("cv", "cv_template_path").unwrap().clone();

    let cv_template_file = config.get("cv", "cv_template_file").unwrap();
    let file: &str = cv_template_file.as_str();

    cv_template_path.push('/');
    cv_template_path.push_str(file);
    clean_string_from_quotes(&cv_template_path)
}

pub fn get_destination_folder() -> String {
    let config = crate::CONFIG.get().unwrap().clone();

    let destination_folder = config.get("destination", "cv_path").unwrap();
    clean_string_from_quotes(&destination_folder.clone())
}


// HELPERS
pub fn clean_string_from_quotes(cv_template_path: &str) -> String {
    cv_template_path.replace(['\"', '\''], "")
}
