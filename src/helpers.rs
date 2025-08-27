use log::{error, info};
use std::fs;
use std::process::{Command, Stdio};

use skim::prelude::*;
use std::io::Cursor;

use crate::config_parse::{get_db_configurations, get_variable_from_config};
use crate::global_conf::GLOBAL_VAR;

pub fn clean_string_from_quotes(cv_template_path: &str) -> String {
    cv_template_path.replace(['\"', '\''], "")
}

pub fn fix_home_directory_path(file_path: &str) -> String {
    if file_path.contains('~') {
        let home_dir = dirs::home_dir().unwrap();
        file_path.replace('~', home_dir.to_str().unwrap())
    } else {
        file_path.to_string()
    }
}

pub fn check_file_exists(file_path: &str) -> Result<String, &str> {
    let fixed_file_path = fix_home_directory_path(file_path);

    // TODO
    // if db file does not exist, create it
    // if fs::metadata(file_path).is_err() {
    //     panic!("File {} does not exist", file_path)
    // };

    if fs::metadata(fixed_file_path.clone()).is_ok() {
        Ok(fixed_file_path)
    } else {
        println!("Could not check if file exists");
        Err("File does not exist!")
    }
}

pub fn check_if_db_env_is_set_or_set_from_config() -> Result<bool, String> {
    let engine = match GLOBAL_VAR.get() {
        Some(eng) => eng.get_user_input_db_engine(),
        None => return Err("Could not get the DATABASE_URL env variable !".to_string()),
    };

    if engine == "postgres" {
        if let Ok(val) = std::env::var("DATABASE_URL") {
            drop(val);
            Ok(true)
        } else {
            let db_url = GLOBAL_VAR.get().unwrap().get_user_input_db_url();
            std::env::set_var("DATABASE_URL", db_url);
            Ok(true)
        }
    } else {
        let db_path = get_db_configurations()?;

        if let Ok(val) = std::env::var("DATABASE_URL") {
            drop(val);
            Ok(true)
        } else {
            std::env::set_var("DATABASE_URL", format!("sqlite://{db_path}"));
            Ok(true)
        }
    }
}

pub fn view_cv_file(cv_path: &str) -> Result<bool, &str> {
    let file_name = match get_variable_from_config("cv", "cv_template_file") {
        Ok(s) => s.to_string(),
        Err(e) => {
            error!("Could not get the cv_template_file variable: {e:?}");
            return Err("Could not get the cv_template_file variable: {e:?}");
        }
    };

    let cv_dir = cv_path.to_string().replace(&file_name, "");

    let pdf_viewer = match get_variable_from_config("optional", "pdf_viewer") {
        Ok(s) => s,
        Err(e) => {
            panic!("Could not the pdf_viewer variable: {e:?}")
        }
    };

    let pdf_file = cv_path.replace(".tex", ".pdf");

    match Command::new(pdf_viewer)
        .current_dir(cv_dir)
        .stdout(Stdio::null())
        .arg(pdf_file)
        .spawn()
    {
        Ok(_) => {
            info!("CV compiled successfully");
            Ok(true)
        }
        Err(e) => {
            error!("Error compiling CV: {e}");
            Err("Error compiling CV: {e}")
        }
    }
}

pub fn my_fzf(list_to_show: Vec<String>) -> String {
    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .build()
        .unwrap();

    let input: String = list_to_show.into_iter().collect();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let selected_items =
        Skim::run_with(&options, Some(items)).map_or_else(Vec::new, |out| out.selected_items);

    if selected_items.len() == 1 {
        selected_items
            .first()
            .expect("Should have had at least one item")
            .output()
            .to_string()
    } else {
        panic!("shit, no items found");
    }
}
