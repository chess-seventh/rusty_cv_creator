use crate::cli_structure::FilterArgs;
use crate::database::{establish_connection_postgres, read_cv_from_db};
use crate::file_handlers;
use crate::helpers::my_fzf;
use diesel::prelude::*;
use log::{error, info, warn};
use std::path::Path;

pub fn show_cvs(filters: &FilterArgs) -> String {
    // TODO:
    // apply filters
    warn!("TODO: apply these filters: {filters:?}");
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
