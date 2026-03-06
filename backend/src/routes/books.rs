use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::auth::{extract_library_id, TokenStore};
use crate::db::DbPool;
use crate::models::{Book, NewBook, Author, NewAuthor, Tag, BookAuthor, BookTag};
use crate::schema::{books, authors, tags, book_authors, book_tags};

pub fn router() -> Router<(DbPool, TokenStore)> {
    Router::new()
        .route("/", get(list_books).post(create_book))
        .route("/{id}", get(get_book).put(update_book).delete(delete_book))
        .route("/{id}/archive", post(toggle_archive))
}

#[derive(Serialize)]
struct BookResponse {
    #[serde(flatten)]
    book: Book,
    authors: Vec<Author>,
    tags: Vec<Tag>,
}

#[derive(Deserialize)]
pub struct AuthorInput {
    first_name: String,
    last_name: String,
}

#[derive(Deserialize)]
pub struct CreateBookRequest {
    title: String,
    scan_date: String,
    isbn: Option<String>,
    cover_url: Option<String>,
    authors: Vec<AuthorInput>,
    tag_ids: Vec<i32>,
}

fn load_book_response(conn: &mut SqliteConnection, book: Book, library_id: i32) -> Result<BookResponse, diesel::result::Error> {
    let author_ids: Vec<i32> = book_authors::table
        .filter(book_authors::book_id.eq(book.id))
        .select(book_authors::author_id)
        .load(conn)?;
    let book_authors = authors::table
        .filter(authors::library_id.eq(library_id))
        .filter(authors::id.eq_any(&author_ids))
        .load::<Author>(conn)?;
    let tag_ids: Vec<i32> = book_tags::table
        .filter(book_tags::book_id.eq(book.id))
        .select(book_tags::tag_id)
        .load(conn)?;
    let book_tags = tags::table
        .filter(tags::library_id.eq(library_id))
        .filter(tags::id.eq_any(&tag_ids))
        .load::<Tag>(conn)?;
    Ok(BookResponse { book, authors: book_authors, tags: book_tags })
}

async fn list_books(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<BookResponse>>, StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let all_books = books::table
        .filter(books::library_id.eq(library_id))
        .load::<Book>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut results = Vec::new();
    for book in all_books {
        let resp = load_book_response(&mut conn, book, library_id)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        results.push(resp);
    }
    Ok(Json(results))
}

async fn get_book(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Path(id): Path<i32>,
) -> Result<Json<BookResponse>, StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let book = books::table
        .filter(books::library_id.eq(library_id))
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let resp = load_book_response(&mut conn, book, library_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(resp))
}

async fn create_book(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Json(req): Json<CreateBookRequest>,
) -> Result<(StatusCode, Json<BookResponse>), StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_book = NewBook {
        title: req.title,
        scan_date: req.scan_date,
        isbn: req.isbn,
        cover_url: req.cover_url,
        library_id,
        archived: false,
    };
    diesel::insert_into(books::table)
        .values(&new_book)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let book = books::table
        .filter(books::library_id.eq(library_id))
        .order(books::id.desc())
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for a in &req.authors {
        let author = authors::table
            .filter(authors::library_id.eq(library_id))
            .filter(authors::first_name.eq(&a.first_name))
            .filter(authors::last_name.eq(&a.last_name))
            .first::<Author>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let author_id = match author {
            Some(existing) => existing.id,
            None => {
                diesel::insert_into(authors::table)
                    .values(&NewAuthor {
                        first_name: a.first_name.clone(),
                        last_name: a.last_name.clone(),
                        library_id,
                    })
                    .execute(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                authors::table
                    .filter(authors::library_id.eq(library_id))
                    .order(authors::id.desc())
                    .first::<Author>(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .id
            }
        };
        diesel::insert_into(book_authors::table)
            .values(&BookAuthor { book_id: book.id, author_id })
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    for tag_id in &req.tag_ids {
        diesel::insert_into(book_tags::table)
            .values(&BookTag { book_id: book.id, tag_id: *tag_id })
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let resp = load_book_response(&mut conn, book, library_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn update_book(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Path(id): Path<i32>,
    Json(req): Json<CreateBookRequest>,
) -> Result<Json<BookResponse>, StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _existing = books::table
        .filter(books::library_id.eq(library_id))
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    diesel::update(books::table.filter(books::library_id.eq(library_id)).find(id))
        .set((
            books::title.eq(&req.title),
            books::scan_date.eq(&req.scan_date),
            books::isbn.eq(&req.isbn),
            books::cover_url.eq(&req.cover_url),
        ))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::delete(book_authors::table.filter(book_authors::book_id.eq(id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for a in &req.authors {
        let author = authors::table
            .filter(authors::library_id.eq(library_id))
            .filter(authors::first_name.eq(&a.first_name))
            .filter(authors::last_name.eq(&a.last_name))
            .first::<Author>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let author_id = match author {
            Some(existing) => existing.id,
            None => {
                diesel::insert_into(authors::table)
                    .values(&NewAuthor {
                        first_name: a.first_name.clone(),
                        last_name: a.last_name.clone(),
                        library_id,
                    })
                    .execute(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                authors::table
                    .filter(authors::library_id.eq(library_id))
                    .order(authors::id.desc())
                    .first::<Author>(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .id
            }
        };
        diesel::insert_into(book_authors::table)
            .values(&BookAuthor { book_id: id, author_id })
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    diesel::delete(book_tags::table.filter(book_tags::book_id.eq(id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for tag_id in &req.tag_ids {
        diesel::insert_into(book_tags::table)
            .values(&BookTag { book_id: id, tag_id: *tag_id })
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let book = books::table
        .filter(books::library_id.eq(library_id))
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let resp = load_book_response(&mut conn, book, library_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(resp))
}

async fn delete_book(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Path(id): Path<i32>,
) -> StatusCode {
    let library_id = match extract_library_id(&token_store, auth) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let deleted = diesel::delete(
        books::table
            .filter(books::library_id.eq(library_id))
            .filter(books::id.eq(id))
    )
    .execute(&mut conn)
    .unwrap_or(0);
    if deleted > 0 { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}

async fn toggle_archive(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Path(id): Path<i32>,
) -> Result<Json<BookResponse>, StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let book = books::table
        .filter(books::library_id.eq(library_id))
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    diesel::update(books::table.filter(books::library_id.eq(library_id)).find(id))
        .set(books::archived.eq(!book.archived))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let updated = books::table
        .filter(books::library_id.eq(library_id))
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp = load_book_response(&mut conn, updated, library_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(resp))
}
