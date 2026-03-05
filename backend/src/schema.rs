diesel::table! {
    books (id) {
        id -> Integer,
        title -> Text,
        scan_date -> Text,
        isbn -> Nullable<Text>,
        cover_url -> Nullable<Text>,
    }
}

diesel::table! {
    authors (id) {
        id -> Integer,
        first_name -> Text,
        last_name -> Text,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
        kind -> Text,
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

diesel::allow_tables_to_appear_in_same_query!(
    books, authors, tags,
    book_authors, book_tags,
);
