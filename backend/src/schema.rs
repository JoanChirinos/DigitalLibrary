diesel::table! {
    libraries (id) {
        id -> Integer,
        name -> Text,
        passkey_hash -> Text,
    }
}

diesel::table! {
    books (id) {
        id -> Integer,
        title -> Text,
        scan_date -> Text,
        isbn -> Nullable<Text>,
        cover_url -> Nullable<Text>,
        library_id -> Integer,
    }
}

diesel::table! {
    authors (id) {
        id -> Integer,
        first_name -> Text,
        last_name -> Text,
        library_id -> Integer,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
        kind -> Text,
        library_id -> Integer,
    }
}

diesel::table! {
    book_authors (book_id, author_id) {
        book_id -> Integer,
        author_id -> Integer,
    }
}

diesel::table! {
    book_tags (book_id, tag_id) {
        book_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::joinable!(book_authors -> books (book_id));
diesel::joinable!(book_authors -> authors (author_id));
diesel::joinable!(book_tags -> books (book_id));
diesel::joinable!(book_tags -> tags (tag_id));
diesel::joinable!(books -> libraries (library_id));
diesel::joinable!(authors -> libraries (library_id));
diesel::joinable!(tags -> libraries (library_id));

diesel::allow_tables_to_appear_in_same_query!(
    libraries, books, authors, tags,
    book_authors, book_tags,
);
