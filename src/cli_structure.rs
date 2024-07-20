use chrono::NaiveDateTime;
use clap::{Args, Parser, Subcommand};
use crate::helpers::parse_date;
use crate::user_action::{insert_cv, remove_cv, show_cvs};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct UserInput {
    #[arg(short, long)]
    #[arg(default_value_t = true)]
    pub save_to_database: bool,

    #[arg(short, long)]
    #[arg(default_value_t = String::from("~/.config/rusty-cv-creator/rusty-cv-config.ini"))]
    pub config_ini: String,

    #[arg(long, value_parser)]
    pub filter_date: Option<String>,

    #[arg(long, value_parser)]
    pub filter_name: Option<String>,

    #[arg(long, value_parser)]
    pub filter_company: Option<String>,

    #[arg(long, value_parser)]
    pub filter_job: Option<String>,

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
pub struct ListCV {
}

#[derive(Args, Debug, Clone)]
pub struct ShowCV {
    pub company_name: Option<String>,
    pub job_title: Option<String>,
    pub quote: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct InsertCV {
    pub company_name: String,
    pub job_title: String,
    pub quote: String,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateCV {
    pub job_title: String,
    pub company_name: String,
    pub quote: Option<String>,
}

#[derive(Args, Debug, Clone)]
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
                remove_cv()
            }
            UserAction::List(_list) => {
                todo!()
                //
                // if list.filter_date.is_some() && list.filter_word.is_some() {
                //     let parsed_date = parse_date(&list.filter_date.unwrap());
                //     list_cvs(parsed_date, list.filter_word.unwrap());
                //     "hello".to_string()
                // } else if list.filter_date.is_some() {
                //     let parsed_date = parse_date(&list.filter_date.unwrap());
                //     list_cvs_by_date(parsed_date);
                //     "hello".to_string()
                // } else if list.filter_word.is_some() {
                //     list_cvs_by_word(list.filter_word.unwrap());
                //     "hello".to_string()
                // } else {
                //     "bye".to_string()
                // }
            }
            UserAction::Update(_update) => {
                todo!();
            }
    }
}

pub fn match_user_filters(user_input: UserInput) -> UserFilters {
    UserFilters {
        date : Some(parse_date(user_input.filter_date)),
        name : user_input.filter_name,
        job : user_input.filter_job,
        company : user_input.filter_company,
    }
}

