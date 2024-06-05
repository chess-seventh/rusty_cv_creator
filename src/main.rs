mod config_parse;
mod cli_structure;
mod file_handlers;

use std::fs;
use crate::cli_structure::UserInput;
use crate::config_parse::CONFIG;

use clap::Parser;

fn main() {

    let user_input = UserInput::parse();
    // println!("{:#?}", user_input);

    let config_path = user_input.config_ini;
    let save_to_database = user_input.save_to_database;
    check_file_exists(&config_path);

    let config = config_parse::read_config_file(&config_path);
    CONFIG.set(config).unwrap();


    let (_cv_path, _db_path) = config_parse::get_configurations(save_to_database);
    let (job_title, company_name, quote) = cli_structure::match_user_action(user_input.action);

    let created_cv_dir = file_handlers::create_directory(&job_title, &company_name);

    let destination_cv_file_full_path = format!("{created_cv_dir}/PivaFrancescoCV.tex");

    make_cv_changes_based_on_input(&job_title, quote, &destination_cv_file_full_path);

    file_handlers::compile_cv(&created_cv_dir, "PivaFrancescoCV.tex");
}

fn check_file_exists(file_path: &str) {
    // TODO if db file does not exist, create it
    // if fs::metadata(file_path).is_err() {
    //     panic!("File {} does not exist", file_path)
    // };
    assert!(fs::metadata(file_path).is_ok(), "File {file_path} does not exist");
}

fn make_cv_changes_based_on_input(job_title: &str, quote: Option<String>, cv_file_path: &str) {
    let cv_file_content = read_destination_cv_file(cv_file_path);
    let changed_content = change_values_in_destination_cv(&cv_file_content, job_title, quote);
    match write_to_destination_cv_file(cv_file_path, &changed_content) {
        Ok(()) => println!("CV file updated successfully"),
        Err(e) => println!("Error updating CV file: {e}")
    }
}

fn write_to_destination_cv_file(cv_file_path: &str, content: &String) -> std::io::Result<()> {
    fs::write(cv_file_path, content)?;
    Ok(())
}

fn read_destination_cv_file(destination_cv_file: &str) -> String {
    println!("Reading CV file: {destination_cv_file}");
    fs::read_to_string(destination_cv_file)
        .expect("Should have been able to read the file")
}

fn get_position_value_to_change() -> String {
    let config = crate::CONFIG.get().unwrap().clone();
    let mut replace_position: String = config.get("to_replace", "position_line_to_change").unwrap().clone();
    replace_position = config_parse::clean_string_from_quotes(&replace_position.clone());
    replace_position
}

fn get_quote_value_to_change() -> String {
    let config = crate::CONFIG.get().unwrap().clone();
    let mut replace_quote: String = config.get("to_replace", "quote_line_to_change").unwrap().clone();
    replace_quote = config_parse::clean_string_from_quotes(&replace_quote.clone());
    replace_quote
}

fn change_values_in_destination_cv(cv_file_content: &str, job_title: &str, quote: Option<String>) -> String {
    let mut modified_cv_content = change_position_in_destination_cv(cv_file_content, job_title);
    modified_cv_content = change_quote_in_destination_cv(&modified_cv_content, quote);

    println!("CV Updated!");
    modified_cv_content
}

fn change_position_in_destination_cv(cv_file_content: &str, job_title: &str) -> String {
    let replace_position = get_position_value_to_change();
    println!("Changed position to: {replace_position}");
    cv_file_content.replace(replace_position.as_str(), job_title)
}

fn change_quote_in_destination_cv(cv_file_content: &str, quote: Option<String>) -> String {
    let replace_quote = get_quote_value_to_change();
    if quote.is_none() {
        println!("Removing quote");

        return cv_file_content.lines()
            .filter(|&line| line.trim() != replace_quote)
            .collect::<Vec<_>>()
            .join("\n");
    }

    println!("Changed quote to: {replace_quote}");
    cv_file_content.replace(replace_quote.as_str(), quote.unwrap_or_default().as_str())

}
