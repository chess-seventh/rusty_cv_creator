use crate::database::save_new_cv_to_db;
use crate::prepare_cv;
// use crate::cli_structure::FilterArgs;
use crate::global_conf::get_global_var;
use log::{info, warn};

pub fn insert_cv() -> Result<String, Box<dyn std::error::Error>> {
    // These come from the UserInput, FilterArgs
    // TODO: remove unwrap to make sure we don't break stuff
    let job_title = get_global_var().get_job_title()?;
    let company_name = get_global_var().get_company_name()?;
    let quote = get_global_var().get_quote().ok();

    let destination_cv_file_full_path = prepare_cv(&job_title, &company_name, quote.as_ref())?;

    // This comes from the INI file.
    let save_to_db = get_global_var().get_user_input_save_to_db();

    if save_to_db.to_owned() {
        let _db_cv = save_new_cv_to_db(
            &destination_cv_file_full_path,
            &job_title,
            &company_name,
            quote.as_ref(),
        );
        info!("Saved CV to database");
    } else {
        warn!("CV NOT SAVED TO DATABASE!");
    }

    Ok(destination_cv_file_full_path)
}
