use chrono::NaiveDateTime;
use clap::{Args, Parser, Subcommand};
use crate::helpers::parse_date;
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
#[command(about = "List all CVs that are in the database", long_about = None)]
pub struct ListCV {
}

#[derive(Args, Debug, Clone)]
#[command(about = "Show a specific CV that is in the database", long_about = None)]
pub struct ShowCV {
    pub company_name: Option<String>,
    pub job_title: Option<String>,
    pub quote: Option<String>,
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
pub struct UpdateCV {
    pub job_title: String,
    pub company_name: String,
    pub quote: Option<String>,
}

#[derive(Args, Debug, Clone)]
#[command(about = "Remove a specific CV from the database and remove the directory", long_about = None)]
pub struct RemoveCV {
    pub job_title: Option<String>,
    pub company_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserFilters {
    pub date: Option<NaiveDateTime>,
    pub name: Option<String>,
    pub job: Option<String>,
    pub company: Option<String>,
}

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
    UserFilters {
        date : parse_date(user_input.filter_date),
        name : user_input.filter_name,
        job : user_input.filter_job,
        company : user_input.filter_company,
    }
}

