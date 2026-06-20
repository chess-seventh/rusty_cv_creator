use crate::{
    UserInput,
    cli_structure::{FilterArgs, UserAction},
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
            UserAction::Insert(filter_args)
            | UserAction::Remove(filter_args)
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

    pub fn get_user_input_db_engine(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.get_user_input_vars("db", "engine")
    }

    pub fn get_user_input_db_url(&self) -> Result<String, &str> {
        self.config
            .get("db", "db_pg_host")
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
            action: UserAction::Insert(FilterArgs {
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
