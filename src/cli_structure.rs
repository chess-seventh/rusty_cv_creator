use crate::global_conf::GlobalVars;
use crate::user_action::{insert_cv, remove_cv, show_cvs};
use chrono::NaiveDateTime;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(about = "A way to save my CV applications", long_about = None)]
pub struct UserInput {
    /// Choice to save the CV in the database [default: true]
    #[arg(short, long)]
    #[arg(default_value_t = true)]
    pub save_to_database: bool,

    /// Choice to show the generated CV [default: false]
    #[arg(short, long)]
    #[arg(default_value_t = false)]
    pub view_generated_cv: bool,

    /// Choice to generate the dirs or CV [default: true]
    #[arg(short, long)]
    #[arg(default_value_t = true)]
    pub generate_dirs: bool,

    /// Directory of the configuration ini
    #[arg(short, long)]
    #[arg(default_value_t = String::from("~/.config/rusty-cv-creator/rusty-cv-config.ini"))]
    pub config_ini: String,

    /// Database engine (supports only postgresql and sqlite)
    #[arg(short, long)]
    #[arg(default_value_t = String::from("sqlite"))]
    pub db_engine: String,

    /// Filter by date
    #[arg(long, value_parser)]
    pub filter_date: Option<String>,

    /// Filter on any name
    #[arg(long, value_parser)]
    pub filter_name: Option<String>,

    /// Filter on the company
    #[arg(long, value_parser)]
    pub filter_company: Option<String>,

    /// Filter by the job name
    #[arg(long, value_parser)]
    pub filter_job: Option<String>,

    /// What action should run
    #[command(subcommand)]
    pub action: UserAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum UserAction {
    Insert(InsertCV),
    Update(UpdateCV),
    Remove(RemoveCV),
    Show(ShowCV),
    List(ListCV),
}

#[derive(Args, Debug, Clone)]
#[command(about = "Generate and insert a new CV", long_about = None)]
pub struct InsertCV {
    pub company_name: String,
    pub job_title: String,
    pub quote: String,
}

#[derive(Args, Debug, Clone)]
#[command(about = "Update a field of a CV in the database", long_about = None)]
pub struct UpdateCV {}

#[derive(Args, Debug, Clone)]
#[command(about = "Remove a specific CV from the database and remove the directory", long_about = None)]
pub struct RemoveCV {}

#[derive(Args, Debug, Clone)]
#[command(about = "Show a specific CV that is in the database", long_about = None)]
pub struct ShowCV {}

#[derive(Args, Debug, Clone)]
#[command(about = "List all CVs that are in the database", long_about = None)]
pub struct ListCV {}

pub fn match_user_action() -> String {
    let input = GlobalVars::get_user_input();
    let filters = match_user_filters(input.clone());
    // let db_engine = input.db_engine;
    println!("{filters:?}");
    match input.action {
        UserAction::Insert(_insert) => insert_cv(),
        UserAction::Show(_show) => show_cvs(),
        UserAction::Remove(_remove) => {
            remove_cv();
            "removed the selected cv!".to_string()
        }
        UserAction::List(_list) => {
            todo!()
        }
        UserAction::Update(_update) => {
            todo!();
        }
    }
}

pub fn match_user_filters(user_input: UserInput) -> UserFilters {
    let mut user_filters = UserFilters::default();
    user_filters.create(user_input)
}

#[derive(Debug, Clone, Default)]
pub struct UserFilters {
    pub date: Option<NaiveDateTime>,
    pub name: Option<String>,
    pub job: Option<String>,
    pub company: Option<String>,
}

impl UserFilters {
    pub fn create(&mut self, user_input: UserInput) -> Self {
        UserFilters {
            date: self.parse_date(user_input.filter_date),
            name: user_input.filter_name,
            job: user_input.filter_job,
            company: user_input.filter_company,
        }
    }

    fn parse_date(&mut self, filter_date: Option<String>) -> Option<NaiveDateTime> {
        filter_date.as_ref()?;

        self.date = match chrono::NaiveDateTime::parse_from_str(&filter_date.clone().unwrap(), "%B")
        {
            Ok(d) => Some(d),
            Err(_) => {
                match chrono::NaiveDateTime::parse_from_str(&filter_date.clone().unwrap(), "%b") {
                    Ok(d) => Some(d),
                    Err(_) => {
                        match chrono::NaiveDateTime::parse_from_str(&filter_date.unwrap(), "%m") {
                            Ok(d) => Some(d),
                            _ => panic!("could not parse the date"),
                        }
                    }
                }
            }
        };
        self.date
    }
}
