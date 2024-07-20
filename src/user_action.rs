use log::{info, warn, error};
use std::path::Path;
use diesel::prelude::*;
use skim::prelude::*;
use std::io::Cursor;

use crate::cli_structure;
use crate::prepare_cv;
use crate::database::{read_cv_from_database, establish_connection, save_new_cv_to_database};
use crate::file_handlers;


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
    use rusty_cv_creator::schema::cv::dsl::{cv, pdf_cv_path};

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
        panic!("couldn't remove dir");
    }

    String::new()
}

pub fn insert_cv(save_to_db: bool, insert: &cli_structure::InsertCV) -> String {
    let destination_cv_file_full_path = prepare_cv(&insert.job_title, &insert.company_name, &insert.quote);
    if save_to_db {
        let _db_cv = save_new_cv_to_database(&insert.job_title, &insert.company_name, &destination_cv_file_full_path, &insert.quote);
        info!("Saved CV to database");
    } else {
        warn!("CV NOT SAVED TO DATABASE!");
    };
    destination_cv_file_full_path
}

// pub fn list_cvs_by_date(_filter_date: NaiveDateTime) {
//     println!("Running list_cvs_by_date!");
// }

// pub fn list_cvs_by_word(_filter_word: String) {
//     println!("Running list_cvs_by_word!");
// }

// pub fn list_cvs(_filter_date: NaiveDateTime, _filter_word: String) {
//     println!("Running list_cvs!");
// }

