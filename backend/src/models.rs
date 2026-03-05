use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// --- Libraries ---

#[derive(Queryable, Serialize)]
pub struct Library {
    pub id: i32,
    pub name: String,
    #[serde(skip)]
    pub passkey_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::libraries)]
pub struct NewLibrary {
    pub name: String,
    pub passkey_hash: String,
}

// --- Books ---

#[derive(Queryable, Serialize)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub scan_date: String,
    pub isbn: Option<String>,
    pub cover_url: Option<String>,
    pub library_id: i32,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::books)]
pub struct NewBook {
    pub title: String,
    pub scan_date: String,
    pub isbn: Option<String>,
    pub cover_url: Option<String>,
    pub library_id: i32,
}

// --- Authors ---

#[derive(Queryable, Serialize)]
pub struct Author {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub library_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::authors)]
pub struct NewAuthor {
    pub first_name: String,
    pub last_name: String,
    pub library_id: i32,
}

// --- Tags (genres, owners, custom tags) ---

#[derive(Queryable, Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub kind: String,
    pub library_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tags)]
pub struct NewTag {
    pub name: String,
    pub kind: String,
    pub library_id: i32,
}

#[derive(Deserialize)]
pub struct NewTagRequest {
    pub name: String,
    pub kind: String,
}

// --- Join tables ---

#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::book_authors)]
pub struct BookAuthor {
    pub book_id: i32,
    pub author_id: i32,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::book_tags)]
pub struct BookTag {
    pub book_id: i32,
    pub tag_id: i32,
}
