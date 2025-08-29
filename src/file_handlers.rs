use crate::config_parse::get_variable_from_config_file;
use crate::global_conf::GLOBAL_VAR;
use crate::helpers::{clean_string_from_quotes, fix_home_directory_path};
use chrono::{DateTime, Local};
use log::{error, info};
use std::fs;
use std::io::Error;
use std::path::Path;
use std::process::{Command, Stdio};

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

    if check_dir_exists(cv_dir) {
        info!("✅ Directory exists");
    } else {
        error!("Directory does not exist");
        panic!("Directory does not exist");
    }

    if check_file_exists(cv_dir, cv_file) {
        info!("✅ File exists");
    } else {
        error!("File does not exist");
        panic!("File does not exist");
    }

    // TODO: Make sure to have the destination CV directory where to output the pdf, out, log, aux.
    let cmd_output = Command::new("xelatex")
        .arg("-output-directory=/home/seventh/Documents/CV_testing/outputs/")
        .arg("--file-line-error")
        .arg("--interaction=nonstopmode")
        .arg(cv_file)
        .current_dir(cv_dir)
        // .stdout(Stdio::null())
        .status()
        .expect("Failed to run XELATEX");

    if cmd_output.success() {
        info!("✅ CV compiled successfully");
    } else {
        error!("XELATEX Error compiling CV");
        panic!("XELATEX Error compiling CV");
    }
}

pub fn make_cv_changes_based_on_input(
    job_title: &str,
    quote: &str,
    cv_file_path: &str,
) -> Result<String, String> {
    let cv_file_content = read_destination_cv_file(cv_file_path);
    let changed_content = change_values_in_destination_cv(&cv_file_content, job_title, quote)?;
    match write_to_destination_cv_file(cv_file_path, &changed_content) {
        Ok(()) => {
            info!("✅ CV file updated successfully");
            Ok("✅ CV file updated successfully".to_string())
        }
        Err(e) => {
            error!("Error updating CV file: {e:}");
            panic!("Error updating CV file: {e:}");
        }
    }
}

#[cfg(not(tarpaulin_include))]
pub fn create_directory(job_title: &str, company_name: &str) -> Result<String, String> {
    let var = match get_variable_from_config_file("destination", "cv_path") {
        Ok(s) => s,
        Err(e) => {
            error!("Could not get cv_path variable: {e:}");
            return Err(format!("Could not get cv_path variable: {e:}").to_string());
        }
    };

    let destination_folder = fix_home_directory_path(&var);
    let now = GLOBAL_VAR.get().unwrap().get_today();

    match prepare_year_dir(&destination_folder, now) {
        Ok(y) => info!("✅ Year directory created successfully: {y:}"),
        Err(e) => {
            error!("Error creating year directory: {e:}");
            return Err(format!("Error creating year directory: {e:}").to_string());
        }
    }

    let (cv_template_path, full_destination_path) =
        match prepare_path_for_new_cv(job_title, company_name, &destination_folder, now) {
            Ok(s) => s,
            Err(e) => {
                error!("{e:?}");
                return Err(format!("{e:?}").to_string());
            }
        };

    match copy_dir::copy_dir(cv_template_path, full_destination_path.clone()) {
        Ok(_) => info!("✅ Directory created & copied successfully"),
        Err(e) => {
            error!("Error copying directory: {e:}");
            return Err(format!("Error copying directory: {e:}").to_string());
        }
    }
    Ok(full_destination_path)
}

pub fn remove_cv_dir(path_to_remove: &Path) -> std::io::Result<()> {
    fs::remove_dir_all(path_to_remove)?;
    Ok(())
}

#[cfg(not(tarpaulin_include))]
fn prepare_path_for_new_cv(
    job_title: &str,
    company_name: &str,
    destination_folder: &str,
    now: &DateTime<Local>,
) -> Result<(String, String), String> {
    let var = match get_variable_from_config_file("cv", "cv_template_path") {
        Ok(s) => s,
        Err(e) => {
            error!("Could not get cv_template_path variable {e:}");
            return Err(format!("Could not get cv_template_path variable {e:}").to_string());
        }
    };

    let cv_template_path: String = fix_home_directory_path(&var);

    let formatted_job_title = job_title.replace(' ', "-");
    let formatted_company_name = company_name.replace(' ', "-");

    let date_dir = now.format("%Y/%Y-%m-%d").to_string();
    let full_destination_path = fix_home_directory_path(&format!(
        "{destination_folder}/{date_dir}_{formatted_company_name}_{formatted_job_title}"
    ));

    info!("✅ Creating directory: {full_destination_path}");
    info!("✅ Copying from: {}", cv_template_path.clone());

    Ok((cv_template_path, full_destination_path))
}

