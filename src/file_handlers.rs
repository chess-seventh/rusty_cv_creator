use log::{info,error};
use chrono::{DateTime, Local};
use std::io::Error;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use crate::global_conf::GlobalVars;
use crate::helpers::{fix_home_directory_path, clean_string_from_quotes};
use crate::config_parse::get_variable_from_config;


fn check_dir_exists(dir: &str) -> bool {
    Path::new(dir).try_exists().expect("Dir does not exist")
}

fn check_file_exists(dir: &str, file: &str) -> bool {
    let full_path = format!("{dir}/{file}");
    Path::new(&full_path).is_file()
}

pub fn compile_cv(cv_dir: &str, cv_file: &str) {
    info!("CV_DIR: {cv_dir}");
    info!("CV_FILE: {cv_file}");

    if check_dir_exists(cv_dir) { info!("✅ Directory exists") } else {
            error!("Directory does not exist");
            panic!("Directory does not exist");
    };

    if check_file_exists(cv_dir, cv_file) { info!("✅ File exists") } else {
            error!("File does not exist");
            panic!("File does not exist");
    };

    match Command::new("xelatex")
        .arg(cv_file)
        .current_dir(cv_dir)
        .stdout(Stdio::null())
        .spawn() {
        Ok(_) => { 
            info!("✅ CV compiled successfully");
        },
        Err(e) => {
            error!("XELATEX Error compiling CV: {e}");
            panic!("XELATEX Error compiling CV: {e}");
        }
    }
}

pub fn make_cv_changes_based_on_input(job_title: &str, quote: &str, cv_file_path: &str) {
    let cv_file_content = read_destination_cv_file(cv_file_path);
    let changed_content = change_values_in_destination_cv(&cv_file_content, job_title, quote);
    match write_to_destination_cv_file(cv_file_path, &changed_content) {
        Ok(()) => info!("✅ CV file updated successfully"),
        Err(e) => {
            error!("Error updating CV file: {e}");
            panic!("Error updating CV file: {e}");
        }
    }
}

pub fn create_directory(job_title: &str, company_name: &str) -> String {
    let destination_folder = fix_home_directory_path(&get_variable_from_config("destination", "cv_path"));
    let now = GlobalVars::get_today();

    match prepare_year_dir(&destination_folder, now) {
        Ok(y) => info!("✅ Year directory created successfully: {y:}"),
        Err(e) => panic!("Error creating year directory: {e}")
    }

    let (cv_template_path, full_destination_path) = prepare_path_for_new_cv(job_title, company_name, &destination_folder, now);

    match copy_dir::copy_dir(cv_template_path, full_destination_path.clone()) {
        Ok(_) => info!("✅ Directory created & copied successfully"),
        Err(e) => {
            error!("Error copying directory: {e}");
            panic!("Error copying directory: {e}");
        }
    }
    full_destination_path
}

pub fn remove_cv_dir(path_to_remove: &Path) -> std::io::Result<()> {
    fs::remove_dir_all(path_to_remove)?;
    Ok(())
}

fn prepare_path_for_new_cv(job_title: &str, company_name: &str, destination_folder: &str, now: DateTime<Local>) -> (String, String) {
    let cv_template_path: String = fix_home_directory_path(&get_variable_from_config("cv", "cv_template_path"));

    let formatted_job_title = job_title.replace(' ', "-");
    let formatted_company_name = company_name.replace(' ', "-");


    let date_dir = now.format("%Y/%Y-%m-%d").to_string();
    let full_destination_path = fix_home_directory_path(&format!("{destination_folder}/{date_dir}_{formatted_company_name}_{formatted_job_title}"));

    info!("✅ Creating directory: {full_destination_path}");
    info!("✅ Copying from: {}", cv_template_path.clone());

    (cv_template_path, full_destination_path)
}

fn prepare_year_dir(destination_folder: &String, now: DateTime<Local>) -> Result<String, Error> {
    let year_full_dir = format!("{}/{}", destination_folder, now.format("%Y"));
    fs::create_dir_all(year_full_dir.clone())?;
    Ok(clean_string_from_quotes(&year_full_dir.clone()))
}

fn write_to_destination_cv_file(cv_file_path: &str, content: &String) -> std::io::Result<()> {
    let fix_cv_file_path = fix_home_directory_path(cv_file_path);
    fs::write(fix_cv_file_path, content)?;
    Ok(())
}

fn read_destination_cv_file(destination_cv_file: &str) -> String {
    let fix_destination_cv_file = fix_home_directory_path(destination_cv_file);
    info!("✅ Reading CV file: {fix_destination_cv_file}");
    fs::read_to_string(fix_destination_cv_file)
        .expect("Should have been able to read the file")
}

fn change_values_in_destination_cv(cv_file_content: &str, job_title: &str, _quote: &str) -> String {
    change_position_in_destination_cv(cv_file_content, job_title)
    // modified_cv_content = change_quote_in_destination_cv(&modified_cv_content, quote);
    // modified_cv_content
}

fn change_position_in_destination_cv(cv_file_content: &str, job_title: &str) -> String {
    let replace_position = get_variable_from_config("to_replace", "position_line_to_change");

    info!("✅ Changed position from: {replace_position} to: {job_title}");

    let new = cv_file_content.replace(replace_position.as_str(), job_title);

    assert!((new != cv_file_content), "Didn't change shit");

    new

}

fn _change_quote_in_destination_cv(cv_file_content: &str, quote: &str) -> String {
    let replace_quote = get_variable_from_config("to_replace", "quote_line_to_change");
    if quote.is_empty() {
        info!("✅ Removing quote");

        return cv_file_content.lines()
            .filter(|&line| !line.contains(&replace_quote))
            .collect::<Vec<_>>()
            .join("\n");
    }

    info!("✅ Changed quote to: {quote:?}");
    cv_file_content.replace(replace_quote.as_str(), quote)
}

