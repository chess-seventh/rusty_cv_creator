use crate::command_runner::SystemRunner;
use crate::database::{DbConnection, establish_connection, save_new_cv_to_db};
use crate::global_conf::AppContext;
use crate::prepare_cv;
use log::{info, warn};
use rusty_cv_creator::models::Cv;

pub fn insert_cv(ctx: &AppContext) -> Result<String, Box<dyn std::error::Error>> {
    // These come from the UserInput, FilterArgs
    let job_title = ctx.get_job_title()?;
    let company_name = ctx.get_company_name()?;
    let quote = ctx.get_quote().ok();
    let variant = ctx.get_variant();

    let destination_cv_file_full_path = prepare_cv(
        ctx,
        &SystemRunner,
        &job_title,
        &company_name,
        variant.as_ref(),
    )?;

    // This comes from the INI file.
    let save_to_db = ctx.get_user_input_save_to_db();
    let application_date = ctx.get_today_str();

    // A failed DB save must not discard a successfully generated CV — log and continue.
    if let Err(e) = run_persistence(
        save_to_db,
        || establish_connection(ctx),
        &destination_cv_file_full_path,
        &job_title,
        &company_name,
        quote.as_ref(),
        &application_date,
    ) {
        warn!("Could not save CV to database: {e:}");
    }

    Ok(destination_cv_file_full_path)
}

/// Persist the generated CV only when the user opted in via `--save-to-database`.
///
/// The database connection is opened lazily through `open_conn`, so the opt-out
/// path performs no connection and no write at all (CVs can be generated fully
/// offline). Returns `Some(cv)` when a row was written, `None` when opted out.
fn run_persistence<F>(
    save_to_db: bool,
    open_conn: F,
    cv_path: &str,
    job_title: &str,
    company: &str,
    quote: Option<&String>,
    application_date: &str,
) -> Result<Option<Cv>, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<DbConnection, Box<dyn std::error::Error>>,
{
    if !save_to_db {
        warn!("CV NOT SAVED TO DATABASE!");
        return Ok(None);
    }

    let mut conn = open_conn()?;
    let cv = save_new_cv_to_db(
        &mut conn,
        cv_path,
        job_title,
        company,
        quote,
        application_date,
    )?;
    info!("Saved CV to database");
    Ok(Some(cv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_structure::{FilterArgs, UserAction, UserInput};
    use configparser::ini::Ini;
    use diesel::prelude::*;

    fn context_without_job_title() -> AppContext {
        let ui = UserInput {
            action: UserAction::Insert(FilterArgs::default()),
            save_to_database: false,
            view_generated_cv: false,
            dry_run: false,
            config_ini: String::new(),
            engine: "sqlite".to_string(),
        };
        AppContext::new(Ini::new(), chrono::Local::now(), ui)
    }

    fn sqlite_conn_with_table() -> DbConnection {
        let mut conn = DbConnection::Sqlite(SqliteConnection::establish(":memory:").unwrap());
        diesel::sql_query(
            "CREATE TABLE cv (id INTEGER PRIMARY KEY AUTOINCREMENT, application_date VARCHAR, \
             job_title VARCHAR NOT NULL, company VARCHAR NOT NULL, quote VARCHAR NOT NULL, \
             pdf_cv_path VARCHAR NOT NULL, generated BOOLEAN NOT NULL DEFAULT 1)",
        )
        .execute(&mut conn)
        .unwrap();
        conn
    }

    #[test]
    fn test_insert_cv_errors_when_job_title_missing() {
        // A FilterArgs without a job_title cannot drive a CV build: insert_cv
        // surfaces the error instead of relying on a (now removed) global panic.
        let ctx = context_without_job_title();
        assert!(insert_cv(&ctx).is_err());
    }

    #[test]
    fn test_run_persistence_opt_out_writes_nothing_and_opens_no_connection() {
        // The connection factory must NOT be invoked when save is opted out.
        let result = run_persistence(
            false,
            || -> Result<DbConnection, Box<dyn std::error::Error>> {
                panic!("connection must not be opened when --save-to-database is omitted")
            },
            "/tmp/cv.pdf",
            "Dev",
            "ACME",
            None,
            "2024-01-01",
        )
        .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_run_persistence_opt_in_writes_row() {
        let result = run_persistence(
            true,
            || Ok::<_, Box<dyn std::error::Error>>(sqlite_conn_with_table()),
            "/tmp/cv.pdf",
            "Dev",
            "ACME",
            None,
            "2024-01-01",
        )
        .unwrap();
        let cv = result.expect("a CV row should have been written when opted in");
        assert_eq!(cv.pdf_cv_path, "/tmp/cv.pdf");
        assert_eq!(cv.job_title, "Dev");
    }
}
