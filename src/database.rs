use crate::cli_structure::FilterArgs;
use crate::global_conf::get_global_var;
use crate::global_conf::GLOBAL_VAR;
use crate::helpers::fix_home_directory_path;
use diesel::prelude::*;
use log::{error, info};
use rusty_cv_creator::models::Cv;
use rusty_cv_creator::models::NewCv;
use rusty_cv_creator::schema::cv::{self};
use std::env;

extern crate skim;

#[allow(dead_code)]
pub enum _ConnectionType {
    Postgres(PgConnection),
    Sqlite(SqliteConnection),
}

#[allow(clippy::used_underscore_items)]
#[allow(dead_code)]
fn define_connection_type(
    worker_type: &str,
) -> Result<_ConnectionType, Box<dyn std::error::Error>> {
    match worker_type {
        "postgres" => Ok(_ConnectionType::Postgres(establish_connection_postgres()?)),
        "sqlite" => Ok(_ConnectionType::Sqlite(_establish_connection_sqlite())),
        _ => panic!("worker type not found"),
    }
}

pub fn establish_connection_postgres() -> Result<PgConnection, Box<dyn std::error::Error>> {
    let db_url = get_global_var().get_user_input_db_url()?;

    Ok(PgConnection::establish(&db_url)?) // .unwrap_or_else(|_| panic!("Error connecting to {db_url}"))
}

pub fn _establish_connection_sqlite() -> SqliteConnection {
    let database_url = &env::var("DATABASE_URL")
        .unwrap_or_else(|_| panic!("Could not get DATABASE_URL from env when establishing sqlite"));

    let db = fix_home_directory_path(database_url);
    println!("{db:?}");
    SqliteConnection::establish(&db)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

// fn check_if_entry_exists() -> Result<i64, diesel::result::Error> {
fn check_if_entry_exists(
    g_job_title: &str,
    g_company: &str,
    g_quote: Option<&String>,
) -> Option<i32> {
    use rusty_cv_creator::schema::cv::dsl::cv;
    use rusty_cv_creator::schema::cv::{company, job_title, quote};

    let conn = &mut establish_connection_postgres().ok()?;

    let my_quote = match g_quote {
        Some(q) => q,
        None => &String::new(),
    };

    let selection = cv
        .select(Cv::as_select())
        .filter(job_title.eq(g_job_title))
        .filter(company.eq(g_company))
        .filter(quote.eq(my_quote))
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

// pub fn save_new_cv_to_db(cv_path: &str, insert_args: FilterArgs) -> Result<Cv, &str> {
pub fn save_new_cv_to_db(
    cv_path: &str,
    job_title: &str,
    company: &str,
    quote: Option<&String>,
) -> Result<Cv, Box<dyn std::error::Error>> {
    // let db_engine = GlobalVars::get_db_engine();
    // let conn = &mut define_connection_type("sqlite").unwrap();

    let global_var = if let Some(v) = GLOBAL_VAR.get() {
        info!("Could get GLOBAL_VAR");
        v
    } else {
        error!("Could not get GLOBAL_VAR, something is wrong");
        return Err("Could not get GLOBAL_VAR, something is wrong"
            .to_string()
            .into());
    };

    // let job_title = insert_args.clone().job_title.unwrap().clone();
    // let company = insert_args.clone().company_name.unwrap().clone();
    // let quote = insert_args.clone().quote.unwrap().clone();

    let application_date = global_var.get_today_str();

    let conn = &mut establish_connection_postgres().unwrap();

    if let Some(id) = check_if_entry_exists(job_title, company, quote) {
        info!("Entry already exists with id: {id}");

        return Ok(cv::table.find(id).first(conn).expect("Error loading CV"));
    }

    let my_quote = match quote {
        Some(q) => q,
        None => &String::new(),
    };

    let new_cv = NewCv {
        application_date: Some(&application_date),
        job_title,
        company,
        quote: my_quote,
        pdf_cv_path: cv_path,
        generated: true,
    };

    Ok(diesel::insert_into(cv::table)
        .values(&new_cv)
        .returning(Cv::as_returning())
        .get_result(conn)
        .expect("Error saving new CV"))
}

pub fn read_cv_from_db(filters: &FilterArgs) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use rusty_cv_creator::schema::cv::dsl::cv;

    let conn = &mut establish_connection_postgres()?;

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

    Ok(pdf_cvs)
}
