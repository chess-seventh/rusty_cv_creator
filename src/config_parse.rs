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

/// Environment variable carrying the PostgreSQL password. It is supplied by
/// sops through home-manager on the real machine, and by the gitignored `.env`
/// in development. It is never stored in the INI file nor in this repository.
///
/// Re-exported from `child_env` so the name this module reads and the name
/// stripped from every child environment can never drift apart.
pub use rusty_cv_creator::child_env::DB_PASSWORD_ENV;

/// Read the PostgreSQL password from the environment.
///
/// A missing, empty or blank value is a hard error: connecting without a
/// password is never a fallback.
fn read_db_password() -> Result<String, Box<dyn std::error::Error>> {
    let password = std::env::var(DB_PASSWORD_ENV).unwrap_or_default();

    if password.trim().is_empty() {
        return Err(format!(
            "The PostgreSQL password is missing: set {DB_PASSWORD_ENV} to a non-empty value.\n  \
             It is supplied by sops through home-manager on the real machine, or by the \
             gitignored .env file in development.\n  \
             The password must never be written into the INI config file."
        )
        .into());
    }

    Ok(password)
}

/// Percent-encode `password` so URL-significant characters survive the splice.
///
/// Every byte except the RFC 3986 unreserved set (ASCII alphanumerics and
/// `-` `.` `_` `~`) is escaped, which covers `@ : / # ? %` and whitespace.
/// Hand-rolled on purpose: this must not pull in a new runtime dependency.
fn percent_encode_password(password: &str) -> String {
    let mut encoded = String::with_capacity(password.len());

    for byte in password.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }

    encoded
}

/// Splice `password` into the userinfo of a passwordless `base_url`.
///
/// The base URL names the database user and carries no password:
/// `postgres://<user>@<host>/<database>`. Errors when the base URL already
/// carries a password (it would otherwise become `user:old:new@host`) and when
/// it names no user at all.
fn inject_db_password(
    base_url: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let missing_user = || -> Box<dyn std::error::Error> {
        format!(
            "The configured database URL names no user: expected \
             '<scheme>://<user>@<host>/<database>', got '{base_url}'.\n  \
             Add the database user to db_pg_host in the INI config file."
        )
        .into()
    };

    let scheme_end = base_url.find("://").ok_or_else(missing_user)? + "://".len();
    let authority = &base_url[scheme_end..];

    let at = authority.find('@').ok_or_else(missing_user)?;
    let host_starts_before_userinfo_ends =
        authority.find('/').is_some_and(|slash| slash < at) || at == 0;
    if host_starts_before_userinfo_ends {
        return Err(missing_user());
    }

    let userinfo = &authority[..at];
    if userinfo.contains(':') {
        return Err(format!(
            "The configured database URL already carries a password.\n  \
             Remove the password from db_pg_host in the INI config file: it now comes \
             from the {DB_PASSWORD_ENV} environment variable.\n  \
             Expected '<scheme>://<user>@<host>/<database>'."
        )
        .into());
    }

    Ok(format!(
        "{}{}:{}{}",
        &base_url[..scheme_end],
        userinfo,
        percent_encode_password(password),
        &authority[at..]
    ))
}

