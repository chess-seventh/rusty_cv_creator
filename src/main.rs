#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use core::panic;

use clap::Parser;
use dotenvy::dotenv;
use log::{error, info};
use std::io;

mod cli_structure;
mod command_runner;
mod config_parse;
mod cv_insert;
mod database;
mod file_handlers;
mod global_conf;
mod helpers;
mod user_action;

use crate::cli_structure::{UserAction, UserInput, match_user_action};
use crate::command_runner::{CommandRunner, SystemRunner};
use crate::config_parse::{get_variable_from_config_file, set_global_vars};
use crate::file_handlers::{
    BuildConfig, compile_cv, create_directory, remove_created_dir_from_pro, resolve_variant,
};
use crate::global_conf::get_global_var;
use crate::helpers::{
    check_if_db_env_is_set_or_set_from_config, ensure_tools_available, view_cv_file,
};

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() {
    env_logger::init();
    dotenv().ok();

    let user_input = UserInput::parse();

    set_global_vars(&user_input.clone());
    let _ = check_if_db_env_is_set_or_set_from_config();

    let _action: UserAction = get_global_var().get_user_input_action();

    let cv_full_path = match_user_action(user_input.clone());

    if !cv_full_path.is_empty() {
        if user_input.view_generated_cv {
            let pdf_viewer = get_variable_from_config_file("optional", "pdf_viewer")
                .unwrap_or_else(|e| panic!("Could not get the pdf_viewer variable: {e:?}"));
            if let Err(e) = ensure_tools_available(&[pdf_viewer.as_str()]) {
                panic!("{e:}");
            }
            match view_cv_file(&SystemRunner, &cv_full_path, &pdf_viewer) {
                Ok(b) => b,
                Err(e) => panic!("{e:}"),
            };
        } else {
            info!("CV saved to: {cv_full_path}");
        }
    }
}

fn prepare_cv(
    runner: &dyn CommandRunner,
    job_title: &str,
    company_name: &str,
    variant_flag: Option<&String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let default_variant = get_variable_from_config_file("variant", "default")
        .unwrap_or_else(|_| "senior-devops".to_string());
    let variant = resolve_variant(variant_flag, job_title, &default_variant);
    info!("Selected CV variant: {variant}");

    let cfg = BuildConfig::from_config()?;
    let pdf_basename = format!("{}-{variant}.pdf", cfg.prefix);

    // Pre-usage check: the builder (`just`) drives `tectonic` via the Justfile.
    ensure_tools_available(&[cfg.builder.as_str(), "tectonic"])?;

    let created_cv_dir = match create_directory(job_title, company_name) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not create directory for CV: {e:?}");
            return Err(format!("Could not create directory for CV: {e:?}")
                .to_string()
                .into());
        }
    };

    compile_cv(runner, &created_cv_dir, &variant, &cfg)?;

    let output_pdf =
        remove_created_dir_from_pro(job_title, company_name, &created_cv_dir, &pdf_basename)?;

    Ok(output_pdf)
}

/// Checks if the device is connected to Tailscale.
/// Returns true if up, false if not, or Err if unable to check.
fn is_tailscale_connected(runner: &dyn CommandRunner) -> io::Result<bool> {
    let (success, stdout) = runner.output("sudo", &["tailscale", "status"])?;

    if success {
        // Tailscale prints "Logged out." when disconnected, network details otherwise.
        Ok(!stdout.contains("Logged out."))
    } else {
        Err(io::Error::other("tailscale status command failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_runner::testing::FakeRunner;

    /// A fake builder that "compiles" by writing the expected PDF into `cwd`.
    struct PdfWritingRunner {
        pdf_name: String,
    }

    impl CommandRunner for PdfWritingRunner {
        fn status(&self, _program: &str, _args: &[&str], cwd: Option<&str>) -> io::Result<bool> {
            if let Some(dir) = cwd {
                std::fs::write(format!("{dir}/{}", self.pdf_name), b"%PDF-1.4")?;
            }
            Ok(true)
        }
        fn output(&self, _program: &str, _args: &[&str]) -> io::Result<(bool, String)> {
            Ok((true, String::new()))
        }
        fn spawn(&self, _program: &str, _args: &[&str]) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_prepare_cv_end_to_end_with_fake_builder() {
        let td = tempfile::TempDir::new().unwrap();
        let base = td.path();
        let template = base.join("template");
        std::fs::create_dir_all(&template).unwrap();
        std::fs::write(template.join("TestCV-senior-devops.tex"), "x").unwrap();
        let dest = base.join("dest");
        let out = base.join("out");
        std::fs::create_dir_all(&dest).unwrap();

        let ini = format!(
            "[cv]\ncv_template_path = \"{tpl}\"\ncv_file_prefix = \"TestCV\"\n\
             [variant]\ndefault = \"senior-devops\"\n\
             [build]\nbuilder = \"just\"\nrecipe = \"build\"\n\
             [destination]\ncv_path = \"{dest}\"\noutput_pdf = \"{out}\"\n\
             [db]\nengine = \"sqlite\"\ndb_file = \"x.db\"\n",
            tpl = template.display(),
            dest = dest.display(),
            out = out.display()
        );
        let ini_path = base.join("conf.ini");
        std::fs::write(&ini_path, ini).unwrap();

        let ui = UserInput {
            action: UserAction::Insert(cli_structure::FilterArgs::default()),
            save_to_database: false,
            view_generated_cv: false,
            dry_run: false,
            config_ini: ini_path.to_str().unwrap().to_string(),
            engine: "sqlite".to_string(),
        };
        set_global_vars(&ui);

        let runner = PdfWritingRunner {
            pdf_name: "TestCV-senior-devops.pdf".to_string(),
        };
        // "Senior DevOps" infers the senior-devops variant.
        let output_pdf = prepare_cv(&runner, "Senior DevOps", "ACME", None).unwrap();

        let out_path = std::path::Path::new(&output_pdf);
        assert!(out_path.is_file());
        assert_eq!(out_path.extension().and_then(|e| e.to_str()), Some("pdf"));
    }

    #[test]
    fn test_is_tailscale_connected_true_when_details() {
        let runner = FakeRunner::with_stdout("100.64.0.1 my-machine ...");
        assert!(is_tailscale_connected(&runner).unwrap());
    }

    #[test]
    fn test_is_tailscale_connected_false_when_logged_out() {
        let runner = FakeRunner::with_stdout("Logged out.");
        assert!(!is_tailscale_connected(&runner).unwrap());
    }

    #[test]
    fn test_is_tailscale_connected_err_on_command_failure() {
        let runner = FakeRunner::failing();
        assert!(is_tailscale_connected(&runner).is_err());
    }

    #[test]
    fn test_is_tailscale_connected_err_on_io_error() {
        let runner = FakeRunner::io_error();
        assert!(is_tailscale_connected(&runner).is_err());
    }
}
