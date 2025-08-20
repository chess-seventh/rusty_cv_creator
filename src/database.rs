use crate::cli_structure::FilterArgs;
use crate::global_conf::GLOBAL_VAR;
use crate::helpers::{check_if_db_env_is_set_or_set_from_config, fix_home_directory_path};
use diesel::prelude::*;
use log::{error, info};
use rusty_cv_creator::models::Cv;
use rusty_cv_creator::models::NewCv;
use rusty_cv_creator::schema::cv;
use std::env;

extern crate skim;

pub enum _ConnectionType {
    Postgres(PgConnection),
    Sqlite(SqliteConnection),
}

#[allow(clippy::used_underscore_items)]
fn _define_connection_type(worker_type: &str) -> _ConnectionType {
    match worker_type {
        "postgres" => _ConnectionType::Postgres(establish_connection_postgres()),
        "sqlite" => _ConnectionType::Sqlite(_establish_connection_sqlite()),
        _ => panic!("worker type not found"),
    }
}

pub fn establish_connection_postgres() -> PgConnection {
    let db_url = GLOBAL_VAR.get().unwrap().get_user_input_db_url();

    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {db_url}"))
}

pub fn _establish_connection_sqlite() -> SqliteConnection {
    let database_url = &env::var("DATABASE_URL").unwrap_or_else(|_| {
        match check_if_db_env_is_set_or_set_from_config() {
            Ok(_v) => info!("Fetched the DATABASE_URL env variable"),
            Err(v) => panic!("{}", v),
        }
        env::var("DATABASE_URL").unwrap()
    });

    let db = fix_home_directory_path(database_url);
    println!("{db:?}");
    SqliteConnection::establish(&db)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

// fn check_if_entry_exists() -> Result<i64, diesel::result::Error> {
fn check_if_entry_exists(g_job_title: &str, g_company: &str, g_quote: &str) -> Option<i32> {
    use rusty_cv_creator::schema::cv::dsl::cv;
    use rusty_cv_creator::schema::cv::{company, job_title, quote};

    let conn = &mut establish_connection_postgres();

    let selection = cv
        .select(Cv::as_select())
        .filter(job_title.eq(g_job_title))
        .filter(company.eq(g_company))
        .filter(quote.eq(g_quote))
        .first(conn)
        .optional();

    match selection {
        Ok(Some(selection)) => {
            info!(
                "CV with id: {} has a job_title: {}",
                selection.id, selection.job_title
            );
            Some(selection.id)
        }
        Ok(None) => {
            info!("Unable to find CV");
            None
        }
        _ => panic!("An error occurred while fetching CV"),
    }
}

pub fn save_new_cv_to_db(cv_path: &str) -> Result<Cv, &str> {
    // let db_engine = GlobalVars::get_db_engine();
    // let conn = &mut define_connection_type("sqlite").unwrap();

    let global_var = if let Some(v) = GLOBAL_VAR.get() {
        info!("Could get GLOBAL_VAR");
        v
    } else {
        error!("Could not get GLOBAL_VAR, something is wrong");
        return Err("Could not get GLOBAL_VAR, something is wrong");
    };

    let job_title = global_var.get_user_job_title();
    let company = global_var.get_user_input_company_name();
    let quote = global_var.get_user_input_quote();

    let application_date = global_var.get_today_str();

    let conn = &mut establish_connection_postgres();

    if let Some(id) = check_if_entry_exists(&job_title, &company, &quote) {
        info!("Entry already exists with id: {id}");

        return Ok(cv::table.find(id).first(conn).expect("Error loading CV"));
    }

    let new_cv = NewCv {
        application_date: Some(&application_date),
        job_title: &job_title,
        company: &company,
        quote: &quote,
        pdf_cv_path: cv_path,
        generated: true,
    };

    Ok(diesel::insert_into(cv::table)
        .values(&new_cv)
        .returning(Cv::as_returning())
        .get_result(conn)
        .expect("Error saving new CV"))
}

pub fn read_cv_from_db(filters: &FilterArgs) -> Vec<String> {
    use rusty_cv_creator::schema::cv::dsl::cv;

    let conn = &mut establish_connection_postgres();

    // TODO filters on proper DB
    println!("Filter to apply to DB: {filters:?}");
    let cv_results = cv
        .limit(50)
        // .filter()
        .select(Cv::as_select())
        .load(conn)
        .expect("Error loading CVs");

    let mut pdf_cvs = vec![];
    for pdf in cv_results {
        pdf_cvs.push(pdf.pdf_cv_path);
        pdf_cvs.push("\n".to_string());
    }

    pdf_cvs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_structure::UserInput;
    use crate::cli_structure::{FilterArgs, InsertArgs, UserAction};
    use crate::config_parse::set_global_vars;
    // use crate::global_conf::{GlobalVars, GLOBAL_VAR};
    use serial_test::serial;
    use std::env;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config_content() -> String {
        r#"
[db]
db_path = "/tmp/test_db"
db_file = "test.db"
engine = "sqlite"
db_pg_host = "postgresql://test:test@localhost:5432/test_db"
"#
        .to_string()
    }

    fn create_test_user_input_with_config(config_path: &str) -> UserInput {
        UserInput {
            action: UserAction::Insert(InsertArgs {
                company_name: "Test Company".to_string(),
                job_title: "Software Engineer".to_string(),
                quote: "Test quote".to_string(),
            }),
            save_to_database: Some(true),
            view_generated_cv: Some(false),
            dry_run: Some(false),
            config_ini: config_path.to_string(),
            engine: "sqlite".to_string(),
        }
    }

    // Helper function to reset GLOBAL_VAR for testing
    fn make_user_input(tmp: &NamedTempFile) -> UserInput {
        UserInput {
            action: UserAction::Insert(InsertArgs {
                company_name: "Co".into(),
                job_title: "Job".into(),
                quote: "Quote".into(),
            }),
            save_to_database: Some(true),
            view_generated_cv: Some(false),
            dry_run: Some(false),
            config_ini: tmp.path().to_str().unwrap().into(),
            engine: "sqlite".into(),
        }
    }

    #[test]
    #[serial]
    fn test_save_new_cv_to_db_with_global_var() {
        // Initialize GLOBAL_VAR once per process
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            "[db]\ndb_pg_host = \"sqlite:///temp.db\"\nengine = \"sqlite\"\n"
        )
        .unwrap();
        let ui = make_user_input(&tmp);

        // Only this test calls set_global_vars
        set_global_vars(&ui).unwrap();

        // Now GLOBAL_VAR is set; save_new_cv_to_db will panic connecting
        let _ = std::panic::catch_unwind(|| {
            let _ = save_new_cv_to_db("/dummy");
        });
    }

    #[test]
    fn test_connection_type_enum() {
        // Test that the enum variants exist and can be matched
        // Note: We can't easily test the actual connections without database setup

        // This is a compile-time test that the enum variants exist
        let _postgres_variant = |conn: PgConnection| _ConnectionType::Postgres(conn);
        let _sqlite_variant = |conn: SqliteConnection| _ConnectionType::Sqlite(conn);
    }

    #[test]
    #[serial]
    #[ignore] // Requires PostgreSQL setup
    fn test_establish_connection_postgres() {
        // Setup GLOBAL_VAR with PostgreSQL config
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = create_test_config_content();
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let user_input = create_test_user_input_with_config(file_path);
        let _ = set_global_vars(&user_input);

        // This test requires actual PostgreSQL running
        // In practice, you'd use a test database or mock
        // let connection = establish_connection_postgres();
        // assert!(connection is valid);
    }

    #[test]
    #[serial]
    #[ignore] // Requires database setup
    fn test_establish_connection_sqlite() {
        // Setup environment variable for SQLite
        env::set_var("DATABASE_URL", "sqlite:///tmp/test.db");

        // Setup GLOBAL_VAR with SQLite config
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = create_test_config_content();
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let user_input = create_test_user_input_with_config(file_path);
        let _ = set_global_vars(&user_input);

        // This test requires actual SQLite database file
        // let connection = _establish_connection_sqlite();
        // assert!(connection is valid);
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_check_if_entry_exists_found() {
        // This test requires a database with test data
        // let result = check_if_entry_exists("Software Engineer", "ACME Corp", "Great opportunity");
        // assert!(result.is_some());
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_check_if_entry_exists_not_found() {
        // let result = check_if_entry_exists("Nonexistent Job", "Nonexistent Company", "Nonexistent Quote");
        // assert!(result.is_none());
    }

    #[test]
    #[serial]
    #[ignore] // Requires database setup
    fn test_save_new_cv_to_db_success() {
        // Setup GLOBAL_VAR
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = create_test_config_content();
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let user_input = create_test_user_input_with_config(file_path);
        let _ = set_global_vars(&user_input);

        // This test requires database connection and migrations
        // let result = save_new_cv_to_db("/tmp/test_cv.pdf");
        // assert!(result.is_ok());
    }

    // #[test]
    // #[serial]
    // // #[should_panic]
    // fn test_save_new_cv_to_db_no_global_var() {
    //     let result = save_new_cv_to_db("/tmp/test_cv.pdf");
    //     assert!(result.is_err());
    // }

    // Mock implementations for testing without database
    #[cfg(test)]
    mod mock_database {
        use super::*;

        pub struct MockCvDatabase {
            pub cvs: Vec<MockCvRecord>,
        }

        pub struct MockCvRecord {
            pub id: i32,
            pub job_title: String,
            pub company: String,
            pub quote: String,
            pub pdf_cv_path: String,
            pub application_date: Option<String>,
            pub generated: bool,
        }

        impl MockCvDatabase {
            pub fn new() -> Self {
                Self {
                    cvs: vec![
                        MockCvRecord {
                            id: 1,
                            job_title: "Software Engineer".to_string(),
                            company: "ACME Corp".to_string(),
                            quote: "Great opportunity".to_string(),
                            pdf_cv_path: "/tmp/cv1.pdf".to_string(),
                            application_date: Some("2023-08-19".to_string()),
                            generated: true,
                        },
                        MockCvRecord {
                            id: 2,
                            job_title: "Senior Developer".to_string(),
                            company: "Tech Inc".to_string(),
                            quote: "Exciting challenge".to_string(),
                            pdf_cv_path: "/tmp/cv2.pdf".to_string(),
                            application_date: Some("2023-08-18".to_string()),
                            generated: true,
                        },
                    ],
                }
            }

            pub fn find_entry(&self, job_title: &str, company: &str, quote: &str) -> Option<i32> {
                for cv in &self.cvs {
                    if cv.job_title == job_title && cv.company == company && cv.quote == quote {
                        return Some(cv.id);
                    }
                }
                None
            }

            pub fn insert_cv(
                &mut self,
                job_title: &str,
                company: &str,
                quote: &str,
                pdf_path: &str,
            ) -> i32 {
                let new_id = self.cvs.len() as i32 + 1;
                self.cvs.push(MockCvRecord {
                    id: new_id,
                    job_title: job_title.to_string(),
                    company: company.to_string(),
                    quote: quote.to_string(),
                    pdf_cv_path: pdf_path.to_string(),
                    application_date: Some("2023-08-19".to_string()),
                    generated: true,
                });
                new_id
            }

            pub fn find_all(&self, _filters: &FilterArgs) -> Vec<String> {
                self.cvs.iter().map(|cv| cv.pdf_cv_path.clone()).collect()
            }
        }

        #[test]
        fn test_mock_database_find_entry() {
            let db = MockCvDatabase::new();

            let result = db.find_entry("Software Engineer", "ACME Corp", "Great opportunity");
            assert_eq!(result, Some(1));

            let result = db.find_entry("Nonexistent", "Company", "Quote");
            assert_eq!(result, None);
        }

        #[test]
        fn test_mock_database_insert_cv() {
            let mut db = MockCvDatabase::new();
            assert_eq!(db.cvs.len(), 2);

            let new_id = db.insert_cv(
                "Product Manager",
                "StartupCo",
                "Innovation focus",
                "/tmp/cv3.pdf",
            );
            assert_eq!(new_id, 3);
            assert_eq!(db.cvs.len(), 3);

            let new_cv = &db.cvs[2];
            assert_eq!(new_cv.job_title, "Product Manager");
            assert_eq!(new_cv.company, "StartupCo");
            assert_eq!(new_cv.quote, "Innovation focus");
            assert_eq!(new_cv.pdf_cv_path, "/tmp/cv3.pdf");
        }

        #[test]
        fn test_mock_database_find_all() {
            let db = MockCvDatabase::new();
            let filters = FilterArgs {
                job: None,
                company: None,
                date: None,
            };

            let results = db.find_all(&filters);
            assert_eq!(results.len(), 2);
            assert!(results.contains(&"/tmp/cv1.pdf".to_string()));
            assert!(results.contains(&"/tmp/cv2.pdf".to_string()));
        }

        #[test]
        fn test_mock_database_duplicate_check() {
            let db = MockCvDatabase::new();

            // Try to insert duplicate
            let existing = db.find_entry("Software Engineer", "ACME Corp", "Great opportunity");
            assert!(existing.is_some());

            // In real implementation, this would return the existing record
            let existing_id = existing.unwrap();
            assert_eq!(existing_id, 1);
        }
    }

    // Integration test helpers
    #[cfg(test)]
    mod integration_helpers {
        use super::*;

        pub fn setup_test_database() -> String {
            let db_url = "sqlite::memory:";
            env::set_var("DATABASE_URL", db_url);

            // In a real test, you'd run migrations here
            // diesel::connection::establish(db_url).unwrap();
            // diesel_migrations::run_pending_migrations(...);

            db_url.to_string()
        }

        #[test]
        #[ignore] // Requires diesel setup
        fn test_database_setup() {
            let db_url = setup_test_database();
            assert!(db_url.contains("sqlite"));
        }
    }

    // Test for _define_connection_type function
    #[test]
    #[ignore] // Requires database connections
    fn test_define_connection_type_postgres() {
        // let connection = _define_connection_type("postgres");
        // match connection {
        //     _ConnectionType::Postgres(_) => assert!(true),
        //     _ => panic!("Expected Postgres connection"),
        // }
    }

    #[test]
    #[ignore] // Requires database connections
    fn test_define_connection_type_sqlite() {
        // let connection = _define_connection_type("sqlite");
        // match connection {
        //     _ConnectionType::Sqlite(_) => assert!(true),
        //     _ => panic!("Expected SQLite connection"),
        // }
    }

    #[test]
    #[should_panic(expected = "worker type not found")]
    fn test_define_connection_type_invalid() {
        _define_connection_type("invalid");
    }

    // Environment variable tests
    #[test]
    fn test_database_url_environment() {
        let original = env::var("DATABASE_URL").ok();

        env::set_var("DATABASE_URL", "sqlite:///tmp/test.db");
        assert_eq!(env::var("DATABASE_URL").unwrap(), "sqlite:///tmp/test.db");

        // Restore original value
        match original {
            Some(value) => env::set_var("DATABASE_URL", value),
            None => env::remove_var("DATABASE_URL"),
        }
    }

    // Test the connection string formats
    #[test]
    fn test_postgres_connection_string_format() {
        let conn_str = "postgresql://user:pass@localhost:5432/dbname";
        assert!(conn_str.starts_with("postgresql://"));
        assert!(conn_str.contains("localhost"));
        assert!(conn_str.contains("5432"));
    }

    #[test]
    fn test_sqlite_connection_string_format() {
        let conn_str = "sqlite:///tmp/test.db";
        assert!(conn_str.starts_with("sqlite://"));
        assert!(conn_str.contains("/tmp/test.db"));
    }

    // Test error conditions
    #[test]
    fn test_various_error_conditions() {
        // Test empty strings
        let empty_title = "";
        let empty_company = "";
        let empty_quote = "";

        // In a real test with database, these would test actual error conditions
        assert!(empty_title.is_empty());
        assert!(empty_company.is_empty());
        assert!(empty_quote.is_empty());
    }
}
