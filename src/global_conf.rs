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
