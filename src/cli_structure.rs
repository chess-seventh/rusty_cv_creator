use chrono::NaiveDateTime;
use clap::{Args, Parser, Subcommand};
use crate::user_action::{insert_cv, remove_cv, show_cvs};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(about = "A way to save my CV applications", long_about = None)]
pub struct UserInput {
    /// Choice to save the CV in the database [default: true]
    #[arg(short, long)]
    #[arg(default_value_t = true)]
    pub save_to_database: bool,

    /// Choice to generate the dirs or CV [default: false]
    #[arg(short, long)]
    #[arg(default_value_t = false)]
    pub generate_dirs: bool,

    /// Directory of the configuration ini
    #[arg(short, long)]
    #[arg(default_value_t = String::from("~/.config/rusty-cv-creator/rusty-cv-config.ini"))]
    pub config_ini: String,

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
pub struct UpdateCV { }

#[derive(Args, Debug, Clone)]
#[command(about = "Remove a specific CV from the database and remove the directory", long_about = None)]
pub struct RemoveCV { }

#[derive(Args, Debug, Clone)]
#[command(about = "Show a specific CV that is in the database", long_about = None)]
pub struct ShowCV { }

#[derive(Args, Debug, Clone)]
#[command(about = "List all CVs that are in the database", long_about = None)]
pub struct ListCV { }

pub fn match_user_action(input: UserInput) -> String {
    let filters = match_user_filters(input.clone());
    println!("{filters:?}");
    match input.action {
            UserAction::Insert(insert) => {
                insert_cv(input.save_to_database, &insert)
            }
            UserAction::Show(_show) => {
                show_cvs()
            }
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
    let user_filters = UserFilters::default();
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
     pub fn create(&self, user_input: UserInput) -> Self {

        let Some(f_date) = self.parse_date(user_input.filter_date) else {
            panic!("could not parse the date")
        };

        UserFilters {
            date : Some(f_date),
            name : user_input.filter_name,
            job : user_input.filter_job,
            company : user_input.filter_company,
        }
    }

    fn parse_date(&self, filter_date: Option<String>) -> Option<NaiveDateTime> {
        // TODO make sure that we parse all possibilities, else return to use the proper date formats
        // https://docs.rs/chrono/latest/chrono/format/strftime/index.html

        filter_date.as_ref()?;

        match chrono::NaiveDateTime::parse_from_str(&filter_date.clone().unwrap(), "%B") {
            Ok(d) => Some(d),
            Err(_) => match chrono::NaiveDateTime::parse_from_str(&filter_date.clone().unwrap(), "%b") {
                Ok(d) => Some(d),
                Err(_) => match chrono::NaiveDateTime::parse_from_str(&filter_date.unwrap(), "%m") {
                    Ok(d) => Some(d),
                    Err(_) => None,
                }
            }
        }

    } 

}
