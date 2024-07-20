use diesel::prelude::*;
use crate::schema::cv;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = cv)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Cv {
    pub id: i32,
    pub application_date: Option<String>,
    pub job_title: String,
    pub company: String,
    pub quote: String,
    pub pdf_cv_path: String,
    pub pdf_cv: Vec<u8>,
    pub generated: bool
}

#[derive(Insertable)]
#[diesel(table_name = cv)]
pub struct NewCv<'a> {
    pub application_date: Option<&'a str>,
    pub job_title: &'a str,
    pub company: &'a str,
    pub quote: &'a str,
    pub pdf_cv_path: &'a str,
    pub pdf_cv: &'a [u8],
    pub generated: bool
}
