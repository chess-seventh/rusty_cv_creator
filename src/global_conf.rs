use crate::{cli_structure::UserAction, UserInput};
use configparser::ini::Ini;
use once_cell::sync::OnceCell;
use chrono::{DateTime, Local};


pub static CONFIG: OnceCell<Ini> = OnceCell::new();

pub static TODAY: OnceCell<DateTime<Local>> = OnceCell::new();

pub static USER_INPUT: OnceCell<UserInput> = OnceCell::new();

pub struct GlobalVars { }


impl GlobalVars {
    pub fn get_config() -> Ini {
        CONFIG.get().expect("Config.ini file not initialized").clone()
    }

    pub fn get_today() -> DateTime<Local> {
        *TODAY.get_or_init(Local::now)
    }

    pub fn get_today_str() -> String {
        let today = GlobalVars::get_today();
        today.format("%e-%b-%Y").to_string()
    }

    pub fn get_user_input() -> UserInput {
        USER_INPUT.get().expect("UserInput not initialized").clone()
    }

    pub fn get_user_action() -> UserAction {
        let user_input = GlobalVars::get_user_input();
        user_input.action
    }

    pub fn get_user_job_title() -> String {
        match GlobalVars::get_user_action() {
            UserAction::Insert(insert) => insert.job_title,
            _ => panic!("Could not get the job title")
        }
    }

    pub fn get_user_company_name() -> String {
        match GlobalVars::get_user_action() {
            UserAction::Insert(insert) => insert.company_name,
            _ => panic!("Could not get the company name")
        }
    }

    pub fn get_user_quote() -> String {
        match GlobalVars::get_user_action() {
            UserAction::Insert(insert) => insert.quote,
            _ => panic!("Could not get the quote")
        }
    }

    pub fn get_user_save_to_db() -> bool {
        let user_input = GlobalVars::get_user_input();
        user_input.save_to_database
    }

    pub fn get_db_engine() -> String {
        let config = GlobalVars::get_config();
        config.get("db", "db_engine").expect("Could not get the database engine")
    }

    pub fn get_db_url() -> String {
        let config = GlobalVars::get_config();
        config.get("db", "db_pg_host").expect("Could not get the database engine")
    }
}
