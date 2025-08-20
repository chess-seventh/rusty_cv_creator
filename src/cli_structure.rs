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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_input_default_values() {
        // Test that the default values are correctly set
        // Note: This test is for the structure, clap testing is more complex
        let insert_args = InsertArgs {
            company_name: "Test Company".to_string(),
            job_title: "Software Engineer".to_string(),
            quote: "Great opportunity".to_string(),
        };

        assert_eq!(insert_args.company_name, "Test Company");
        assert_eq!(insert_args.job_title, "Software Engineer");
        assert_eq!(insert_args.quote, "Great opportunity");
    }

    #[test]
    fn test_insert_args_creation() {
        let args = InsertArgs {
            company_name: "ACME Corp".to_string(),
            job_title: "Senior Developer".to_string(),
            quote: "Innovative solutions".to_string(),
        };

        assert_eq!(args.company_name, "ACME Corp");
        assert_eq!(args.job_title, "Senior Developer");
        assert_eq!(args.quote, "Innovative solutions");
    }

    #[test]
    fn test_filter_args_creation_all_some() {
        let args = FilterArgs {
            job: Some("Developer".to_string()),
            company: Some("ACME".to_string()),
            date: Some("2023-08-19".to_string()),
        };

        assert_eq!(args.job, Some("Developer".to_string()));
        assert_eq!(args.company, Some("ACME".to_string()));
        assert_eq!(args.date, Some("2023-08-19".to_string()));
    }

    #[test]
    fn test_filter_args_creation_all_none() {
        let args = FilterArgs {
            job: None,
            company: None,
            date: None,
        };

        assert_eq!(args.job, None);
        assert_eq!(args.company, None);
        assert_eq!(args.date, None);
    }

    #[test]
    fn test_filter_args_creation_partial() {
        let args = FilterArgs {
            job: Some("Engineer".to_string()),
            company: None,
            date: Some("2023".to_string()),
        };

        assert_eq!(args.job, Some("Engineer".to_string()));
        assert_eq!(args.company, None);
        assert_eq!(args.date, Some("2023".to_string()));
    }

    #[test]
    fn test_user_action_insert() {
        let insert_args = InsertArgs {
            company_name: "Test Co".to_string(),
            job_title: "Developer".to_string(),
            quote: "Test quote".to_string(),
        };
        let action = UserAction::Insert(insert_args.clone());

        match action {
            UserAction::Insert(args) => {
                assert_eq!(args.company_name, "Test Co");
                assert_eq!(args.job_title, "Developer");
                assert_eq!(args.quote, "Test quote");
            }
            _ => panic!("Expected Insert variant"),
        }
    }

    #[test]
    fn test_user_action_update() {
        let filter_args = FilterArgs {
            job: Some("Developer".to_string()),
            company: None,
            date: None,
        };
        let action = UserAction::Update(filter_args.clone());

        match action {
            UserAction::Update(args) => {
                assert_eq!(args.job, Some("Developer".to_string()));
                assert_eq!(args.company, None);
                assert_eq!(args.date, None);
            }
            _ => panic!("Expected Update variant"),
        }
    }

    #[test]
    fn test_user_action_remove() {
        let filter_args = FilterArgs {
            job: None,
            company: Some("ACME".to_string()),
            date: None,
        };
        let action = UserAction::Remove(filter_args.clone());

        match action {
            UserAction::Remove(args) => {
                assert_eq!(args.job, None);
                assert_eq!(args.company, Some("ACME".to_string()));
                assert_eq!(args.date, None);
            }
            _ => panic!("Expected Remove variant"),
        }
    }

    #[test]
    fn test_user_action_list() {
        let filter_args = FilterArgs {
            job: None,
            company: None,
            date: Some("2023-08-19".to_string()),
        };
        let action = UserAction::List(filter_args.clone());

        match action {
            UserAction::List(args) => {
                assert_eq!(args.job, None);
                assert_eq!(args.company, None);
                assert_eq!(args.date, Some("2023-08-19".to_string()));
            }
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_user_filters_default() {
        let filters = UserFilters::default();
        assert_eq!(filters.job, None);
        assert_eq!(filters.company, None);
        assert_eq!(filters.date, None);
    }

    #[test]
    fn test_user_filters_creation() {
        let filters = UserFilters {
            job: Some("Engineer".to_string()),
            company: Some("TechCorp".to_string()),
            date: Some("2023".to_string()),
        };

        assert_eq!(filters.job, Some("Engineer".to_string()));
        assert_eq!(filters.company, Some("TechCorp".to_string()));
        assert_eq!(filters.date, Some("2023".to_string()));
    }

    // Tests for the parse_date function
    #[test]
    fn test_parse_date_full_date() {
        let result = parse_date("2023-08-19");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2023-08-19");
    }

    #[test]
    fn test_parse_date_invalid_format() {
        let result = parse_date("invalid-date");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse date: invalid-date");
    }

    #[test]
    fn test_parse_date_empty_string() {
        let result = parse_date("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse date: ");
    }

    #[test]
    fn test_parse_date_invalid_year() {
        let result = parse_date("99999");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse date: 99999");
    }

    #[test]
    fn test_parse_date_invalid_month() {
        let result = parse_date("2023-13");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse date: 2023-13");
    }

    #[test]
    fn test_parse_date_invalid_day() {
        let result = parse_date("2023-02-30");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse date: 2023-02-30");
    }

    #[test]
    fn test_parse_date_case_sensitivity() {
        let result = parse_date("AUGUST");
        assert!(result.is_err()); // chrono is case-sensitive
        assert_eq!(result.unwrap_err(), "Failed to parse date: AUGUST");
    }

    // Test Debug trait implementations
    #[test]
    fn test_debug_implementations() {
        let insert_args = InsertArgs {
            company_name: "Test".to_string(),
            job_title: "Dev".to_string(),
            quote: "Quote".to_string(),
        };
        let debug_str = format!("{insert_args:?}");
        assert!(debug_str.contains("Test"));
        assert!(debug_str.contains("Dev"));
        assert!(debug_str.contains("Quote"));

        let filter_args = FilterArgs {
            job: Some("Dev".to_string()),
            company: None,
            date: None,
        };
        let debug_str = format!("{filter_args:?}");
        assert!(debug_str.contains("Dev"));

        let user_filters = UserFilters {
            job: Some("Engineer".to_string()),
            company: None,
            date: None,
        };
        let debug_str = format!("{user_filters:?}");
        assert!(debug_str.contains("Engineer"));
    }

    // Test Clone trait implementations
    #[test]
    fn test_clone_implementations() {
        let insert_args = InsertArgs {
            company_name: "Test".to_string(),
            job_title: "Dev".to_string(),
            quote: "Quote".to_string(),
        };
        let cloned = insert_args.clone();
        assert_eq!(insert_args.company_name, cloned.company_name);
        assert_eq!(insert_args.job_title, cloned.job_title);
        assert_eq!(insert_args.quote, cloned.quote);

        let filter_args = FilterArgs {
            job: Some("Dev".to_string()),
            company: None,
            date: None,
        };
        let cloned = filter_args.clone();
        assert_eq!(filter_args.job, cloned.job);
        assert_eq!(filter_args.company, cloned.company);
        assert_eq!(filter_args.date, cloned.date);

        let user_filters = UserFilters::default();
        let cloned = user_filters.clone();
        assert_eq!(user_filters.job, cloned.job);
        assert_eq!(user_filters.company, cloned.company);
        assert_eq!(user_filters.date, cloned.date);
    }
}
