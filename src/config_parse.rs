use crate::cli_structure::UserInput;
use crate::global_conf::AppContext;
use crate::helpers::{check_config_file_exists, clean_string_from_quotes, fix_home_directory_path};
use configparser::ini::Ini;
use log::{debug, info};
use std::fs;

/// Build the immutable [`AppContext`] for this run (ADR-0006).
///
/// Loads the INI file referenced by `user_input`, captures the run timestamp,
/// and bundles them with the parsed `UserInput`. Constructed once in `main` and
/// then threaded by shared borrow — replaces the former `set_global_vars`.
pub fn build_context(user_input: &UserInput) -> AppContext {
    let read_file_path = user_input.clone().config_ini;
    info!("Reading config file here: {read_file_path:}");

    let file_path = match check_config_file_exists(read_file_path.as_str()) {
        Ok(filepath) => filepath,
        Err(e) => panic!("Error in checking that the file path exists: {e:}"),
    };

    let contents = fs::read_to_string(file_path.clone())
        .unwrap_or_else(|_| panic!("Should have been able to read the file: {file_path:}"));

    let config = load_config(contents);
    let today = chrono::offset::Local::now();

    AppContext::new(config, today, user_input.clone())
}

fn load_config(config_string: String) -> Ini {
    info!("Reading the config file");
    let mut config = Ini::new();
    config
        .read(config_string)
        .expect("Could not read the INI file!");
    config
}

pub fn get_variable_from_config_file(
    ctx: &AppContext,
    section: &str,
    variable: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Retrieving from config: {section:} {variable:}");
    let config_get = ctx.get_user_input_vars(section, variable)?;

    let value = fix_home_directory_path(&config_get);

    Ok(clean_string_from_quotes(&value))
}

pub fn get_db_configurations(ctx: &AppContext) -> Result<String, Box<dyn std::error::Error>> {
    debug!("Getting DB Configuration");
    let cfg_db_path = ctx.get_user_input_vars("db", "db_path")?;
    let cfg_db_file = ctx.get_user_input_vars("db", "db_file")?;

    let mut db_path = fix_home_directory_path(&clean_string_from_quotes(&cfg_db_path));
    let db_file = clean_string_from_quotes(&cfg_db_file);

    db_path.push('/');
    db_path.push_str(db_file.as_str());
    Ok(db_path)
}

/// Resolve the `(engine, url)` pair the DB layer needs to open a connection.
///
/// Mirrors [`crate::helpers::check_if_db_env_is_set_or_set_from_config`]:
/// - `postgres` -> the `db_pg_host` configured in the INI file.
/// - `sqlite`   -> the `DATABASE_URL` env var when set, otherwise a
///   `sqlite://<configured-path>` URL built from the INI config.
pub fn resolve_db_target(ctx: &AppContext) -> Result<(String, String), Box<dyn std::error::Error>> {
    let engine = ctx.get_user_input_db_engine()?;

    match engine.trim() {
        "postgres" => {
            let url = ctx
                .get_user_input_db_url()
                .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
            Ok((engine, url))
        }
        "sqlite" => {
            let url = match std::env::var("DATABASE_URL") {
                Ok(value) => fix_home_directory_path(&value),
                Err(_) => format!("sqlite://{}", get_db_configurations(ctx)?),
            };
            Ok((engine, url))
        }
        _ => Ok((engine, String::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_structure::{FilterArgs, UserAction};
    use chrono::Local;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn empty_context() -> AppContext {
        let ui = UserInput {
            action: UserAction::List(FilterArgs::default()),
            save_to_database: false,
            view_generated_cv: false,
            dry_run: false,
            config_ini: String::new(),
            engine: "sqlite".to_string(),
        };
        AppContext::new(Ini::new(), Local::now(), ui)
    }

    #[test]
    fn test_load_config_reads_values() {
        let config = "[section]\nkey = \"value\"";
        let ini = load_config(config.to_string());
        assert_eq!(ini.get("section", "key").unwrap(), "\"value\"");
    }

    #[test]
    fn test_build_context_loads_config() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "[db]\ndb_path = \"/tmp\"\ndb_file = \"test.db\"").unwrap();
        let ui = UserInput {
            action: UserAction::List(FilterArgs::default()),
            save_to_database: false,
            view_generated_cv: false,
            dry_run: false,
            config_ini: f.path().to_str().unwrap().to_string(),
            engine: "sqlite".to_string(),
        };
        let ctx = build_context(&ui);
        assert_eq!(get_db_configurations(&ctx).unwrap(), "/tmp/test.db");
    }

    #[test]
    fn test_get_variable_from_config_file_error_if_missing() {
        let ctx = empty_context();
        let result = get_variable_from_config_file(&ctx, "missing", "key");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_db_configurations_error_if_missing() {
        let ctx = empty_context();
        let result = get_db_configurations(&ctx);
        assert!(result.is_err());
    }
}
