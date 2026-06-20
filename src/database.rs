use crate::models::Cv;
use crate::models::NewCv;
use crate::schema::cv::{self};
use diesel::prelude::*;
use log::{error, info};

/// A backend-agnostic connection so the same query code runs against `Postgres`
/// in production and `SQLite` in tests.
#[derive(diesel::MultiConnection)]
pub enum DbConnection {
    Postgresql(PgConnection),
    Sqlite(SqliteConnection),
}

/// Establish a connection for the given `engine` against `url`.
///
/// The caller resolves the engine/url pair (e.g. from the INI config); this
/// function is pure infrastructure and depends only on the diesel models/schema.
pub fn establish_connection(
    engine: &str,
    url: &str,
) -> Result<DbConnection, Box<dyn std::error::Error>> {
    match engine.trim() {
        "postgres" => Ok(DbConnection::Postgresql(PgConnection::establish(url)?)),
        "sqlite" => Ok(DbConnection::Sqlite(SqliteConnection::establish(url)?)),
        other => Err(format!("Unknown DB engine: {other}").into()),
    }
}

fn check_if_entry_exists(
    conn: &mut DbConnection,
    g_job_title: &str,
    g_company: &str,
    g_quote: Option<&String>,
) -> Option<i32> {
    use crate::schema::cv::dsl::cv;
    use crate::schema::cv::{company, job_title, quote};

    let empty = String::new();
    let my_quote = g_quote.unwrap_or(&empty);

    // NOTE: `MultiConnection` does not support `Selectable::as_select`, so we
    // rely on the default (all-columns) selection, which matches `Cv`'s fields.
    let selection = cv
        .filter(job_title.eq(g_job_title))
        .filter(company.eq(g_company))
        .filter(quote.eq(my_quote))
        .first::<Cv>(conn)
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
        Err(e) => {
            error!("An error occurred while fetching CV: {e:}");
            None
        }
    }
}

pub fn save_new_cv_to_db(
    conn: &mut DbConnection,
    cv_path: &str,
    job_title: &str,
    company: &str,
    quote: Option<&String>,
    application_date: &str,
) -> Result<Cv, Box<dyn std::error::Error>> {
    if let Some(id) = check_if_entry_exists(conn, job_title, company, quote) {
        info!("Entry already exists with id: {id}");
        return Ok(cv::table.find(id).first::<Cv>(conn)?);
    }

    let empty = String::new();
    let my_quote = quote.unwrap_or(&empty);

    let new_cv = NewCv {
        application_date: Some(application_date),
        job_title,
        company,
        quote: my_quote,
        pdf_cv_path: cv_path,
        generated: true,
    };

    Ok(diesel::insert_into(cv::table)
        .values(&new_cv)
        .returning(cv::all_columns)
        .get_result::<Cv>(conn)?)
}

/// Return up to 50 stored CV PDF paths, each followed by a newline entry.
pub fn read_cv_paths(conn: &mut DbConnection) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use crate::schema::cv::dsl::cv;

    let cv_results = cv.limit(50).load::<Cv>(conn)?;

    let mut pdf_cvs = vec![];

    for pdf in cv_results {
        pdf_cvs.push(pdf.pdf_cv_path);
        pdf_cvs.push("\n".to_string());
    }

    Ok(pdf_cvs)
}

