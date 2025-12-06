// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "race"))]
    pub struct Race;
}

diesel::table! {
    account (id) {
        id -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Race;

    character (id) {
        id -> Int8,
        account_id -> Int8,
        created_at -> Timestamptz,
        #[max_length = 16]
        name -> Varchar,
        race -> Race,
    }
}

diesel::table! {
    dev_account (id) {
        #[max_length = 16]
        id -> Varchar,
        account_id -> Int8,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(character -> account (account_id));
diesel::joinable!(dev_account -> account (account_id));

diesel::allow_tables_to_appear_in_same_query!(account, character, dev_account,);
