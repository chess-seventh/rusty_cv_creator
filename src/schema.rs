// @generated automatically by Diesel CLI.

diesel::table! {
    cv (id) {
        id -> Integer,
        job_title -> Text,
        company -> Text,
        quote -> Text,
        pdf_cv_path -> Text,
        pdf_cv -> Binary,
        generated -> Bool,
    }
}
