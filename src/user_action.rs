use crate::cli_structure::FilterArgs;
use crate::database::{
    establish_connection_postgres, read_cv_from_database, save_new_cv_to_database,
};
use crate::file_handlers;
use crate::global_conf::GLOBAL_VAR;
use crate::helpers::my_fzf;
use crate::prepare_cv;
use diesel::prelude::*;
use log::{error, info, warn};
use std::path::Path;

pub fn show_cvs(filters: &FilterArgs) -> String {
    // TODO: apply filters
    println!("TODO: apply these filters: {filters:?}");
    let pdfs = read_cv_from_database(filters);
    my_fzf(pdfs)
}

pub fn remove_cv(filters: &FilterArgs) {
    use rusty_cv_creator::schema::cv::dsl::{cv, pdf_cv_path};

    let conn = &mut establish_connection_postgres();

    let cv_remove = show_cvs(filters);

    let pattern = format!("%{cv_remove}%");

    let _ = diesel::delete(cv.filter(pdf_cv_path.like(pattern)))
        .execute(conn)
        .expect("Error deleting cvs");

    let dir_of_cv_path = Path::new(&cv_remove).parent().unwrap();

    if let Ok(()) = file_handlers::remove_cv_dir(dir_of_cv_path) {
        info!("Removed dir_of_cv_path");
    } else {
        error!("Couldn't remove dir: {}", dir_of_cv_path.display());
        panic!("Couldn't remove dir: {}", dir_of_cv_path.display());
    }
}

pub fn insert_cv() -> String {
    let save_to_db = GLOBAL_VAR.get().unwrap().get_user_input_save_to_db();
    let job_title = GLOBAL_VAR.get().unwrap().get_user_job_title();
    let company_name = GLOBAL_VAR.get().unwrap().get_user_input_company_name();
    let quote = GLOBAL_VAR.get().unwrap().get_user_input_quote();
    let destination_cv_file_full_path = prepare_cv(&job_title, &company_name, &quote);

    if save_to_db {
        let _db_cv = save_new_cv_to_database(&destination_cv_file_full_path);
        info!("Saved CV to database");
    } else {
        warn!("CV NOT SAVED TO DATABASE!");
    }

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
