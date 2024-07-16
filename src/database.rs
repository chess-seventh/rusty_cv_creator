use diesel::prelude::*;
use log::{info, error};
use std::env;
use std::path::Path;
use rusty_cv_creator::models::Cv;
use rusty_cv_creator::models::NewCv;
use rusty_cv_creator::schema::cv;
use crate::helpers;
use crate::file_handlers;

extern crate skim;
use skim::prelude::*;
use std::io::Cursor;


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

fn read_cv_from_database() -> Vec<String> {
    use rusty_cv_creator::schema::cv::dsl::*;

    let conn = &mut establish_connection();
    let cv_results = cv
        .limit(20)
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

pub fn show_cvs() -> String {

    let pdfs = read_cv_from_database();

    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(false)
        .build()
        .unwrap();

    let input: String = pdfs.into_iter().collect();

    // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
    // `SkimItem` was implemented for `AsRef<str>` by default
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    // `run_with` would read and show items from the stream
    let selected_items = Skim::run_with(&options, Some(items)).map_or_else(Vec::new, |out| out.selected_items);

    if selected_items.len() == 1 {
        selected_items.first().expect("Should have had at least one item").output().to_string()
    } else {
        panic!("shit");
    }
}


pub fn remove_cv() -> String {
    use rusty_cv_creator::schema::cv::dsl::*;

    let conn = &mut establish_connection();

    let cv_remove = show_cvs();

    let pattern = format!("%{cv_remove}%");

    let num_deleted = diesel::delete(cv.filter(pdf_cv_path.like(pattern)))
        .execute(conn)
        .expect("Error deleting posts");

    println!("Deleted the {num_deleted:}");
    let dir_of_cv_path = Path::new(&cv_remove).parent().unwrap();

    print!("{dir_of_cv_path:?}");

    if let Ok(()) = file_handlers::remove_cv_dir(dir_of_cv_path) {
        info!("removed dir_of_cv_path");
    } else {
        error!("couldn't remove dir");
    }

    String::new()
}
