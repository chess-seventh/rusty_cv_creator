mod config_parse;
mod cli_structure;
mod file_handlers;
mod helpers;
mod database;

use log::info;
use crate::cli_structure::UserInput;
use crate::config_parse::CONFIG;
use dotenvy::dotenv;

use clap::Parser;
use crate::helpers::view_cv_file;

fn main() {
    dotenv().ok();

    let user_input = UserInput::parse();

    let config_path = user_input.clone().config_ini;
    let config = config_parse::read_config_file(&config_path);
    CONFIG.set(config).unwrap();
    helpers::check_if_db_env_is_set_or_set_from_config();

    cli_structure::match_user_action(user_input);
}

fn prepare_cv(job_title: &str, company_name: &str, quote: &str) -> String {
    let cv_template_file = helpers::fix_home_directory_path(&config_parse::get_variable_from_config("cv", "cv_template_file"));
    let created_cv_dir = file_handlers::create_directory(job_title, company_name);
    let destination_cv_file_full_path = helpers::fix_home_directory_path(&format!("{created_cv_dir}/{cv_template_file}"));

    file_handlers::make_cv_changes_based_on_input(job_title, quote, &destination_cv_file_full_path);
    file_handlers::compile_cv(&created_cv_dir, &cv_template_file);

    destination_cv_file_full_path
}

fn run_insert(save_to_db: bool, insert: &cli_structure::InsertCV) {
    let destination_cv_file_full_path = prepare_cv(&insert.job_title, &insert.company_name, &insert.quote);
    if save_to_db {
        let _db_cv = database::save_new_cv_to_database(&insert.job_title, &insert.company_name, &destination_cv_file_full_path, &insert.quote);
        info!("Saved CV to database");
        view_cv_file(&destination_cv_file_full_path);
    }
}
