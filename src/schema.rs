// @generated automatically by Diesel CLI.

diesel::table! {
    cv (id) {
        id -> Integer,
        application_date -> Nullable<Text>,
        job_title -> Text,
        company -> Text,
        quote -> Text,
        pdf_cv_path -> Text,
        generated -> Bool,
    }
}
