use crate::{
    cli_structure::{FilterArgs, UserAction},
    UserInput,
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
    get_global_var().get_config()
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

    pub fn get_config(&self) -> Ini {
        self.config
            .get()
            .expect("Config.ini file not initialized")
            .clone()
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

    pub fn get_user_input_db_url(&self) -> String {
        self.get_config()
            .get("db", "db_pg_host")
            .expect("Could not get the database engine")
    }
}
