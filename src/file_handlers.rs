use chrono::{DateTime, Local};
use std::io::Error;
use std::fs;
use std::process::{Command, Stdio};
use crate::config_parse;

pub fn create_directory(job_title: &str, company_name: &str) -> String {
    let destination_folder = config_parse::get_destination_folder();
    let now = chrono::offset::Local::now();

    match prepare_year_dir(&destination_folder, now) {
        Ok(y) => println!("Year directory created successfully: {y:}"),
        Err(e) => panic!("Error creating year directory: {e}")
    }

    let (cv_template_path, full_destination_path) = prepare_path_for_new_cv(job_title, company_name, &destination_folder, now);

    match copy_dir::copy_dir(cv_template_path, full_destination_path.clone()) {
        Ok(_) => println!("Directory created & copied successfully"),
        Err(e) => println!("Error creating directory: {e}")
    }
    full_destination_path
}

fn prepare_path_for_new_cv(job_title: &str, company_name: &str, destination_folder: &str, now: DateTime<Local>) -> (String, String) {
    let config = crate::CONFIG.get().unwrap().clone();

    let mut cv_template_path: String = config.get("cv", "cv_template_path").unwrap().clone();
    cv_template_path = config_parse::clean_string_from_quotes(&cv_template_path.clone());
    cv_template_path.push('/');

    let formatted_job_title = job_title.replace(' ', "-");
    let formatted_company_name = company_name.replace(' ', "-");


    let date_dir = now.format("%Y/%Y-%m-%d").to_string();
    let full_destination_path = format!("{destination_folder}/{date_dir}_{formatted_company_name}_{formatted_job_title}");

    println!("Creating directory: {full_destination_path}");
    println!("Copying from: {}", cv_template_path.clone());

    (cv_template_path, full_destination_path)
}

fn prepare_year_dir(destination_folder: &String, now: DateTime<Local>) -> Result<String, Error> {
    let year_full_dir = format!("{}/{}", destination_folder, now.format("%Y"));
    fs::create_dir_all(year_full_dir.clone())?;
    Ok(config_parse::clean_string_from_quotes(&year_full_dir.clone()))
}

pub fn compile_cv(cv_dir: &str, cv_file: &str) {
    match Command::new("xelatex")
        .current_dir(cv_dir)
        .stdout(Stdio::null())
        .arg(cv_file)
        .spawn() {
        Ok(_) => println!("CV compiled successfully"),
        Err(e) => println!("Error compiling CV: {e}")
    }
}
