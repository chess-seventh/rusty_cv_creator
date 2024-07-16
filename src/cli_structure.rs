use clap::{Args, Parser, Subcommand};
use crate::run_insert;
use crate::database::{remove_cv, show_cvs};

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
    pub filter_word: Option<String>,
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

pub fn match_user_action(input: UserInput) -> String {
    match input.action {
            UserAction::Insert(insert) => {
                run_insert(input.save_to_database, &insert)
            }
            UserAction::Show(_show) => {
                show_cvs()
            }
            UserAction::Remove(_remove) => {
                remove_cv()
            }
            UserAction::Update(_update) => {
                todo!();
            }
            UserAction::List(_list) => {
                todo!();
            }
    }
}

