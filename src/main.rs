use dotenvy::dotenv;
use clap::Parser;

mod config_parse;
mod cli_structure;
mod file_handlers;
mod helpers;
mod database;
mod user_action;
mod global_conf;

use crate::cli_structure::{UserInput, match_user_action};
use crate::helpers::{view_cv_file, check_if_db_env_is_set_or_set_from_config, fix_home_directory_path};
use crate::config_parse::{set_global_vars, get_variable_from_config};
use crate::file_handlers::{create_directory, make_cv_changes_based_on_input, compile_cv};



fn main() {
    env_logger::init();
    dotenv().ok();

    let user_input = UserInput::parse();

    set_global_vars(user_input.clone());
    // CONFIG.set(config).unwrap();
    check_if_db_env_is_set_or_set_from_config();

    let cv_full_path = match_user_action();
    if !cv_full_path.is_empty() {
        view_cv_file(&cv_full_path);
    }
}

fn prepare_cv(job_title: &str, company_name: &str, quote: &str) -> String {
    let cv_template_file = fix_home_directory_path(&get_variable_from_config("cv", "cv_template_file"));

    let created_cv_dir = create_directory(job_title, company_name);

    let destination_cv_file_full_path = fix_home_directory_path(&format!("{created_cv_dir}/{cv_template_file}"));

    make_cv_changes_based_on_input(job_title, quote, &destination_cv_file_full_path);
    compile_cv(&created_cv_dir, &cv_template_file);

    destination_cv_file_full_path
}


