// @generated automatically by Diesel CLI.

diesel::table! {
    releases (id) {
        id -> Int4,
        date -> Date,
        name -> Varchar,
        directory_name -> Nullable<Varchar>,
        file_count -> Nullable<Int2>,
        size -> Nullable<Int8>,
        torrent_url -> Nullable<Varchar>,
    }
}
