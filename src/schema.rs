// @generated automatically by Diesel CLI.

diesel::table! {
    cv (id) {
        id -> Int4,
        application_date -> Nullable<Varchar>,
        job_title -> Varchar,
        company -> Varchar,
        quote -> Varchar,
        pdf_cv_path -> Varchar,
        generated -> Bool,
    }
}
