use std::fs;

// HELPERS
pub fn clean_string_from_quotes(cv_template_path: &str) -> String {
    cv_template_path.replace(['\"', '\''], "")
}

pub fn check_file_exists(file_path: &str) {
    // TODO if db file does not exist, create it
    // if fs::metadata(file_path).is_err() {
    //     panic!("File {} does not exist", file_path)
    // };
    assert!(fs::metadata(file_path).is_ok(), "File {file_path} does not exist");
}
