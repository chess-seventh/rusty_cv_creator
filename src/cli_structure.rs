use crate::global_conf::GLOBAL_VAR;
use crate::user_action::{insert_cv, remove_cv};
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(about = "A way to save my CV applications", long_about = None)]
pub struct UserInput {
    /// What action should run
    #[command(subcommand)]
    pub action: UserAction,

    /// Choice to save the CV in the database [default: true]
    #[arg(short, long)]
    pub save_to_database: Option<bool>,

    /// Choice to show the generated CV [default: false]
    #[arg(short, long)]
    pub view_generated_cv: Option<bool>,

    /// Dry Run the whole process without creating or showing anything [default: false]
    #[arg(short, long)]
    pub dry_run: Option<bool>,

    /// Directory of the configuration ini
    #[arg(short, long, default_value_t = String::from("~/.config/rusty-cv-creator/rusty-cv-config.ini"))]
    pub config_ini: String,

    /// Database engine (supports only postgresql and sqlite)
    #[arg(short, long, default_value_t = String::from("sqlite"))]
    pub engine: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum UserAction {
    Insert(InsertArgs),
    Update(FilterArgs),
    Remove(FilterArgs),
    List(FilterArgs),
}

#[derive(Args, Debug, Clone, Default)]
#[command(about = "Generate and insert a new CV", long_about = None)]
pub struct InsertArgs {
    pub company_name: String,
    pub job_title: String,
    pub quote: String,
}

#[derive(Args, Debug, Clone, Default)]
#[command(about = "Update a field of a CV in the database", long_about = None)]
pub struct FilterArgs {
    #[arg(short, long)]
    pub job: Option<String>,

    #[arg(short, long)]
    pub company: Option<String>,

    #[arg(short, long)]
    pub date: Option<String>,
}

pub fn match_user_action() -> String {
    let input = GLOBAL_VAR.get().unwrap().get_user_input();
    match input.action {
        UserAction::Insert(_insert) => match insert_cv() {
            Ok(s) => s,
            Err(e) => panic!("{e:?}"),
        },

        UserAction::Remove(filters) => {
            remove_cv(&filters);
            let out = format!("filter args for LIST: {filters:?}");
            println!("{out:?}");
            out
        }
        UserAction::List(filters) => {
            let out = format!("filter args for LIST: {filters:?}");
            println!("{out:?}");
            out
        }
        UserAction::Update(filters) => {
            let out = format!("filter args for UPDATE: {filters:?}");
            println!("{out:?}");
            out
        }
    }
}

#[derive(Debug, Clone, Default, Parser)]
pub struct UserFilters {
    pub job: Option<String>,
    pub company: Option<String>,
    pub date: Option<String>,
}

// TODO: this function should parse the user input when filtering or listing cv.
// TODO Fix this should return ParseError when things do not work.
#[allow(dead_code)]
fn parse_date(input: &str) -> Result<String, String> {
    let formats = [
        "%Y",       // Year
        "%Y-%m",    // Year-Month
        "%Y-%m-%d", // Year-Month-Day
        "%B",       // Full month name
        "%b",       // Abbreviated month name
    ];

    for format in &formats {
        if let Ok(parsed) = NaiveDate::parse_from_str(input, format) {
            return Ok(parsed.to_string());
        }
    }

    Err(format!("Failed to parse date: {input}"))
}
