use diesel::prelude::*;
use std::env;
use rusty_cv_creator::models::Cv;
use rusty_cv_creator::models::NewCv;
use rusty_cv_creator::schema::cv;
use crate::helpers;

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

pub fn save_new_cv_to_database(job_title: &str, company: &str, cv_path: &str, quote: &str) -> Cv {
    let conn = &mut establish_connection();

    let decoded_cv = helpers::read_destination_cv_file(cv_path);
    let new_post = NewCv {
        job_title,
        company,
        quote,
        pdf_cv_path: cv_path,
        pdf_cv: &decoded_cv,
        generated: true
    };

    diesel::insert_into(cv::table)
        .values(&new_post)
        .returning(Cv::as_returning())
        .get_result(conn)
        .expect("Error saving new CV")
}
