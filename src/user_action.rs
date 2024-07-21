use log::{info, warn, error};
use std::path::Path;
use diesel::prelude::*;

use crate::cli_structure;
use crate::config_parse::GlobalVars;
use crate::helpers::my_fzf;
use crate::prepare_cv;
use crate::database::{read_cv_from_database, establish_connection, save_new_cv_to_database};
use crate::file_handlers;


pub fn show_cvs() -> String {
    let pdfs = read_cv_from_database();
    my_fzf(pdfs)
}

pub fn remove_cv() {
    use rusty_cv_creator::schema::cv::dsl::{cv, pdf_cv_path};

    let conn = &mut establish_connection();

    let cv_remove = show_cvs();

    let pattern = format!("%{cv_remove}%");

    let _ = diesel::delete(cv.filter(pdf_cv_path.like(pattern)))
        .execute(conn)
        .expect("Error deleting posts");

    let dir_of_cv_path = Path::new(&cv_remove).parent().unwrap();

    if let Ok(()) = file_handlers::remove_cv_dir(dir_of_cv_path) {
        info!("Removed dir_of_cv_path");
    } else {
        error!("Couldn't remove dir");
        panic!("Couldn't remove dir");
    }
}

pub fn insert_cv(save_to_db: bool, insert: &cli_structure::InsertCV) -> String {
    let destination_cv_file_full_path = prepare_cv(&insert.job_title, &insert.company_name, &insert.quote);
    let now = GlobalVars::get_today().format("%e-%b-%Y").to_string();
    if save_to_db {
        let _db_cv = save_new_cv_to_database(&insert.job_title, &insert.company_name, &destination_cv_file_full_path, &insert.quote, Some(&now));
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

