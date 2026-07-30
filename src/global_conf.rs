use crate::{
    UserInput,
    cli_structure::{FilterArgs, UserAction},
    helpers::clean_string_from_quotes,
};
use chrono::{DateTime, Local};
use configparser::ini::Ini;

/// Immutable, dependency-injected configuration value (ADR-0006).
///
/// Replaces the former process-global config cell (`OnceCell`): the parsed INI
/// config, the run timestamp (`today`) and the parsed `UserInput` are captured
/// once in `main` and threaded by shared borrow (`&AppContext`). It exposes
/// **read accessors only** — no setters, no interior mutability — so "a `&self`
/// method silently mutates shared config" is non-representable.
#[derive(Debug, Clone)]
pub struct AppContext {
    config: Ini,
    today: DateTime<Local>,
    user_input: UserInput,
}

impl AppContext {
    pub fn new(config: Ini, today: DateTime<Local>, user_input: UserInput) -> Self {
        AppContext {
            config,
            today,
            user_input,
        }
    }

    // Part of the read-only accessor surface preserved from `GlobalVars`
    // (ADR-0006). Currently exercised by tests / kept for parity.
    #[allow(dead_code)]
    pub fn config(&self) -> &Ini {
        &self.config
    }

    pub fn get_user_input_vars(
        &self,
        section: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.config
            .get(section, key)
            .ok_or(format!("Could not get {section:} {key:}").into())
    }

    pub fn get_today(&self) -> &DateTime<Local> {
        &self.today
    }

    pub fn get_today_str(&self) -> String {
        self.get_today().format("%e-%b-%Y").to_string()
    }

    pub fn get_today_str_yyyy_mm_dd(&self) -> String {
        self.get_today().format("%Y-%m-%d").to_string()
    }

    pub fn get_year_str(&self) -> String {
        self.get_today().format("%Y").to_string()
    }

    pub fn get_user_input(&self) -> UserInput {
        self.user_input.clone()
    }

    pub fn get_user_input_action(&self) -> UserAction {
        self.user_input.action.clone()
    }

    pub fn get_user_input_action_filter_args(&self) -> FilterArgs {
        match self.get_user_input_action() {
            UserAction::Insert(insert_args) => insert_args.into(),
            UserAction::Remove(filter_args)
            | UserAction::List(filter_args)
            | UserAction::Update(filter_args) => filter_args,
        }
    }

    pub fn get_job_title(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self.get_user_input_action_filter_args().job_title {
            Some(job) => Ok(job),
            None => Err("This filter does not have the 'job_title' keyword"
                .to_string()
                .into()),
        }
    }

    pub fn get_company_name(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self.get_user_input_action_filter_args().company_name {
            Some(job) => Ok(job),
            None => Err("This filter does not have the 'company_name' keyword"
                .to_string()
                .into()),
        }
    }

    pub fn get_quote(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self.get_user_input_action_filter_args().quote {
            Some(job) => Ok(job),
            None => Err("This filter does not have the 'quote' keyword"
                .to_string()
                .into()),
        }
    }

    pub fn get_variant(&self) -> Option<String> {
        self.get_user_input_action_filter_args().variant
    }

    /// TS-05/D8 — the `--repo` override for this run, if any. Overrides the INI
    /// `cv_template_path` (flag > INI); absent → the INI value is used.
    pub fn get_repo_override(&self) -> Option<String> {
        self.get_user_input().repo
    }

    /// TS-05/D8 — the `--branch` override for this run, if any. Overrides the INI
    /// `cv_template_ref` (flag > INI); absent → the INI ref (or repo default).
    pub fn get_branch_override(&self) -> Option<String> {
        self.get_user_input().branch
    }

    pub fn _get_date(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self.get_user_input_action_filter_args().date {
            Some(job) => Ok(job),
            None => Err("This filter does not have the 'date' keyword"
                .to_string()
                .into()),
        }
    }

    pub fn get_user_input_save_to_db(&self) -> bool {
        self.get_user_input().save_to_database
    }

    /// The `[db] engine` value, stripped of the surrounding quotes the
    /// documented config format uses (`engine = "postgres"`). Without the
    /// stripping the engine name never matches `postgres`/`sqlite` and the
    /// caller falls through to its catch-all arm.
    pub fn get_user_input_db_engine(&self) -> Result<String, Box<dyn std::error::Error>> {
        let engine = self.get_user_input_vars("db", "engine")?;
        Ok(clean_string_from_quotes(&engine))
    }

