use log::{info, warn, error};
use std::path::Path;
use diesel::prelude::*;

use crate::global_conf::GlobalVars;
use crate::helpers::my_fzf;
use crate::prepare_cv;
use crate::database::{establish_connection_postgres, read_cv_from_database, save_new_cv_to_database};
use crate::file_handlers;


pub fn show_cvs() -> String {
    let pdfs = read_cv_from_database();
    my_fzf(pdfs)
}

pub fn remove_cv() {
    use rusty_cv_creator::schema::cv::dsl::{cv, pdf_cv_path};

    let conn = &mut establish_connection_postgres();

    let cv_remove = show_cvs();

    let pattern = format!("%{cv_remove}%");

    let _ = diesel::delete(cv.filter(pdf_cv_path.like(pattern)))
        .execute(conn)
        .expect("Error deleting posts");

    let dir_of_cv_path = Path::new(&cv_remove).parent().unwrap();

    if let Ok(()) = file_handlers::remove_cv_dir(dir_of_cv_path) {
        info!("Removed dir_of_cv_path");
    } else {
        error!("Couldn't remove dir: {}", dir_of_cv_path.display());
        panic!("Couldn't remove dir: {}", dir_of_cv_path.display());
    }
}

pub fn insert_cv() -> String {
    let save_to_db = GlobalVars::get_user_save_to_db();
    let job_title = GlobalVars::get_user_job_title();
    let company_name = GlobalVars::get_user_company_name();
    let quote = GlobalVars::get_user_quote();
    let destination_cv_file_full_path = prepare_cv(&job_title, &company_name, &quote);

    if save_to_db {
        // let _db_cv = save_new_cv_to_database(&job_title, &company_name, &destination_cv_file_full_path, &quote, Some(&now));
        let _db_cv = save_new_cv_to_database(&destination_cv_file_full_path);
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

