// @generated automatically by Diesel CLI.

diesel::table! {
    cvs (id) {
        id -> Integer,
        job_title -> Text,
        company -> Text,
        quote -> Nullable<Text>,
        generated -> Bool,
    }
}
