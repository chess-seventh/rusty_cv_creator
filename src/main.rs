#[allow(unused_imports)]
use std::error::Error;

use clap::Parser;
use dotenvy::dotenv;

mod cli_structure;
mod config_parse;

mod cv;

mod database;
mod file_handlers;
mod global_conf;
mod helpers;
mod user_action;

use crate::cli_structure::{match_user_action, UserInput};

use crate::config_parse::{get_variable_from_config, set_global_vars};

use crate::cv::cv_main::cv_generate;

use crate::file_handlers::{compile_cv, create_directory, make_cv_changes_based_on_input};

use crate::helpers::{
    check_if_db_env_is_set_or_set_from_config, fix_home_directory_path, view_cv_file,
};

fn main() {
    cv_generate()
}

#[allow(dead_code)]
fn og_main() {
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

#[allow(dead_code)]
fn prepare_cv(job_title: &str, company_name: &str, quote: &str) -> String {
    let cv_template_file =
        fix_home_directory_path(&get_variable_from_config("cv", "cv_template_file"));

    let created_cv_dir = create_directory(job_title, company_name);

    let destination_cv_file_full_path =
        fix_home_directory_path(&format!("{created_cv_dir}/{cv_template_file}"));

    make_cv_changes_based_on_input(job_title, quote, &destination_cv_file_full_path);
    compile_cv(&created_cv_dir, &cv_template_file);

    destination_cv_file_full_path
}
