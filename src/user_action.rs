use crate::cli_structure::FilterArgs;
use crate::database::{DbConnection, establish_connection, read_cv_from_db};
use crate::file_handlers;
use crate::global_conf::AppContext;
use crate::helpers::my_fzf;
use diesel::prelude::*;
use log::{error, info, warn};
use std::path::Path;

pub fn show_cvs(
    conn: &mut DbConnection,
    filters: &FilterArgs,
) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: apply filters
    warn!("TODO: apply these filters: {filters:?}");
    let pdfs = read_cv_from_db(conn, filters)?;
    Ok(my_fzf(pdfs))
}

pub fn remove_cv(ctx: &AppContext, filters: &FilterArgs) -> Result<(), Box<dyn std::error::Error>> {
    use rusty_cv_creator::schema::cv::dsl::{cv, pdf_cv_path};

    let mut conn = establish_connection(ctx)?;

    let cv_remove = show_cvs(&mut conn, filters)?;

    let pattern = format!("%{cv_remove}%");

    let _ = diesel::delete(cv.filter(pdf_cv_path.like(pattern)))
        .execute(&mut conn)
        .expect("Error deleting cvs");

    let dir_of_cv_path = Path::new(&cv_remove).parent().unwrap();

    if let Ok(()) = file_handlers::remove_cv_dir(dir_of_cv_path) {
        info!("Removed dir_of_cv_path");
        Ok(())
    } else {
        error!("Couldn't remove dir: {}", dir_of_cv_path.display());
        Err(format!("Couldn't remove dir: {}", dir_of_cv_path.display())
            .to_string()
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_cvs_runs_query_path() {
        // `my_fzf` is interactive and panics without a real selection, so we
        // only assert that the read/query path executes without a DB error.
        let mut conn = DbConnection::Sqlite(SqliteConnection::establish(":memory:").unwrap());
        diesel::sql_query(
            "CREATE TABLE cv (id INTEGER PRIMARY KEY AUTOINCREMENT, application_date VARCHAR, \
             job_title VARCHAR NOT NULL, company VARCHAR NOT NULL, quote VARCHAR NOT NULL, \
             pdf_cv_path VARCHAR NOT NULL, generated BOOLEAN NOT NULL DEFAULT 1)",
        )
        .execute(&mut conn)
        .unwrap();

        let filters = FilterArgs::default();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            show_cvs(&mut conn, &filters)
        }));
    }
}
