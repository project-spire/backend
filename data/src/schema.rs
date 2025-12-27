// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "location"))]
    pub struct Location;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "race"))]
    pub struct Race;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vector3"))]
    pub struct Vector3;
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
    use super::sql_types::Location;
    use super::sql_types::Vector3;

    character (id) {
        id -> Int8,
        account_id -> Int8,
        created_at -> Timestamptz,
        #[max_length = 16]
        name -> Varchar,
        race -> Race,
        level -> Int2,
        exp -> Int8,
        karma -> Int8,
        location -> Location,
        position -> Vector3,
        health -> Int8,
        mana -> Nullable<Int8>,
        gold -> Int8,
    }
}

diesel::table! {
    character_path (character_id, data_id) {
        character_id -> Int8,
        data_id -> Int4,
        is_active -> Bool,
        level -> Int2,
        exp -> Int4,
    }
}

diesel::table! {
    character_talent (character_id, data_id) {
        character_id -> Int8,
        data_id -> Int4,
        is_active -> Bool,
        level -> Int2,
        exp -> Int4,
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
diesel::joinable!(character_path -> character (character_id));
diesel::joinable!(character_talent -> character (character_id));
diesel::joinable!(dev_account -> account (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    character,
    character_path,
    character_talent,
    dev_account,
);