/// Load every stored application as a full [`Cv`] record (display source for the TUI).
pub fn load_all_applications(
    conn: &mut DbConnection,
) -> Result<Vec<Cv>, Box<dyn std::error::Error>> {
    // NOTE: `MultiConnection` forbids `as_select`; default (all-columns)
    // selection matches `Cv`'s field order against the `cv` table.
    Ok(crate::schema::cv::dsl::cv.load::<Cv>(conn)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an in-memory `SQLite` connection with the `cv` table created.
    fn sqlite_test_conn() -> DbConnection {
        let mut conn = DbConnection::Sqlite(
            SqliteConnection::establish(":memory:").expect("in-memory sqlite"),
        );
        diesel::sql_query(
            "CREATE TABLE cv (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                application_date VARCHAR, \
                job_title VARCHAR NOT NULL, \
                company VARCHAR NOT NULL, \
                quote VARCHAR NOT NULL, \
                pdf_cv_path VARCHAR NOT NULL, \
                generated BOOLEAN NOT NULL DEFAULT 1\
            )",
        )
        .execute(&mut conn)
        .expect("create cv table");
        conn
    }

    #[test]
    fn test_establish_connection_sqlite_in_memory_ok() {
        assert!(establish_connection("sqlite", ":memory:").is_ok());
    }

    #[test]
    fn test_establish_connection_unknown_engine_errors() {
        assert!(establish_connection("bogus", "").is_err());
    }

    #[test]
    fn test_save_new_cv_inserts_row() {
        let mut conn = sqlite_test_conn();
        let saved = save_new_cv_to_db(
            &mut conn,
            "/tmp/cv.pdf",
            "Senior DevOps Engineer",
            "ACME",
            None,
            "2024-01-01",
        )
        .unwrap();

        assert_eq!(saved.job_title, "Senior DevOps Engineer");
        assert_eq!(saved.company, "ACME");
        assert_eq!(saved.quote, "");
        assert_eq!(saved.pdf_cv_path, "/tmp/cv.pdf");
        assert!(saved.generated);
    }

    #[test]
    fn test_save_new_cv_is_idempotent_on_duplicate() {
        let mut conn = sqlite_test_conn();
        let first =
            save_new_cv_to_db(&mut conn, "/a.pdf", "SRE", "ACME", None, "2024-01-01").unwrap();
        // Same job/company/quote -> returns the existing row instead of inserting.
        let second =
            save_new_cv_to_db(&mut conn, "/b.pdf", "SRE", "ACME", None, "2024-02-02").unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(second.pdf_cv_path, "/a.pdf");

        let rows = read_cv_paths(&mut conn).unwrap();
        // One PDF path + its trailing newline entry.
        assert_eq!(rows.iter().filter(|r| r.as_str() == "/a.pdf").count(), 1);
    }

    #[test]
    fn test_save_new_cv_stores_quote() {
        let mut conn = sqlite_test_conn();
        let quote = "stay hungry".to_string();
        let saved = save_new_cv_to_db(
            &mut conn,
            "/q.pdf",
            "Platform",
            "ACME",
            Some(&quote),
            "2024-01-01",
        )
        .unwrap();
        assert_eq!(saved.quote, "stay hungry");
    }

    #[test]
    fn test_read_cv_paths_returns_paths() {
        let mut conn = sqlite_test_conn();
        save_new_cv_to_db(&mut conn, "/one.pdf", "A", "X", None, "2024-01-01").unwrap();
        save_new_cv_to_db(&mut conn, "/two.pdf", "B", "Y", None, "2024-01-01").unwrap();

        let rows = read_cv_paths(&mut conn).unwrap();
        assert!(rows.contains(&"/one.pdf".to_string()));
        assert!(rows.contains(&"/two.pdf".to_string()));
    }

    #[test]
    fn test_read_cv_paths_empty() {
        let mut conn = sqlite_test_conn();
        let rows = read_cv_paths(&mut conn).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_load_all_applications_returns_rows() {
        let mut conn = sqlite_test_conn();
        save_new_cv_to_db(
            &mut conn,
            "/cv.pdf",
            "Rust Engineer",
            "Acme Corp",
            None,
            "2024-03-15",
        )
        .unwrap();

        let cvs = load_all_applications(&mut conn).unwrap();
        assert_eq!(cvs.len(), 1);
        assert_eq!(cvs[0].company, "Acme Corp");
        assert_eq!(cvs[0].job_title, "Rust Engineer");
        assert_eq!(cvs[0].pdf_cv_path, "/cv.pdf");
        assert_eq!(cvs[0].application_date.as_deref(), Some("2024-03-15"));
    }

    #[test]
    fn test_load_all_applications_empty() {
        let mut conn = sqlite_test_conn();
        let cvs = load_all_applications(&mut conn).unwrap();
        assert!(cvs.is_empty());
    }
}
