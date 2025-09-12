use crate::{
    UserInput,
    cli_structure::{FilterArgs, UserAction},
};
use chrono::{DateTime, Local};
use configparser::ini::Ini;
use once_cell::sync::OnceCell;

pub static GLOBAL_VAR: OnceCell<GlobalVars> = OnceCell::new();

pub fn get_global_var() -> GlobalVars {
    match GLOBAL_VAR.get() {
        Some(gvar) => gvar.clone(),
        None => panic!("GlobalVar Get didn't work"),
    }
}

pub fn _get_global_var_config() -> Ini {
    get_global_var().get_config().unwrap()
}

pub fn get_global_var_config_db_path() -> Result<String, Box<dyn std::error::Error>> {
    let gvar = get_global_var();
    gvar.get_user_input_vars("db", "db_path")
}

pub fn get_global_var_config_db_file() -> Result<String, Box<dyn std::error::Error>> {
    let gvar = get_global_var();
    gvar.get_user_input_vars("db", "db_file")
}

#[derive(Debug, Clone, Default)]
pub struct GlobalVars {
    pub config: OnceCell<Ini>,
    pub today: OnceCell<DateTime<Local>>,
    pub user_input: OnceCell<UserInput>,
}

impl GlobalVars {
    pub fn new() -> Self {
        GlobalVars {
            config: OnceCell::new(),
            today: OnceCell::new(),
            user_input: OnceCell::new(),
        }
    }

    pub fn set_all(&self, config: Ini, today: DateTime<Local>, user_input: UserInput) -> &Self {
        self.config
            .set(config)
            .expect("Coulnd't set config in GlobalVars");
        self.today
            .set(today)
            .expect("Couldn't set today in GlobalVars");
        self.user_input
            .set(user_input)
            .expect("Couldn't set user_input in GlobalVars");

        self
    }

    pub fn get_config(&self) -> Result<Ini, &str> {
        self.config.get().cloned().ok_or("Cnofig not initialized")
    }

    pub fn get_user_input_vars(
        &self,
        section: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let conf = self
            .config
            .get()
            .unwrap_or_else(|| panic!("Failed getting the Config INI"));

        conf.get(section, key)
            .ok_or(format!("Could not get {section:} {key:}").into())
    }

    pub fn get_today(&self) -> &DateTime<Local> {
        self.today.get().unwrap()
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
        self.user_input
            .get()
            .expect("UserInput not initialized")
            .clone()
    }

    pub fn get_user_input_action(&self) -> UserAction {
        self.user_input.get().unwrap().clone().action
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
        // self.get_config().get("db", "engine")
        // .expect("Could not get the database engine")
    }

    pub fn get_user_input_db_url(&self) -> Result<String, &str> {
        self.get_config()
            .unwrap()
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
            }),
            save_to_database: true,
            view_generated_cv: false,
            dry_run: false,
            config_ini: String::new(),
            engine: "sqlite".to_string(),
        }
    }

    #[test]
    fn test_globalvars_new_creates_empty_cells() {
        let gvars = GlobalVars::new();
        assert!(gvars.config.get().is_none());
        assert!(gvars.today.get().is_none());
        assert!(gvars.user_input.get().is_none());
    }

    #[test]
    fn test_set_all_sets_cells() {
        let gvars = GlobalVars::new();
        let now = Local.with_ymd_and_hms(2025, 8, 30, 10, 0, 0).unwrap();
        let ui = dummy_user_input();
        gvars.set_all(dummy_ini(), now, ui.clone());
        assert!(gvars.config.get().is_some());
        assert!(gvars.today.get().is_some());
        assert!(gvars.user_input.get().is_some());
    }

    #[test]
    fn test_get_user_input_vars_returns_value() {
        let gvars = GlobalVars::new();
        let now = Local.with_ymd_and_hms(2025, 8, 30, 10, 0, 0).unwrap();
        gvars.set_all(dummy_ini(), now, dummy_user_input());
        let val = gvars.get_user_input_vars("test", "key");
        assert_eq!(val.unwrap(), "value");
    }

    #[test]
    fn test_get_today_str_and_year_str() {
        let gvars = GlobalVars::new();
        let now = Local.with_ymd_and_hms(2025, 8, 30, 10, 0, 0).unwrap();
        gvars.set_all(dummy_ini(), now, dummy_user_input());
        assert!(gvars.get_today_str().contains("Aug"));
        assert_eq!(gvars.get_year_str(), "2025");
    }

    #[test]
    #[allow(clippy::used_underscore_items)]
    fn test_get_job_company_quote_and_date() {
        let gvars = GlobalVars::new();
        let now = Local.with_ymd_and_hms(2025, 8, 30, 10, 0, 0).unwrap();
        let ui = dummy_user_input();
        gvars.set_all(dummy_ini(), now, ui.clone());

        assert_eq!(gvars.get_job_title().unwrap(), "Dev");
        assert_eq!(gvars.get_company_name().unwrap(), "Company");
        assert_eq!(gvars.get_quote().unwrap(), "Quote");
        assert_eq!(gvars._get_date().unwrap(), "2024-01-01");
        assert!(gvars.get_user_input_save_to_db());
    }
}
