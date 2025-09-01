use crate::{cv_insert::insert_cv, user_action::remove_cv};
use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(about = "Generate and save CV based on a Latex template", long_about = None)]
pub struct UserInput {
    /// What action should run
    #[command(subcommand)]
    pub action: UserAction,

    /// Choice to save the CV in the database [default: false]
    #[arg(short, long, default_value_t = false)]
    pub save_to_database: bool,

    /// Choice to show the generated CV [default: false]
    #[arg(short, long, default_value_t = false)]
    pub view_generated_cv: bool,

    /// Dry Run the whole process without creating or showing anything [default: false]
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Directory of the configuration ini
    #[arg(short, long, default_value_t = String::from("~/.config/rusty-cv-creator/rusty-cv-config.ini"))]
    pub config_ini: String,

    /// Database engine (supports only postgresql and sqlite)
    #[arg(short, long, default_value_t = String::from("sqlite"))]
    pub engine: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum UserAction {
    #[command(about = "Insert CV", long_about = None)]
    Insert(FilterArgs),

    #[command(about = "Update CV", long_about = None)]
    Update(FilterArgs),

    #[command(about = "Remove CV", long_about = None)]
    Remove(FilterArgs),

    #[command(about = "List CVs", long_about = None)]
    List(FilterArgs),
}

#[derive(Args, Debug, Clone, Default)]
pub struct FilterArgs {
    #[arg(short, long)]
    pub job_title: Option<String>,

    #[arg(short, long)]
    pub company_name: Option<String>,

    #[arg(short, long)]
    pub quote: Option<String>,

    #[arg(short, long)]
    pub date: Option<String>,
}

pub fn match_user_action(user_input: UserInput) -> String {
    match user_input.action {
        UserAction::Insert(_insert_args) => match insert_cv() {
            Ok(s) => s,
            Err(e) => panic!("{e:?}"),
        },

        UserAction::Remove(filters) => {
            let _ = remove_cv(&filters);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_full_date() {
        let r = parse_date("2024-01-01");
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), "2024-01-01");
    }

    #[test]
    fn test_parse_date_invalid() {
        let r = parse_date("badstring");
        assert!(r.is_err());
    }

    #[test]
    fn test_filter_args_default() {
        let args = FilterArgs::default();
        assert!(args.job_title.is_none());
        assert!(args.company_name.is_none());
        assert!(args.quote.is_none());
        assert!(args.date.is_none());
    }
}
