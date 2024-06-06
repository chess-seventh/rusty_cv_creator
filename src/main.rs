mod config_parse;
mod cli_structure;
mod file_handlers;
mod helpers;
mod database;

use crate::cli_structure::UserInput;
use crate::config_parse::CONFIG;

use clap::Parser;

fn main() {

    let user_input = UserInput::parse();

    let config_path = user_input.config_ini;

    helpers::check_file_exists(&config_path);

    let config = config_parse::read_config_file(&config_path);
    CONFIG.set(config).unwrap();

    let (job_title, company_name, quote) = cli_structure::match_user_action(user_input.action);

    let cv_template_file = config_parse::get_cv_template_file();

    let created_cv_dir = file_handlers::create_directory(&job_title, &company_name);

    let destination_cv_file_full_path = format!("{created_cv_dir}/{cv_template_file}");

    file_handlers::make_cv_changes_based_on_input(&job_title, quote, &destination_cv_file_full_path);

    file_handlers::compile_cv(&created_cv_dir, &cv_template_file);
}

fn save_new_cv_to_database(destination_cv_file_full_path: &str, job_title: &str, company_name: &str, quote: Option<String>) {
    let db_path = config_parse::get_db_configurations();
    let db = database::Database::new(&db_path);

    db.insert_new_cv(destination_cv_file_full_path, job_title, company_name);
}