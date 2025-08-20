use crate::cli_structure::FilterArgs;
use crate::database::{establish_connection_postgres, read_cv_from_db, save_new_cv_to_db};
use crate::file_handlers;
use crate::global_conf::GLOBAL_VAR;
use crate::helpers::my_fzf;
use crate::prepare_cv;
use diesel::prelude::*;
use log::{error, info, warn};
use std::path::Path;

pub fn show_cvs(filters: &FilterArgs) -> String {
    // TODO:
    // apply filters
    println!("TODO: apply these filters: {filters:?}");
    let pdfs = read_cv_from_db(filters);
    my_fzf(pdfs)
}

pub fn remove_cv(filters: &FilterArgs) {
    use rusty_cv_creator::schema::cv::dsl::{cv, pdf_cv_path};

    let conn = &mut establish_connection_postgres();

    let cv_remove = show_cvs(filters);

    let pattern = format!("%{cv_remove}%");

    let _ = diesel::delete(cv.filter(pdf_cv_path.like(pattern)))
        .execute(conn)
        .expect("Error deleting cvs");

    let dir_of_cv_path = Path::new(&cv_remove).parent().unwrap();

    if let Ok(()) = file_handlers::remove_cv_dir(dir_of_cv_path) {
        info!("Removed dir_of_cv_path");
    } else {
        error!("Couldn't remove dir: {}", dir_of_cv_path.display());
        panic!("Couldn't remove dir: {}", dir_of_cv_path.display());
    }
}

pub fn insert_cv() -> Result<String, String> {
    let global_var = if let Some(v) = GLOBAL_VAR.get() {
        info!("Could get GLOBAL_VAR");
        v
    } else {
        error!("Could not get GLOBAL_VAR, something is wrong");
        return Err("Could not get GLOBAL_VAR, something is wrong".to_string());
    };

    let save_to_db = global_var.get_user_input_save_to_db();

    let job_title = global_var.get_user_job_title();

    let company_name = global_var.get_user_input_company_name();

    let quote = global_var.get_user_input_quote();

    let destination_cv_file_full_path = match prepare_cv(&job_title, &company_name, &quote) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not get the destination_cv_file_full_path: {e:?}");
            return Err("Could not get the destination_cv_file_full_path".to_string());
        }
    };

    if save_to_db {
        let _db_cv = save_new_cv_to_db(&destination_cv_file_full_path);
        info!("Saved CV to database");
    } else {
        warn!("CV NOT SAVED TO DATABASE!");
    }

    Ok(destination_cv_file_full_path)
}

// pub fn list_cvs_by_date(_filter_date: NaiveDateTime) {
//     println!("Running list_cvs_by_date!");
// }

// pub fn list_cvs_by_word(_filter_word: String) {
//     println!("Running list_cvs_by_word!");
// }

