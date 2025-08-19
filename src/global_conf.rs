use crate::{cli_structure::UserAction, UserInput};
use chrono::{DateTime, Local};
use configparser::ini::Ini;
use once_cell::sync::OnceCell;

pub static GLOBAL_VAR: OnceCell<GlobalVars> = OnceCell::new();

#[derive(Debug, Clone)]
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

    pub fn get_config(&self) -> Ini {
        self.config
            .get()
            .expect("Config.ini file not initialized")
            .clone()
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
        self.get_user_input().action
    }

    pub fn get_user_job_title(&self) -> String {
        match self.get_user_input_action() {
            UserAction::Insert(insert) => insert.job_title,
            _ => panic!("Could not get the job title"),
        }
    }

    pub fn get_user_input_company_name(&self) -> String {
        match self.get_user_input_action() {
            UserAction::Insert(insert) => insert.company_name,
            _ => panic!("Could not get the company name"),
        }
    }

    pub fn get_user_input_quote(&self) -> String {
        match self.get_user_input_action() {
            UserAction::Insert(insert) => insert.quote,
            _ => panic!("Could not get the quote"),
        }
    }

    pub fn get_user_input_save_to_db(&self) -> bool {
        self.get_user_input().save_to_database.unwrap_or(true)
    }

    pub fn get_user_input_db_engine(&self) -> String {
        self.get_config()
            .get("db", "engine")
            .expect("Could not get the database engine")
    }

    pub fn get_user_input_db_url(&self) -> String {
        self.get_config()
            .get("db", "db_pg_host")
            .expect("Could not get the database engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_structure::{FilterArgs, InsertArgs, UserAction};
    use chrono::{Datelike, TimeZone};
    use configparser::ini::Ini;
    // use serial_test::serial;

    fn create_test_user_input() -> UserInput {
        UserInput {
            action: UserAction::Insert(InsertArgs {
                company_name: "Test Company".to_string(),
                job_title: "Software Engineer".to_string(),
                quote: "Test quote".to_string(),
            }),
            save_to_database: Some(true),
            view_generated_cv: Some(false),
            dry_run: Some(false),
            config_ini: "test.ini".to_string(),
            engine: "sqlite".to_string(),
        }
    }

    fn create_test_config() -> Ini {
        let mut config = Ini::new();
        config.set("db", "engine", Some("sqlite".to_string()));
        config.set("db", "db_pg_host", Some("postgresql://test".to_string()));
        config
    }

    #[test]
    fn test_globalvars_new() {
        let global_vars = GlobalVars::new();
        assert!(global_vars.config.get().is_none());
        assert!(global_vars.today.get().is_none());
        assert!(global_vars.user_input.get().is_none());
    }

    #[test]
    fn test_globalvars_set_all() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local
            .with_ymd_and_hms(2023, 8, 19, 10, 0, 0)
            .unwrap();
        let user_input = create_test_user_input();

        let result = global_vars.set_all(config, today, user_input.clone());

        assert!(global_vars.config.get().is_some());
        assert!(global_vars.today.get().is_some());
        assert!(global_vars.user_input.get().is_some());
        assert_eq!(result.get_user_input().engine, "sqlite");
    }

    #[test]
    fn test_get_config() {
        let global_vars = GlobalVars::new();
        let mut config = Ini::new();
        config.set("test", "key", Some("value".to_string()));
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let retrieved_config = global_vars.get_config();
        assert_eq!(retrieved_config.get("test", "key").unwrap(), "value");
    }

    #[test]
    #[should_panic(expected = "Config.ini file not initialized")]
    fn test_get_config_uninitialized() {
        let global_vars = GlobalVars::new();
        global_vars.get_config();
    }

    #[test]
    fn test_get_today() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local
            .with_ymd_and_hms(2023, 8, 19, 15, 30, 45)
            .unwrap();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let retrieved_today = global_vars.get_today();
        assert_eq!(retrieved_today.year_ce(), (true, 2023));
        assert_eq!(retrieved_today.month(), 8);
        assert_eq!(retrieved_today.day(), 19);
    }

    #[test]
    fn test_get_today_str() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local
            .with_ymd_and_hms(2023, 8, 19, 15, 30, 45)
            .unwrap();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let today_str = global_vars.get_today_str();
        assert!(today_str.contains("19"));
        assert!(today_str.contains("Aug"));
        assert!(today_str.contains("2023"));
    }

    #[test]
    fn test_get_today_str_yyyy_mm_dd() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local
            .with_ymd_and_hms(2023, 8, 19, 15, 30, 45)
            .unwrap();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let today_str = global_vars.get_today_str_yyyy_mm_dd();
        assert_eq!(today_str, "2023-08-19");
    }

    #[test]
    fn test_get_year_str() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local
            .with_ymd_and_hms(2023, 8, 19, 15, 30, 45)
            .unwrap();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let year_str = global_vars.get_year_str();
        assert_eq!(year_str, "2023");
    }

    #[test]
    fn test_get_user_input() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input.clone());

        let retrieved_input = global_vars.get_user_input();
        assert_eq!(retrieved_input.engine, "sqlite");
        assert_eq!(retrieved_input.save_to_database, Some(true));
    }

    #[test]
    #[should_panic(expected = "UserInput not initialized")]
    fn test_get_user_input_uninitialized() {
        let global_vars = GlobalVars::new();
        global_vars.get_user_input();
    }

    #[test]
    fn test_get_user_input_action() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let action = global_vars.get_user_input_action();
        match action {
            UserAction::Insert(insert_args) => {
                assert_eq!(insert_args.company_name, "Test Company");
                assert_eq!(insert_args.job_title, "Software Engineer");
                assert_eq!(insert_args.quote, "Test quote");
            }
            _ => panic!("Expected Insert action"),
        }
    }

    #[test]
    fn test_get_user_job_title() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let job_title = global_vars.get_user_job_title();
        assert_eq!(job_title, "Software Engineer");
    }

    #[test]
    #[should_panic(expected = "Could not get the job title")]
    fn test_get_user_job_title_wrong_action() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let mut user_input = create_test_user_input();
        user_input.action = UserAction::List(FilterArgs {
            job: None,
            company: None,
            date: None,
        });

        global_vars.set_all(config, today, user_input);
        global_vars.get_user_job_title();
    }

    #[test]
    fn test_get_user_input_company_name() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let company_name = global_vars.get_user_input_company_name();
        assert_eq!(company_name, "Test Company");
    }

    #[test]
    #[should_panic(expected = "Could not get the company name")]
    fn test_get_user_input_company_name_wrong_action() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let mut user_input = create_test_user_input();
        user_input.action = UserAction::Update(FilterArgs {
            job: None,
            company: None,
            date: None,
        });

        global_vars.set_all(config, today, user_input);
        global_vars.get_user_input_company_name();
    }

    #[test]
    fn test_get_user_input_quote() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let quote = global_vars.get_user_input_quote();
        assert_eq!(quote, "Test quote");
    }

    #[test]
    #[should_panic(expected = "Could not get the quote")]
    fn test_get_user_input_quote_wrong_action() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let mut user_input = create_test_user_input();
        user_input.action = UserAction::Remove(FilterArgs {
            job: Some("test".to_string()),
            company: None,
            date: None,
        });

        global_vars.set_all(config, today, user_input);
        global_vars.get_user_input_quote();
    }

    #[test]
    fn test_get_user_input_save_to_db_some_true() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let mut user_input = create_test_user_input();
        user_input.save_to_database = Some(true);

        global_vars.set_all(config, today, user_input);

        let save_to_db = global_vars.get_user_input_save_to_db();
        assert_eq!(save_to_db, true);
    }

    #[test]
    fn test_get_user_input_save_to_db_some_false() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let mut user_input = create_test_user_input();
        user_input.save_to_database = Some(false);

        global_vars.set_all(config, today, user_input);

        let save_to_db = global_vars.get_user_input_save_to_db();
        assert_eq!(save_to_db, false);
    }

    #[test]
    fn test_get_user_input_save_to_db_none() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let mut user_input = create_test_user_input();
        user_input.save_to_database = None;

        global_vars.set_all(config, today, user_input);

        let save_to_db = global_vars.get_user_input_save_to_db();
        assert_eq!(save_to_db, true); // Default is true
    }

    #[test]
    fn test_get_user_input_db_engine() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let db_engine = global_vars.get_user_input_db_engine();
        assert_eq!(db_engine, "sqlite");
    }

    #[test]
    #[should_panic(expected = "Could not get the database engine")]
    fn test_get_user_input_db_engine_missing() {
        let global_vars = GlobalVars::new();
        let config = Ini::new();
        // Don't set the db engine
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);
        global_vars.get_user_input_db_engine();
    }

    #[test]
    fn test_get_user_input_db_url() {
        let global_vars = GlobalVars::new();
        let config = create_test_config();
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);

        let db_url = global_vars.get_user_input_db_url();
        assert_eq!(db_url, "postgresql://test");
    }

    #[test]
    #[should_panic(expected = "Could not get the database engine")]
    fn test_get_user_input_db_url_missing() {
        let global_vars = GlobalVars::new();
        let config = Ini::new();
        // Don't set the db_pg_host
        let today = chrono::Local::now();
        let user_input = create_test_user_input();

        global_vars.set_all(config, today, user_input);
        global_vars.get_user_input_db_url();
    }
}