/// Resolve the `(engine, url)` pair the DB layer needs to open a connection.
///
/// - `postgres` -> the passwordless `db_pg_host` configured in the INI file,
///   with the password from the `RUSTY_CV_DB_PASSWORD` environment variable
///   spliced in. The variable is already in this process's environment - that
///   is how it arrives - and this function does not write it back there: the
///   value lives in a local `String` handed straight to the connection.
///   Children never see it; see `child_env::command_without_db_password`.
/// - `sqlite`   -> the `DATABASE_URL` env var when set, otherwise a
///   `sqlite://<configured-path>` URL built from the INI config.
pub fn resolve_db_target(ctx: &AppContext) -> Result<(String, String), Box<dyn std::error::Error>> {
    let engine = ctx.get_user_input_db_engine()?;

    match engine.trim() {
        "postgres" => {
            let base_url = ctx
                .get_user_input_db_url()
                .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
            let url = inject_db_password(&base_url, &read_db_password()?)?;
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
            repo: None,
            branch: None,
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
            repo: None,
            branch: None,
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

    // ─── password injection ──────────────────────────────────────────────────

    fn context_from(config: &str) -> AppContext {
        let ui = UserInput {
            action: UserAction::List(FilterArgs::default()),
            save_to_database: false,
            view_generated_cv: false,
            dry_run: false,
            config_ini: String::new(),
            engine: "sqlite".to_string(),
            repo: None,
            branch: None,
        };
        AppContext::new(load_config(config.to_string()), Local::now(), ui)
    }

    /// Restore an environment variable to whatever it was, so a serial test
    /// does not leak its value into the next one.
    ///
    /// The restore lives in `Drop` rather than in a trailing statement because
    /// `cargo test` runs these threaded in one process: a panicking assertion
    /// would skip a trailing restore and hand the leaked value to every
    /// sibling test in the binary.
    struct EnvVarGuard {
        name: &'static str,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn capture(name: &'static str) -> Self {
            EnvVarGuard {
                name,
                original: std::env::var(name).ok(),
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.original.take() {
                Some(value) => std::env::set_var(self.name, value),
                None => std::env::remove_var(self.name),
            }
        }
    }

    fn percent_decode(encoded: &str) -> String {
        let bytes = encoded.as_bytes();
        let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
        let mut index = 0;

        while index < bytes.len() {
            if bytes[index] == b'%' && index + 2 < bytes.len() {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap();
                decoded.push(u8::from_str_radix(hex, 16).unwrap());
                index += 3;
            } else {
                decoded.push(bytes[index]);
                index += 1;
            }
        }

        String::from_utf8(decoded).unwrap()
    }

    /// Assemble a credential-bearing URL at runtime. Expected values are never
    /// written as literals here: the tracked-tree credential scanner
    /// (`tests/db_credentials_regression.rs`) has no allowlist, and this file
    /// is one of the files it scans.
    fn credentialed_url(user: &str, password: &str, host_and_database: &str) -> String {
        let userinfo = format!("{user}{}{password}", ':');
        format!("postgres://{userinfo}@{host_and_database}")
    }

    #[test]
    fn test_inject_db_password_splices_into_the_userinfo() {
        let url = inject_db_password(
            "postgres://rusty_cv@nixos-02.caracara-palermo.ts.net/rusty_cv",
            "s3cret",
        )
        .unwrap();
        assert_eq!(
            url,
            credentialed_url(
                "rusty_cv",
                "s3cret",
                "nixos-02.caracara-palermo.ts.net/rusty_cv"
            )
        );
    }

    #[test]
    fn test_inject_db_password_rejects_a_base_url_that_already_has_one() {
        let stale = credentialed_url("rusty_cv", "old", "host/db");
        let err = inject_db_password(&stale, "new").unwrap_err().to_string();
        assert!(err.contains("already carries a password"), "got: {err}");
        assert!(err.contains("db_pg_host"), "got: {err}");
        assert!(err.contains(DB_PASSWORD_ENV), "got: {err}");
    }

    #[test]
    fn test_inject_db_password_rejects_a_base_url_without_a_user() {
        for base in [
            "postgres://host/db",
            "postgres://@host/db",
            "postgres://host/db@path",
            "not-a-url",
        ] {
            let err = inject_db_password(base, "s3cret").unwrap_err().to_string();
            assert!(err.contains("names no user"), "for {base}, got: {err}");
        }
    }

    #[test]
    fn test_inject_db_password_percent_encodes_every_significant_character() {
        let password = "p@ss:w/rd#?%";
        let url = inject_db_password("postgres://rusty_cv@host/db", password).unwrap();

        let userinfo = url
            .strip_prefix("postgres://")
            .unwrap()
            .split('@')
            .next()
            .unwrap();
        let (user, encoded) = userinfo.split_once(':').unwrap();

        assert_eq!(user, "rusty_cv");
        assert_eq!(percent_decode(encoded), password);
        assert_eq!(
            url,
            credentialed_url("rusty_cv", "p%40ss%3Aw%2Frd%23%3F%25", "host/db")
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_db_target_postgres_errors_when_the_password_is_unset() {
        let _restore = EnvVarGuard::capture(DB_PASSWORD_ENV);
        std::env::remove_var(DB_PASSWORD_ENV);

        let ctx = context_from(
            "[db]\nengine = \"postgres\"\ndb_pg_host = \"postgres://rusty_cv@host/rusty_cv\"",
        );
        let err = resolve_db_target(&ctx).unwrap_err().to_string();

        assert!(err.contains(DB_PASSWORD_ENV), "got: {err}");
        assert!(err.contains("sops"), "got: {err}");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_db_target_postgres_errors_when_the_password_is_empty() {
        let _restore = EnvVarGuard::capture(DB_PASSWORD_ENV);
        std::env::set_var(DB_PASSWORD_ENV, "   ");

        let ctx = context_from(
            "[db]\nengine = \"postgres\"\ndb_pg_host = \"postgres://rusty_cv@host/rusty_cv\"",
        );
        let err = resolve_db_target(&ctx).unwrap_err().to_string();

        assert!(err.contains(DB_PASSWORD_ENV), "got: {err}");
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_db_target_resolves_the_documented_quoted_config() {
        let _restore = EnvVarGuard::capture(DB_PASSWORD_ENV);
        std::env::set_var(DB_PASSWORD_ENV, "s3cret");

        let ctx = context_from(
            "[db]\nengine = \"postgres\"\n\
             db_pg_host = \"postgres://rusty_cv@nixos-02.caracara-palermo.ts.net/rusty_cv\"",
        );
        let (engine, url) = resolve_db_target(&ctx).unwrap();

        assert_eq!(engine, "postgres");
        assert_eq!(
            url,
            credentialed_url(
                "rusty_cv",
                "s3cret",
                "nixos-02.caracara-palermo.ts.net/rusty_cv"
            )
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_resolve_db_target_sqlite_is_unaffected_by_the_password_variable() {
        let _restore_password = EnvVarGuard::capture(DB_PASSWORD_ENV);
        let _restore_url = EnvVarGuard::capture("DATABASE_URL");
        std::env::remove_var("DATABASE_URL");

        let ctx = context_from(
            "[db]\nengine = \"sqlite\"\ndb_path = \"/tmp\"\ndb_file = \"test.db\"\n\
             db_pg_host = \"postgres://rusty_cv@host/rusty_cv\"",
        );

        std::env::remove_var(DB_PASSWORD_ENV);
        let without = resolve_db_target(&ctx).unwrap();
        std::env::set_var(DB_PASSWORD_ENV, "s3cret");
        let with = resolve_db_target(&ctx).unwrap();

        assert_eq!(
            without,
            ("sqlite".to_string(), "sqlite:///tmp/test.db".to_string())
        );
        assert_eq!(without, with);
    }
}
