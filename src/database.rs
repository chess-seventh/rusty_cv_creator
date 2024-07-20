use diesel::prelude::*;
use std::env;
use rusty_cv_creator::models::Cv;
use rusty_cv_creator::models::NewCv;
use rusty_cv_creator::schema::cv;
use crate::helpers;

extern crate skim;


pub fn establish_connection() -> SqliteConnection {
    let database_url = &env::var("DATABASE_URL").unwrap_or_else(|_| {
        helpers::check_if_db_env_is_set_or_set_from_config();
        env::var("DATABASE_URL").unwrap()
    });
    let db = helpers::fix_home_directory_path(database_url);
    println!("{db:?}");
    SqliteConnection::establish(&db)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

pub fn save_new_cv_to_database(job_title: &str, company: &str, cv_path: &str, quote: &str) -> Cv {
    let conn = &mut establish_connection();

    let decoded_cv = helpers::read_destination_cv_file(cv_path);
    let new_cv = NewCv {
        job_title,
        company,
        quote,
        pdf_cv_path: cv_path,
        pdf_cv: &decoded_cv,
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

    let conn = &mut establish_connection();
    // TODO filters on proper DB
    let cv_results = cv
        .limit(20)
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

