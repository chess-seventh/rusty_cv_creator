use crate::global_conf::GlobalVars;
use clap::Parser;
use dotenvy::dotenv;
use log::{error, info};
use std::path::Path;
use std::process::{Command, Stdio};

mod cli_structure;
mod config_parse;
mod database;
mod file_handlers;
mod global_conf;
mod helpers;
mod user_action;

use crate::cli_structure::{match_user_action, UserInput};
use crate::config_parse::{get_variable_from_config, set_global_vars};
use crate::file_handlers::{compile_cv, create_directory, make_cv_changes_based_on_input};
use crate::helpers::{
    check_if_db_env_is_set_or_set_from_config, fix_home_directory_path, view_cv_file,
};

fn main() {
    env_logger::init();
    dotenv().ok();

    let user_input = UserInput::parse();

    set_global_vars(user_input.clone());
    // CONFIG.set(config).unwrap();
    check_if_db_env_is_set_or_set_from_config();

    let cv_full_path = match_user_action();
    if !cv_full_path.is_empty() {
        if user_input.view_generated_cv {
            view_cv_file(&cv_full_path);
        } else {
            println!("CV saved to: {cv_full_path}");
        }
    }
}

fn prepare_cv(job_title: &str, company_name: &str, quote: &str) -> String {
    let cv_template_file =
        fix_home_directory_path(&get_variable_from_config("cv", "cv_template_file"));

    let created_cv_dir = create_directory(job_title, company_name);

    let destination_cv_file_full_path =
        fix_home_directory_path(&format!("{created_cv_dir}/{cv_template_file}"));

    make_cv_changes_based_on_input(job_title, quote, &destination_cv_file_full_path);
    compile_cv(&created_cv_dir, &cv_template_file);

    // Remove directory and keep only the pdf file

    let path_created_dir = Path::new(&created_cv_dir);
    let pdf_file_name = destination_cv_file_full_path.replace(".tex", ".pdf");

    let application_date = GlobalVars::get_today_str();

    let destination_cv_pdf_copy = format!("/home/seventh/Documents/Wiki/🧠 P.A.R.A./2. 🌐 Areas/3. 👔 Pro/Dossier_Pro/Applications/{application_date}-{job_title}-{company_name}.pdf");

    match Command::new("cp")
        .arg(pdf_file_name)
        .arg(destination_cv_pdf_copy)
        .current_dir(created_cv_dir.clone())
        .stdout(Stdio::null())
        .spawn()
    {
        Ok(_) => {
            info!("✅ Removed Directory: {created_cv_dir}");
        }
        Err(e) => {
            error!("Could not remove the directory: {created_cv_dir}, with error: {e}");
            panic!("Could not remove the directory: {created_cv_dir}, with error: {e}");
        }
    }

    file_handlers::remove_cv_dir(path_created_dir).unwrap();

    match Command::new("rm")
        .arg("-rdf")
        .current_dir(created_cv_dir.clone())
        .stdout(Stdio::null())
        .spawn()
    {
        Ok(_) => {
            info!("✅ Removed Directory: {created_cv_dir}");
        }
        Err(e) => {
            error!("Could not remove the directory: {created_cv_dir}, with error: {e}");
            panic!("Could not remove the directory: {created_cv_dir}, with error: {e}");
        }
    }

    destination_cv_file_full_path
}
