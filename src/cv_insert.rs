use crate::database::save_new_cv_to_db;
use crate::global_conf::get_global_var;
use crate::prepare_cv;
use log::{info, warn};

pub fn insert_cv() -> Result<String, Box<dyn std::error::Error>> {
    // These come from the UserInput, FilterArgs
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "GlobalVar Get didn't work")]
    fn test_insert_cv_panics_without_global() {
        let _ = insert_cv().unwrap();
    }
}