fn prepare_year_dir(destination_folder: &String, now: &DateTime<Local>) -> Result<String, Error> {
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
    fs::read_to_string(fix_destination_cv_file).expect("Should have been able to read the file")
}

fn change_values_in_destination_cv(
    cv_file_content: &str,
    job_title: &str,
    _quote: &str,
) -> Result<String, String> {
    change_position_in_destination_cv(cv_file_content, job_title)
    // modified_cv_content = change_quote_in_destination_cv(&modified_cv_content, quote);
    // modified_cv_content
}

fn change_position_in_destination_cv(
    cv_file_content: &str,
    job_title: &str,
) -> Result<String, String> {
    let replace_position = get_variable_from_config_file("to_replace", "position_line_to_change")?;

    info!("✅ Changed position from: {replace_position} to: {job_title}");

    let new = cv_file_content.replace(replace_position.as_str(), job_title);

    assert!(new != cv_file_content, "Didn't change shit");

    Ok(new)
}

fn _change_quote_in_destination_cv(cv_file_content: &str, quote: &str) -> String {
    let replace_quote = match get_variable_from_config_file("to_replace", "quote_line_to_change") {
        Ok(s) => s,
        Err(e) => {
            error!("Could not get the quote_line_to_change variable: {e:}");
            panic!("Could not get the quote_line_to_change variable: {e:}");
        }
    };

    if quote.is_empty() {
        info!("✅ Removing quote");

        return cv_file_content
            .lines()
            .filter(|&line| !line.contains(&replace_quote))
            .collect::<Vec<_>>()
            .join("\n");
    }

    info!("✅ Changed quote to: {quote:?}");
    cv_file_content.replace(replace_quote.as_str(), quote)
}

// TODO: function should return Result
pub fn remove_created_dir_from_pro(
    job_title: &str,
    company_name: &str,
    created_cv_dir: &String,
    destination_cv_file_full_path: &str,
) {
    // Remove directory and keep only the pdf file
    let path_created_dir = Path::new(&created_cv_dir);
    let application_date = GLOBAL_VAR.get().unwrap().get_today_str_yyyy_mm_dd();
    let application_year = GLOBAL_VAR.get().unwrap().get_year_str();

    let pdf_file_name = destination_cv_file_full_path.replace(".tex", ".pdf");
    let mut remove_dir_of_cv_path = Path::new(&created_cv_dir)
        .parent()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_owned();
    remove_dir_of_cv_path
        .push_str(format!("/{application_date}-{job_title}-{company_name}.pdf").as_str());

    // TODO: make sure that the Path for Obsidian is fetched from config file
    //
    let destination_cv_pdf_copy = format!("/home/seventh/Documents/Wiki/🧠 P.A.R.A./2. 🌐 Areas/3. 👔 Pro/Dossier_Pro/Applications/{application_year}/{application_date}-{job_title}-{company_name}.pdf");

    copy_to_destination(
        created_cv_dir,
        pdf_file_name.clone(),
        destination_cv_pdf_copy.clone(),
    );
    copy_to_destination(created_cv_dir, pdf_file_name, remove_dir_of_cv_path);

    remove_cv_dir(path_created_dir).unwrap();
}

// TODO: function should return Result
fn copy_to_destination(
    created_cv_dir: &String,
    pdf_file_name: String,
    destination_cv_pdf_copy: String,
) {
    match Command::new("cp")
        .arg(pdf_file_name)
        .arg(destination_cv_pdf_copy)
        .current_dir(created_cv_dir.clone())
        .stdout(Stdio::null())
        .spawn()
    {
        Ok(_) => {
            info!("✅ copy Directory: {created_cv_dir}");
        }
        Err(e) => {
            error!("Could not copy the directory: {created_cv_dir}, with error: {e:}");
            panic!("Could not copy the directory: {created_cv_dir}, with error: {e:}");
        }
    }
}
