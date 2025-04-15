use crate::cli_structure::FilterArgs;
use crate::database::{establish_connection_postgres, read_cv_from_db, save_new_cv_to_db};
use crate::file_handlers;
use crate::global_conf::GLOBAL_VAR;
use crate::helpers::my_fzf;
use crate::prepare_cv;
use diesel::prelude::*;
use log::{error, info, warn};
use std::path::Path;

pub fn show_cvs(filters: &FilterArgs) -> String {
    // TODO:
    // apply filters
    println!("TODO: apply these filters: {filters:?}");
    let pdfs = read_cv_from_db(filters);
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

pub fn insert_cv() -> Result<String, String> {
    let global_var = if let Some(v) = GLOBAL_VAR.get() {
        info!("Could get GLOBAL_VAR");
        v
    } else {
        error!("Could not get GLOBAL_VAR, something is wrong");
        return Err("Could not get GLOBAL_VAR, something is wrong".to_string());
    };

    let save_to_db = global_var.get_user_input_save_to_db();

    let job_title = global_var.get_user_job_title();

    let company_name = global_var.get_user_input_company_name();

    let quote = global_var.get_user_input_quote();

    let destination_cv_file_full_path = match prepare_cv(&job_title, &company_name, &quote) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not get the destination_cv_file_full_path: {e:?}");
            return Err("Could not get the destination_cv_file_full_path: {e:?}".to_string());
        }
    };

    if save_to_db {
        let _db_cv = save_new_cv_to_db(&destination_cv_file_full_path);
        info!("Saved CV to database");
    } else {
        warn!("CV NOT SAVED TO DATABASE!");
    }

    Ok(destination_cv_file_full_path)
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
