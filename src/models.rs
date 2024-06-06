use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::cvs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Cvs {
    pub id: i32,
    pub job_title: String,
    pub company: String,
    pub quote: Option<String>,
    pub generated: bool
}
