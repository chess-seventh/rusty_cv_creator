use diesel::prelude::*;
use std::env;
use rusty_cv_creator::models::Cv;
use rusty_cv_creator::models::NewCv;
use rusty_cv_creator::schema::cv;

use crate::helpers::{check_if_db_env_is_set_or_set_from_config, fix_home_directory_path};
use crate::global_conf::GlobalVars;

extern crate skim;

pub enum _ConnectionType {
    Postgres(PgConnection),
    Sqlite(SqliteConnection),
}

fn _define_connection_type(worker_type: &str)-> _ConnectionType {
    match worker_type {
        "postgres" => _ConnectionType::Postgres(establish_connection_postgres()),
        "sqlite" => _ConnectionType::Sqlite(establish_connection_sqlite()),
        _ => panic!("worker type not found")
    }
}

pub fn establish_connection_postgres() -> PgConnection {
    let db_url = GlobalVars::get_db_url();

    PgConnection::establish(&db_url)
        .unwrap_or_else(|_| panic!("Error connecting to {db_url}"))
}


pub fn establish_connection_sqlite() -> SqliteConnection {
    let database_url = &env::var("DATABASE_URL").unwrap_or_else(|_| {
        check_if_db_env_is_set_or_set_from_config();
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
    use rusty_cv_creator::schema::cv::{job_title, company, quote};

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
            println!("CV with id: {} has a job_title: {}", selection.id, selection.job_title);
            Some(selection.id)
        },
        Ok(None) => {
            println!("Unable to find CV");
            None
        },
        _ => panic!("An error occurred while fetching CV"),
    }

}

pub fn save_new_cv_to_database(cv_path: &str) -> Cv {
    // let db_engine = GlobalVars::get_db_engine();
    // let conn = &mut define_connection_type("sqlite").unwrap();

    let job_title = GlobalVars::get_user_job_title();
    let company= GlobalVars::get_user_company_name();
    let quote = GlobalVars::get_user_quote();

    let application_date = GlobalVars::get_today_str();

    let conn = &mut establish_connection_postgres();

    if let Some(id) = check_if_entry_exists(&job_title, &company, &quote) {
        println!("Entry already exists with id: {id}");

        return cv::table
            .find(id)
            .first(conn)
            .expect("Error loading CV")
    }

    let new_cv = NewCv {
        application_date: Some(&application_date),
        job_title: &job_title,
        company: &company,
        quote: &quote,
        pdf_cv_path: cv_path,
        generated: true
    };

    diesel::insert_into(cv::table)
        .values(&new_cv)
        .returning(Cv::as_returning())
        .get_result(conn)
        .expect("Error saving new CV")
}

pub fn read_cv_from_database() -> Vec<String> {
    use rusty_cv_creator::schema::cv::dsl::cv;

    let conn = &mut establish_connection_postgres();

    // TODO filters on proper DB
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

