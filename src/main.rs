use core::panic;

use clap::Parser;
use dotenvy::dotenv;
use log::{error, info};
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

    match set_global_vars(&user_input.clone()) {
        Ok(o) => info!("all good: {o}"),
        Err(e) => panic!("could not set global vars {e}"),
    };

    match check_if_db_env_is_set_or_set_from_config() {
        Ok(_v) => info!("Fetched the DATABASE_URL env variable"),
        Err(v) => panic!("{}", v),
    };

    let cv_full_path = match_user_action();

    if !cv_full_path.is_empty() {
        match user_input.view_generated_cv {
            Some(true) => {
                match view_cv_file(&cv_full_path) {
                    Ok(b) => b,
                    Err(e) => panic!("{e:?}"),
                };
            }
            Some(false) | None => {
                info!("CV saved to: {cv_full_path}");
                // println!("CV saved to: {cv_full_path}");
            }
        }
    }
}

fn prepare_cv(job_title: &str, company_name: &str, quote: &str) -> Result<String, String> {
    let cfg = match get_variable_from_config("cv", "cv_template_file") {
        Ok(c) => c,
        Err(e) => {
            error!("Something went wrong when gathering variable from config: {e:?}");
            return Err(
                "Something went wrong when gathering variable from config: {e:?}".to_string(),
            );
        }
    };
    let cv_template_file = fix_home_directory_path(&cfg);

    let created_cv_dir = match create_directory(job_title, company_name) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not create directory for CV: {e:?}");
            return Err("Could not create directory for CV: {e:?}".to_string());
        }
    };

    let destination_cv_file_full_path =
        fix_home_directory_path(&format!("{created_cv_dir}/{cv_template_file}"));

    make_cv_changes_based_on_input(job_title, quote, &destination_cv_file_full_path);
    compile_cv(&created_cv_dir, &cv_template_file);

    file_handlers::remove_created_dir_from_pro(
        job_title,
        company_name,
        &created_cv_dir,
        &destination_cv_file_full_path,
    );

    Ok(destination_cv_file_full_path)
}
