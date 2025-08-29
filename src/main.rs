use core::panic;

use clap::Parser;
use dotenvy::dotenv;
use log::{error, info};
use std::io::{self};
use std::process::Command;

mod cli_structure;
mod config_parse;
mod cv_insert;
mod database;
mod file_handlers;
mod global_conf;
mod helpers;
mod user_action;

use crate::cli_structure::{match_user_action, UserAction, UserInput};
use crate::config_parse::{get_variable_from_config_file, set_global_vars};
use crate::file_handlers::{compile_cv, create_directory, make_cv_changes_based_on_input};
use crate::global_conf::get_global_var;
use crate::helpers::{
    check_if_db_env_is_set_or_set_from_config, fix_home_directory_path, view_cv_file,
};

#[cfg(not(tarpaulin_include))]
fn main() {
    env_logger::init();
    dotenv().ok();

    let user_input = UserInput::parse();

    set_global_vars(&user_input.clone());
    check_if_db_env_is_set_or_set_from_config();

    let _action: UserAction = get_global_var().get_user_input_action();

    let cv_full_path = match_user_action(user_input.clone());

    if !cv_full_path.is_empty() {
        if user_input.view_generated_cv {
            match view_cv_file(&cv_full_path) {
                Ok(b) => b,
                Err(e) => panic!("{e:}"),
            };
        } else {
            info!("CV saved to: {cv_full_path}");
            // println!("CV saved to: {cv_full_path}");
        }
    }
}

fn prepare_cv(
    job_title: &str,
    company_name: &str,
    quote: Option<&String>,
) -> Result<String, String> {
    let cfg = match get_variable_from_config_file("cv", "cv_template_file") {
        Ok(c) => c,
        Err(e) => {
            error!("Something went wrong when gathering variable from config: {e:?}");
            return Err(
                format!("Something went wrong when gathering variable from config: {e:?}")
                    .to_string(),
            );
        }
    };
    let cv_template_file = fix_home_directory_path(&cfg);

    let created_cv_dir = match create_directory(job_title, company_name) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not create directory for CV: {e:?}");
            return Err(format!("Could not create directory for CV: {e:?}").to_string());
        }
    };

    let destination_cv_file_full_path =
        fix_home_directory_path(&format!("{created_cv_dir}/{cv_template_file}"));

    make_cv_changes_based_on_input(job_title, quote, &destination_cv_file_full_path)?;
    compile_cv(&created_cv_dir, &cv_template_file);

    file_handlers::remove_created_dir_from_pro(
        job_title,
        company_name,
        &created_cv_dir,
        &destination_cv_file_full_path,
    );

    Ok(destination_cv_file_full_path)
}

/// Checks if the device is connected to Tailscale.
/// Returns true if up, false if not, or Err if unable to check.
fn is_tailscale_connected() -> io::Result<bool> {
    let output = Command::new("sudo").arg("tailscale").arg("status").output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Tailscale outputs "Logged out." if disconnected,
                // and network details if connected
                Ok(!stdout.contains("Logged out."))
            } else {
                Err(io::Error::other("tailscale status command failed"))
            }
        }
        Err(e) => Err(e),
    }
}
