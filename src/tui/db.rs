use crate::models::Cv;
use crate::schema::cv::dsl::cv as cv_dsl;
use diesel::prelude::*;

pub fn load_all_applications() -> Result<Vec<Cv>, Box<dyn std::error::Error>> {
    let db_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set")?;
    let conn = &mut PgConnection::establish(&db_url)?;
    Ok(cv_dsl.select(Cv::as_select()).load(conn)?)
}