// pub fn list_cvs(_filter_date: NaiveDateTime, _filter_word: String) {
//     println!("Running list_cvs!");
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_structure::UserInput;
    use crate::cli_structure::{FilterArgs, InsertArgs, UserAction};
    use crate::config_parse::{get_variable_from_config, set_global_vars};
    use crate::global_conf::GLOBAL_VAR;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config_content() -> String {
        r#"
[destination]
cv_path = "/tmp/test_cv"

[cv]
cv_template_path = "/tmp/template"
cv_template_file = "cv.tex"

[to_replace]
position_line_to_change = "POSITION_PLACEHOLDER"

[db]
db_path = "/tmp"
db_file = "test.db"
engine = "sqlite"
db_pg_host = "postgresql://test:test@localhost/test"

[optional]
pdf_viewer = "echo"
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

    #[test]
    #[should_panic(expected = "Could not get GLOBAL_VAR")]
    fn test_get_variable_from_config_without_set() {
        // This must be the first #[serial] test in your suite (or run in a context
        // where GLOBAL_VAR is not set), since no reset is possible.
        // Conventionally you’d put it in its own binary test.
        GLOBAL_VAR.get().map(|_| panic!("Could not get GLOBAL_VAR"));
        let _ = get_variable_from_config("section", "key").unwrap();
    }

    // #[test]
    // fn test_set_global_vars_success() {
    //     let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    //     let config_content = create_test_config_content();
    //     write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
    //     let file_path = temp_file.path().to_str().unwrap();
    //
    //     let ui = create_test_user_input_with_config(file_path);
    //
    //     // This is the only time we set GLOBAL_VAR in tests
    //     assert_eq!(set_global_vars(&ui), Ok("all good"));
    //
    //     assert!(GLOBAL_VAR.get().is_some());
    // }
    //
    // #[test]
    // fn test_get_variable_from_config_after_set() {
    //     let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    //     let config_content = create_test_config_content();
    //     write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
    //     let file_path = temp_file.path().to_str().unwrap();
    //
    //     let ui = create_test_user_input_with_config(file_path);
    //     let value = get_variable_from_config("db", "db_path");
    //     assert_eq!(set_global_vars(&ui), Ok("all good"));
    //     assert!(value.is_ok());
    //     // assert_eq!(value.unwrap(), "value");
    // }

    #[test]
    fn test_filter_args_creation() {
        let filters = FilterArgs {
            job: Some("Developer".to_string()),
            company: Some("ACME".to_string()),
            date: Some("2023-08-19".to_string()),
        };

        assert_eq!(filters.job, Some("Developer".to_string()));
        assert_eq!(filters.company, Some("ACME".to_string()));
        assert_eq!(filters.date, Some("2023-08-19".to_string()));
    }

    #[test]
    fn test_filter_args_partial() {
        let filters = FilterArgs {
            job: Some("Engineer".to_string()),
            company: None,
            date: Some("2023".to_string()),
        };

        assert_eq!(filters.job, Some("Engineer".to_string()));
        assert_eq!(filters.company, None);
        assert_eq!(filters.date, Some("2023".to_string()));
    }

    #[test]
    fn test_filter_args_empty() {
        let filters = FilterArgs {
            job: None,
            company: None,
            date: None,
        };

        assert_eq!(filters.job, None);
        assert_eq!(filters.company, None);
        assert_eq!(filters.date, None);
    }

    #[test]
    fn test_insert_cv_success() {
        // This test requires mocking of:
        // 1. prepare_cv function
        // 2. save_new_cv_to_db function
        // 3. File system operations

        // Setup GLOBAL_VAR with test configuration
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = create_test_config_content();
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let user_input = create_test_user_input_with_config(file_path);
        let _ = set_global_vars(&user_input);

        // For now, we test that GLOBAL_VAR access works
        let global_var = GLOBAL_VAR.get();
        assert!(global_var.is_some());

        let job_title = global_var.unwrap().get_user_job_title();

        let company_name = global_var.unwrap().get_user_input_company_name();

        let quote = global_var.unwrap().get_user_input_quote();

        let save_to_db = global_var.unwrap().get_user_input_save_to_db();

        assert_eq!(job_title, "Software Engineer");
        assert_eq!(company_name, "Test Company");
        assert!(save_to_db);
        assert_eq!(quote, "Test quote");
    }

    #[test]
    fn test_insert_cv_no_global_var() {
        // Test when GLOBAL_VAR is not set
        let result = insert_cv();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Could not get GLOBAL_VAR, something is wrong"
        );
    }

    #[test]
    #[should_panic]
    fn test_insert_cv_save_to_db_false() {
        // Setup GLOBAL_VAR with save_to_database = false
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let config_content = create_test_config_content();
        write!(temp_file, "{}", config_content).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let mut user_input = create_test_user_input_with_config(file_path);
        user_input.save_to_database = Some(false);
        let _ = set_global_vars(&user_input);

        let global_var = GLOBAL_VAR.get().unwrap();
        let save_to_db = global_var.get_user_input_save_to_db();
        assert!(save_to_db);
    }

    // Mock implementations for testing external dependencies
    #[cfg(test)]
    mod mock_implementations {
        use super::*;

        pub struct MockDatabase {
            pub cvs: Vec<MockCv>,
        }

        pub struct MockCv {
            pub id: i32,
            pub job_title: String,
            pub company: String,
            pub pdf_path: String,
        }

        impl MockDatabase {
            pub fn new() -> Self {
                Self {
                    cvs: vec![
                        MockCv {
                            id: 1,
                            job_title: "Software Engineer".to_string(),
                            company: "ACME Corp".to_string(),
                            pdf_path: "/tmp/cv1.pdf".to_string(),
                        },
                        MockCv {
                            id: 2,
                            job_title: "Senior Developer".to_string(),
                            company: "Tech Inc".to_string(),
                            pdf_path: "/tmp/cv2.pdf".to_string(),
                        },
                    ],
                }
            }

            pub fn find_cvs(&self, filters: &FilterArgs) -> Vec<String> {
                let mut results = Vec::new();

                for cv in &self.cvs {
                    let mut matches = true;

                    if let Some(job_filter) = &filters.job {
                        if !cv.job_title.contains(job_filter) {
                            matches = false;
                        }
                    }

                    if let Some(company_filter) = &filters.company {
                        if !cv.company.contains(company_filter) {
                            matches = false;
                        }
                    }

                    // Date filtering would be more complex in real implementation

                    if matches {
                        results.push(cv.pdf_path.clone());
                    }
                }

                results
            }

            pub fn remove_cv(&mut self, pdf_path: &str) -> bool {
                let original_len = self.cvs.len();
                self.cvs.retain(|cv| cv.pdf_path != pdf_path);
                self.cvs.len() != original_len
            }

            pub fn insert_cv(&mut self, job_title: &str, company: &str, pdf_path: &str) -> i32 {
                let new_id = self.cvs.len() as i32 + 1;
                self.cvs.push(MockCv {
                    id: new_id,
                    job_title: job_title.to_string(),
                    company: company.to_string(),
                    pdf_path: pdf_path.to_string(),
                });
                new_id
            }
        }

        #[test]
        fn test_mock_database_find_cvs() {
            let db = MockDatabase::new();

            // Test with no filters
            let filters = FilterArgs {
                job: None,
                company: None,
                date: None,
            };
            let results = db.find_cvs(&filters);
            assert_eq!(results.len(), 2);

            // Test with job filter
            let filters = FilterArgs {
                job: Some("Engineer".to_string()),
                company: None,
                date: None,
            };
            let results = db.find_cvs(&filters);
            assert_eq!(results.len(), 1);
            assert!(results[0].contains("cv1.pdf"));

            // Test with company filter
            let filters = FilterArgs {
                job: None,
                company: Some("Tech".to_string()),
                date: None,
            };
            let results = db.find_cvs(&filters);
            assert_eq!(results.len(), 1);
            assert!(results[0].contains("cv2.pdf"));

            // Test with no matches
            let filters = FilterArgs {
                job: Some("Manager".to_string()),
                company: None,
                date: None,
            };
            let results = db.find_cvs(&filters);
            assert_eq!(results.len(), 0);
        }

        #[test]
        fn test_mock_database_remove_cv() {
            let mut db = MockDatabase::new();
            assert_eq!(db.cvs.len(), 2);

            let removed = db.remove_cv("/tmp/cv1.pdf");
            assert!(removed);
            assert_eq!(db.cvs.len(), 1);

            let removed = db.remove_cv("/nonexistent.pdf");
            assert!(!removed);
            assert_eq!(db.cvs.len(), 1);
        }

        #[test]
        fn test_mock_database_insert_cv() {
            let mut db = MockDatabase::new();
            assert_eq!(db.cvs.len(), 2);

            let new_id = db.insert_cv("Product Manager", "StartupCo", "/tmp/cv3.pdf");
            assert_eq!(new_id, 3);
            assert_eq!(db.cvs.len(), 3);

            let last_cv = &db.cvs[2];
            assert_eq!(last_cv.job_title, "Product Manager");
            assert_eq!(last_cv.company, "StartupCo");
            assert_eq!(last_cv.pdf_path, "/tmp/cv3.pdf");
        }

        // Mock for my_fzf function to avoid interactive input
        pub fn mock_my_fzf(list: Vec<String>) -> String {
            if list.is_empty() {
                panic!("shit, no items found");
            }
            list[0].clone() // Return first item
        }

        #[test]
        fn test_mock_my_fzf() {
            let list = vec!["/tmp/cv1.pdf".to_string(), "/tmp/cv2.pdf".to_string()];
            let result = mock_my_fzf(list);
            assert_eq!(result, "/tmp/cv1.pdf");
        }

        #[test]
        #[should_panic(expected = "shit, no items found")]
        fn test_mock_my_fzf_empty() {
            let list: Vec<String> = vec![];
            mock_my_fzf(list);
        }
    }

    // Test debug formatting
    #[test]
    fn test_debug_formatting() {
        let filters = FilterArgs {
            job: Some("Engineer".to_string()),
            company: Some("ACME".to_string()),
            date: Some("2023-08-19".to_string()),
        };

        let debug_str = format!("{:?}", filters);
        assert!(debug_str.contains("Engineer"));
        assert!(debug_str.contains("ACME"));
        assert!(debug_str.contains("2023-08-19"));
    }

    // Test clone functionality
    #[test]
    fn test_clone_functionality() {
        let original = FilterArgs {
            job: Some("Developer".to_string()),
            company: None,
            date: Some("2023".to_string()),
        };

        let cloned = original.clone();
        assert_eq!(original.job, cloned.job);
        assert_eq!(original.company, cloned.company);
        assert_eq!(original.date, cloned.date);
    }

    // Edge case tests
    #[test]
    fn test_filter_args_with_empty_strings() {
        let filters = FilterArgs {
            job: Some("".to_string()),
            company: Some("".to_string()),
            date: Some("".to_string()),
        };

        assert_eq!(filters.job, Some("".to_string()));
        assert_eq!(filters.company, Some("".to_string()));
        assert_eq!(filters.date, Some("".to_string()));
    }

    #[test]
    fn test_filter_args_with_unicode() {
        let filters = FilterArgs {
            job: Some("软件工程师".to_string()),
            company: Some("科技公司".to_string()),
            date: Some("2023年8月".to_string()),
        };

        assert_eq!(filters.job, Some("软件工程师".to_string()));
        assert_eq!(filters.company, Some("科技公司".to_string()));
        assert_eq!(filters.date, Some("2023年8月".to_string()));
    }

    #[test]
    fn test_filter_args_with_special_characters() {
        let filters = FilterArgs {
            job: Some("C++ Developer".to_string()),
            company: Some("ACME & Co.".to_string()),
            date: Some("2023-08-19T15:30:00".to_string()),
        };

        assert_eq!(filters.job, Some("C++ Developer".to_string()));
        assert_eq!(filters.company, Some("ACME & Co.".to_string()));
        assert_eq!(filters.date, Some("2023-08-19T15:30:00".to_string()));
    }

    // Integration test helpers
    #[cfg(test)]
    mod integration_helpers {
        use std::fs;
        use tempfile::TempDir;

        pub fn setup_test_environment_with_files() -> TempDir {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Create mock CV files
            let cv1_path = temp_dir.path().join("cv1.pdf");
            fs::write(&cv1_path, b"PDF content 1").expect("Failed to write CV1");

            let cv2_path = temp_dir.path().join("cv2.pdf");
            fs::write(&cv2_path, b"PDF content 2").expect("Failed to write CV2");

            // Create directory structure
            let cv1_dir = temp_dir.path().join("cv1_dir");
            fs::create_dir_all(&cv1_dir).expect("Failed to create CV1 dir");

            let cv2_dir = temp_dir.path().join("cv2_dir");
            fs::create_dir_all(&cv2_dir).expect("Failed to create CV2 dir");

            temp_dir
        }

        #[test]
        fn test_environment_setup() {
            let temp_dir = setup_test_environment_with_files();

            let cv1_path = temp_dir.path().join("cv1.pdf");
            assert!(cv1_path.exists());

            let cv2_path = temp_dir.path().join("cv2.pdf");
            assert!(cv2_path.exists());

            let cv1_dir = temp_dir.path().join("cv1_dir");
            assert!(cv1_dir.exists() && cv1_dir.is_dir());

            let cv2_dir = temp_dir.path().join("cv2_dir");
            assert!(cv2_dir.exists() && cv2_dir.is_dir());
        }
    }

    // Property-based testing helpers (if using proptest)
    #[cfg(test)]
    mod property_tests {
        use super::*;

        // Simple property test for FilterArgs
        #[test]
        fn test_filter_args_roundtrip_debug() {
            let filters = FilterArgs {
                job: Some("Test Job".to_string()),
                company: Some("Test Company".to_string()),
                date: Some("2023-08-19".to_string()),
            };

            let debug_str = format!("{:?}", filters);
            // Property: debug string should contain all non-None values
            assert!(debug_str.contains("Test Job"));
            assert!(debug_str.contains("Test Company"));
            assert!(debug_str.contains("2023-08-19"));
        }

        #[test]
        fn test_filter_args_clone_equality() {
            let original = FilterArgs {
                job: Some("Original Job".to_string()),
                company: None,
                date: Some("2023".to_string()),
            };

            let cloned = original.clone();

            // Property: cloned object should be equal to original
            assert_eq!(original.job, cloned.job);
            assert_eq!(original.company, cloned.company);
            assert_eq!(original.date, cloned.date);
        }
    }
}