    /// The `[db] db_pg_host` value, stripped of the surrounding quotes the
    /// documented config format uses. The value is a passwordless base URL —
    /// the password is spliced in at connection time from the environment.
    pub fn get_user_input_db_url(&self) -> Result<String, &str> {
        self.config
            .get("db", "db_pg_host")
            .map(|url| clean_string_from_quotes(&url))
            .ok_or("Could not get the database engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use configparser::ini::Ini;

    fn dummy_ini() -> Ini {
        let mut ini = Ini::new();
        ini.set("test", "key", Some("value".to_string()));
        ini.set("db", "engine", Some("sqlite".to_string()));
        ini.set("db", "db_file", Some("test.db".to_string()));
        ini
    }

    fn dummy_user_input() -> UserInput {
        UserInput {
            action: UserAction::List(FilterArgs {
                job_title: Some("Dev".to_string()),
                company_name: Some("Company".to_string()),
                quote: Some("Quote".to_string()),
                date: Some("2024-01-01".to_string()),
                variant: Some("senior-devops".to_string()),
            }),
            save_to_database: true,
            view_generated_cv: false,
            dry_run: false,
            config_ini: String::new(),
            engine: "sqlite".to_string(),
            repo: None,
            branch: None,
        }
    }

    fn context() -> AppContext {
        let now = Local.with_ymd_and_hms(2025, 8, 30, 10, 0, 0).unwrap();
        AppContext::new(dummy_ini(), now, dummy_user_input())
    }

    #[test]
    fn test_new_exposes_config_and_user_input() {
        let ctx = context();
        assert_eq!(ctx.config().get("test", "key").unwrap(), "value");
        assert!(ctx.get_user_input_save_to_db());
    }

    #[test]
    fn test_get_user_input_vars_returns_value() {
        let ctx = context();
        let val = ctx.get_user_input_vars("test", "key");
        assert_eq!(val.unwrap(), "value");
    }

    #[test]
    fn test_get_user_input_vars_errors_when_missing() {
        let ctx = context();
        assert!(ctx.get_user_input_vars("missing", "key").is_err());
    }

    #[test]
    fn test_db_accessors_strip_the_quotes_of_the_documented_format() {
        // The documented config format quotes every value; without stripping,
        // the engine never matches "postgres" and the URL keeps its quotes.
        let mut ini = Ini::new();
        ini.set("db", "engine", Some("\"postgres\"".to_string()));
        ini.set(
            "db",
            "db_pg_host",
            Some("\"postgres://rusty_cv@example.invalid/rusty_cv\"".to_string()),
        );
        let ctx = AppContext::new(ini, Local::now(), dummy_user_input());

        assert_eq!(ctx.get_user_input_db_engine().unwrap(), "postgres");
        assert_eq!(
            ctx.get_user_input_db_url().unwrap(),
            "postgres://rusty_cv@example.invalid/rusty_cv"
        );
    }

    #[test]
    fn test_db_accessors_leave_unquoted_values_untouched() {
        let mut ini = Ini::new();
        ini.set("db", "engine", Some("sqlite".to_string()));
        ini.set("db", "db_pg_host", Some("postgres://u@h/d".to_string()));
        let ctx = AppContext::new(ini, Local::now(), dummy_user_input());

        assert_eq!(ctx.get_user_input_db_engine().unwrap(), "sqlite");
        assert_eq!(ctx.get_user_input_db_url().unwrap(), "postgres://u@h/d");
    }

    #[test]
    fn test_get_user_input_db_url_errors_when_missing() {
        let ctx = context();
        assert!(ctx.get_user_input_db_url().is_err());
    }

    #[test]
    fn test_get_today_str_and_year_str() {
        let ctx = context();
        assert!(ctx.get_today_str().contains("Aug"));
        assert_eq!(ctx.get_year_str(), "2025");
        assert_eq!(ctx.get_today_str_yyyy_mm_dd(), "2025-08-30");
    }

    #[test]
    #[allow(clippy::used_underscore_items)]
    fn test_get_job_company_quote_and_date() {
        let ctx = context();
        assert_eq!(ctx.get_job_title().unwrap(), "Dev");
        assert_eq!(ctx.get_company_name().unwrap(), "Company");
        assert_eq!(ctx.get_quote().unwrap(), "Quote");
        assert_eq!(ctx._get_date().unwrap(), "2024-01-01");
        assert_eq!(ctx.get_variant().unwrap(), "senior-devops");
        assert!(ctx.get_user_input_save_to_db());
    }
}
