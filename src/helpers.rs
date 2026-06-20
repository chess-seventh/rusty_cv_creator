use log::{error, info, warn};
use std::fs;
use std::path::Path;

use skim::prelude::*;
use std::io::Cursor;

use crate::command_runner::{CommandRunner, SystemRunner};
use crate::config_parse::get_db_configurations;
use crate::global_conf::{GLOBAL_VAR, get_global_var};
use crate::is_tailscale_connected;

/// Hint appended to tool-availability errors, nudging the user to run the
/// program inside the devenv shell, where every required tool is provided.
pub const DEVENV_HINT: &str = "Run this program inside the devenv shell so all required tools are on PATH, e.g.\n    devenv shell -- rusty_cv_creator <args>\n  (or run `devenv shell` first, then the command).";

/// Return `true` if `tool` is found as an executable file on the current PATH.
pub fn tool_on_path(tool: &str) -> bool {
    let Ok(path) = std::env::var("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(tool).is_file())
}

/// Pre-usage check: ensure every tool in `tools` is available on PATH before
/// the program tries to run it.
///
/// On failure the error lists the missing tools and suggests running through
/// devenv, which provides them.
pub fn ensure_tools_available(tools: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let missing: Vec<&str> = tools
        .iter()
        .copied()
        .filter(|tool| !tool_on_path(tool))
        .collect();

    if missing.is_empty() {
        info!("✅ Required tools available: {tools:?}");
        return Ok(());
    }

    error!("Missing required tool(s): {missing:?}");
    Err(format!("Missing required tool(s): {missing:?}.\n  {DEVENV_HINT}").into())
}

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
        // Pre-usage check: the postgres path probes connectivity via Tailscale.
        ensure_tools_available(&["sudo", "tailscale"])?;

        if let Ok(val) = std::env::var("DATABASE_URL") {
            drop(val);
        } else {
            let db_url = get_global_var().get_user_input_db_url()?;
            std::env::set_var("DATABASE_URL", db_url);
            info!("Fetched the DATABASE_URL env variable");
        }
        // info!("Checking if Tailscale is connected");
        match is_tailscale_connected(&SystemRunner) {
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

pub fn view_cv_file(
    runner: &dyn CommandRunner,
    cv_path: &str,
    pdf_viewer: &str,
) -> Result<bool, String> {
    // `cv_path` is the final PDF produced by the build; accept a `.tex` path too.
    let is_pdf = Path::new(cv_path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"));
    let pdf_file = if is_pdf {
        cv_path.to_string()
    } else {
        cv_path.replace(".tex", ".pdf")
    };

    match runner.spawn(pdf_viewer, &[&pdf_file]) {
        Ok(()) => {
            info!("Opened CV: {pdf_file}");
            Ok(true)
        }
        Err(e) => {
            error!("Error opening CV: {e:}");
            Err(format!("Error opening CV: {e:}"))
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
        Skim::run_with(options, Some(items)).map_or_else(|_| Vec::new(), |out| out.selected_items);

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

    #[test]
    fn test_tool_on_path_false_for_missing_tool() {
        assert!(!tool_on_path("definitely-not-a-real-tool-xyz"));
    }

    #[test]
    fn test_ensure_tools_available_ok_for_empty() {
        assert!(ensure_tools_available(&[]).is_ok());
    }

    #[test]
    fn test_ensure_tools_available_errors_and_hints_devenv() {
        let err = ensure_tools_available(&["definitely-not-a-real-tool-xyz"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("definitely-not-a-real-tool-xyz"));
        assert!(err.contains("devenv"));
    }

    #[test]
    fn test_view_cv_file_spawns_viewer_ok() {
        let runner = crate::command_runner::testing::FakeRunner::ok();
        assert!(view_cv_file(&runner, "/tmp/cv.pdf", "zathura").unwrap());
        assert_eq!(runner.calls.borrow()[0], "zathura /tmp/cv.pdf");
    }

    #[test]
    fn test_view_cv_file_converts_tex_to_pdf() {
        let runner = crate::command_runner::testing::FakeRunner::ok();
        assert!(view_cv_file(&runner, "/tmp/cv.tex", "zathura").unwrap());
        assert_eq!(runner.calls.borrow()[0], "zathura /tmp/cv.pdf");
    }

    #[test]
    fn test_view_cv_file_errors_when_spawn_fails() {
        let runner = crate::command_runner::testing::FakeRunner::io_error();
        assert!(view_cv_file(&runner, "/tmp/cv.pdf", "zathura").is_err());
    }

    #[test]
    #[serial_test::serial]
    fn test_tool_on_path_true_for_installed_tool() {
        let td = tempfile::TempDir::new().unwrap();
        let tool = "fake-tool-bin";
        fs::write(td.path().join(tool), "#!/bin/sh\n").unwrap();

        let original = std::env::var("PATH").ok();
        std::env::set_var("PATH", td.path());
        let found = tool_on_path(tool);
        match original {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }

        assert!(found);
    }

    // For check_config_file_exists with an actual file,
    // Write a temp file and check (do this after first batch passes)
}
