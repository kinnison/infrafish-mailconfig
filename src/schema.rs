// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "mailentrykind"))]
    pub struct Mailentrykind;
}

diesel::table! {
    maildomain (id) {
        id -> Int4,
        owner -> Int4,
        domainname -> Varchar,
        remotemx -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Mailentrykind;

    mailentry (id) {
        id -> Int4,
        maildomain -> Int4,
        name -> Varchar,
        kind -> Mailentrykind,
        password -> Nullable<Varchar>,
        expansion -> Nullable<Varchar>,
    }
}

diesel::table! {
    mailuser (id) {
        id -> Int4,
        username -> Varchar,
    }
}

diesel::joinable!(maildomain -> mailuser (owner));
diesel::joinable!(mailentry -> maildomain (maildomain));

diesel::allow_tables_to_appear_in_same_query!(
    maildomain,
    mailentry,
    mailuser,
);
