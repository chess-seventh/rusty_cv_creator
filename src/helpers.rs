use log::{error, info, warn};
use std::fs;
use std::process::{Command, Stdio};

use skim::prelude::*;
use std::io::Cursor;

use crate::config_parse::{get_db_configurations, get_variable_from_config_file};
use crate::global_conf::{GLOBAL_VAR, get_global_var};
use crate::is_tailscale_connected;

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

pub fn check_config_file_exists(file_path: &str) -> Result<String, &str> {
    let fixed_file_path = fix_home_directory_path(file_path);

    // TODO: MINOR
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

pub fn check_if_db_env_is_set_or_set_from_config() -> Result<String, Box<dyn std::error::Error>> {
    let engine = if let Some(eng) = GLOBAL_VAR.get() {
        eng.get_user_input_db_engine()
    } else {
        warn!("Could not get the DATABASE_URL env variable !");
        Err("Could not get the DATABASE_URL env variable !"
            .to_string()
            .into())
    };

    if engine.is_ok_and(|e| "postgres" == e) {
        if let Ok(val) = std::env::var("DATABASE_URL") {
            drop(val);
        } else {
            let db_url = get_global_var().get_user_input_db_url()?;
            std::env::set_var("DATABASE_URL", db_url);
            info!("Fetched the DATABASE_URL env variable");
        }
        // info!("Checking if Tailscale is connected");
        match is_tailscale_connected() {
            Ok(true) => {
                info!("Device is connected to Tailscale!");
                Ok("Device is connected to Tailscale!".to_string())
            }
            Ok(false) => {
                info!("Device is NOT connected to Tailscale.");
                Ok("Device is NOT connected to Tailscale.".to_string())
            }
            Err(e) => {
                warn!("Tailscale issue: {e:}");
                Err(e.into())
            }
        }
    } else {
        //TODO: fix unwrap
        let db_path = match get_db_configurations() {
            Ok(db) => db,
            Err(e) => {
                warn!("Could not get the db configuration: {e:}");
                String::new()
            }
        };

        if let Ok(val) = std::env::var("DATABASE_URL") {
            drop(val);
            Ok("If Let OKAY: Set the DATABASE_URL env variable".to_string())
        } else {
            std::env::set_var("DATABASE_URL", format!("sqlite://{db_path}"));
            Ok("If Let NOT OKAY: Set the DATABASE_URL env variable".to_string())
        }
    }
}

pub fn view_cv_file(cv_path: &str) -> Result<bool, String> {
    let file_name = match get_variable_from_config_file("cv", "cv_template_file") {
        Ok(s) => s.to_string(),
        Err(e) => {
            error!("Could not get the cv_template_file variable: {e:}");
            return Err(format!("Could not get the cv_template_file variable: {e:}").to_string());
        }
    };

    let cv_dir = cv_path.to_string().replace(&file_name, "");

    let pdf_viewer = match get_variable_from_config_file("optional", "pdf_viewer") {
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
            error!("Error compiling CV: {e:}");
            Err(format!("Error compiling CV: {e:}").to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_from_double_quotes() {
        let input = "\"sample text\"";
        let output = clean_string_from_quotes(input);
        assert_eq!(output, "sample text");
    }

    #[test]
    fn test_clean_string_from_single_quotes() {
        let input = "'sample text'";
        let output = clean_string_from_quotes(input);
        assert_eq!(output, "sample text");
    }

    #[test]
    fn test_clean_string_from_mixed_quotes() {
        let input = "\"sam'ple te'xt\"";
        let output = clean_string_from_quotes(input);
        assert_eq!(output, "sample text");
    }

    #[test]
    fn test_clean_string_from_no_quotes() {
        let input = "plain text";
        let output = clean_string_from_quotes(input);
        assert_eq!(output, "plain text");
    }

    #[test]
    fn test_fix_home_directory_path_with_tilde() {
        let input = "~/some/path";
        let expanded = fix_home_directory_path(input);
        assert!(!expanded.contains('~'));
        assert!(expanded.contains("some/path"));
    }

    #[test]
    fn test_fix_home_directory_path_absolute() {
        let input = "/absolute/path";
        let expanded = fix_home_directory_path(input);
        assert_eq!(expanded, "/absolute/path");
    }

    #[test]
    fn test_check_config_file_exists_nonexistent_file() {
        let result = check_config_file_exists("/definitely/does/not/exist");
        assert!(result.is_err());
    }

    // For check_config_file_exists with an actual file,
    // Write a temp file and check (do this after first batch passes)
}
