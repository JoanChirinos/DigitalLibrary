use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use crate::models::{Book, NewBook, Author, NewAuthor, Tag, BookAuthor, BookTag};
use crate::schema::{books, authors, tags, book_authors, book_tags};

pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_books).post(create_book))
        .route("/{id}", get(get_book).put(update_book).delete(delete_book))
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

fn load_book_response(conn: &mut SqliteConnection, book: Book) -> Result<BookResponse, diesel::result::Error> {
    let author_ids: Vec<i32> = book_authors::table
        .filter(book_authors::book_id.eq(book.id))
        .select(book_authors::author_id)
        .load(conn)?;
    let book_authors = authors::table
        .filter(authors::id.eq_any(&author_ids))
        .load::<Author>(conn)?;
    let tag_ids: Vec<i32> = book_tags::table
        .filter(book_tags::book_id.eq(book.id))
        .select(book_tags::tag_id)
        .load(conn)?;
    let book_tags = tags::table
        .filter(tags::id.eq_any(&tag_ids))
        .load::<Tag>(conn)?;
    Ok(BookResponse { book, authors: book_authors, tags: book_tags })
}

async fn list_books(State(pool): State<DbPool>) -> Result<Json<Vec<BookResponse>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let all_books = books::table
        .load::<Book>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut results = Vec::new();
    for book in all_books {
        let resp = load_book_response(&mut conn, book)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        results.push(resp);
    }
    Ok(Json(results))
}

async fn get_book(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<BookResponse>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let book = books::table
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let resp = load_book_response(&mut conn, book)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(resp))
}

async fn create_book(
    State(pool): State<DbPool>,
    Json(req): Json<CreateBookRequest>,
) -> Result<(StatusCode, Json<BookResponse>), StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert the book
    let new_book = NewBook {
        title: req.title,
        scan_date: req.scan_date,
        isbn: req.isbn,
        cover_url: req.cover_url,
    };
    diesel::insert_into(books::table)
        .values(&new_book)
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let book = books::table
        .order(books::id.desc())
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Find-or-create authors and link them
    for a in &req.authors {
        let author = authors::table
            .filter(authors::first_name.eq(&a.first_name))
            .filter(authors::last_name.eq(&a.last_name))
            .first::<Author>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let author_id = match author {
            Some(existing) => existing.id,
            None => {
                diesel::insert_into(authors::table)
                    .values(&NewAuthor { first_name: a.first_name.clone(), last_name: a.last_name.clone() })
                    .execute(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                authors::table
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

    // Link tags
    for tag_id in &req.tag_ids {
        diesel::insert_into(book_tags::table)
            .values(&BookTag { book_id: book.id, tag_id: *tag_id })
            .execute(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let resp = load_book_response(&mut conn, book)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn update_book(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
    Json(req): Json<CreateBookRequest>,
) -> Result<Json<BookResponse>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify book exists
    let _existing = books::table
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Update book fields
    diesel::update(books::table.find(id))
        .set((
            books::title.eq(&req.title),
            books::scan_date.eq(&req.scan_date),
            books::isbn.eq(&req.isbn),
            books::cover_url.eq(&req.cover_url),
        ))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Replace authors: clear old, insert new
    diesel::delete(book_authors::table.filter(book_authors::book_id.eq(id)))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for a in &req.authors {
        let author = authors::table
            .filter(authors::first_name.eq(&a.first_name))
            .filter(authors::last_name.eq(&a.last_name))
            .first::<Author>(&mut conn)
            .optional()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let author_id = match author {
            Some(existing) => existing.id,
            None => {
                diesel::insert_into(authors::table)
                    .values(&NewAuthor { first_name: a.first_name.clone(), last_name: a.last_name.clone() })
                    .execute(&mut conn)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                authors::table
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

    // Replace tags: clear old, insert new
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
        .find(id)
        .first::<Book>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let resp = load_book_response(&mut conn, book)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(resp))
}

async fn delete_book(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
) -> StatusCode {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let deleted = diesel::delete(books::table.find(id))
        .execute(&mut conn)
        .unwrap_or(0);
    if deleted > 0 { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}
