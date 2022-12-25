// @generated automatically by Diesel CLI.

diesel::table! {
    music (id) {
        id -> Integer,
        title -> Varchar,
        artist -> Varchar,
        file_path -> Varchar,
        file_size -> Unsigned<Integer>,
    }
}
