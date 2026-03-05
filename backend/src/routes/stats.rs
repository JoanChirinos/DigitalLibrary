use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use diesel::prelude::*;
use diesel::dsl::count;
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use crate::schema::{books, authors, tags, book_authors, book_tags};

pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/totals", get(totals))
        .route("/by-tag", get(by_tag))
        .route("/by-author", get(by_author))
        .route("/growth", get(growth))
}

// --- Shared ---

#[derive(Deserialize)]
pub struct StatsFilter {
    tags: Option<String>,
    start: Option<String>,
    end: Option<String>,
}

fn parse_tag_ids(tags: &Option<String>) -> Vec<i32> {
    tags.as_ref()
        .map(|s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect())
        .unwrap_or_default()
}

fn filtered_book_ids(
    conn: &mut SqliteConnection,
    tag_ids: &[i32],
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<i32>, diesel::result::Error> {
    let mut query = books::table.into_boxed().select(books::id);

    if !tag_ids.is_empty() {
        let matching: Vec<i32> = book_tags::table
            .filter(book_tags::tag_id.eq_any(tag_ids))
            .group_by(book_tags::book_id)
            .having(count(book_tags::tag_id).eq(tag_ids.len() as i64))
            .select(book_tags::book_id)
            .load(conn)?;
        query = query.filter(books::id.eq_any(matching));
    }
    if let Some(s) = start {
        query = query.filter(books::scan_date.ge(s));
    }
    if let Some(e) = end {
        query = query.filter(books::scan_date.le(e));
    }

    query.load(conn)
}

// --- /stats/totals ---

#[derive(Serialize)]
struct Totals {
    books: i64,
    authors: i64,
    tags: i64,
}

async fn totals(
    State(pool): State<DbPool>,
    Query(filter): Query<StatsFilter>,
) -> Result<Json<Totals>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tag_ids = parse_tag_ids(&filter.tags);
    let ids = filtered_book_ids(&mut conn, &tag_ids, &filter.start, &filter.end)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let book_count = ids.len() as i64;
    let author_count: i64 = authors::table.count().get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tag_count: i64 = tags::table.count().get_result(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Totals { books: book_count, authors: author_count, tags: tag_count }))
}

// --- /stats/by-tag ---

#[derive(Deserialize)]
pub struct ByTagFilter {
    kind: Option<String>,
    #[serde(flatten)]
    base: StatsFilter,
}

#[derive(Serialize, Queryable)]
struct TagCount {
    tag_name: String,
    tag_kind: String,
    count: i64,
}

async fn by_tag(
    State(pool): State<DbPool>,
    Query(filter): Query<ByTagFilter>,
) -> Result<Json<Vec<TagCount>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tag_ids = parse_tag_ids(&filter.base.tags);
    let ids = filtered_book_ids(&mut conn, &tag_ids, &filter.base.start, &filter.base.end)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut query = book_tags::table
        .inner_join(tags::table)
        .filter(book_tags::book_id.eq_any(&ids))
        .group_by((tags::name, tags::kind))
        .select((tags::name, tags::kind, count(book_tags::book_id)))
        .order(count(book_tags::book_id).desc())
        .into_boxed();

    if let Some(ref k) = filter.kind {
        query = query.filter(tags::kind.eq(k));
    }

    let results: Vec<TagCount> = query.load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(results))
}

// --- /stats/by-author ---

#[derive(Serialize)]
struct AuthorCount {
    first_name: String,
    last_name: String,
    count: i64,
}

async fn by_author(
    State(pool): State<DbPool>,
    Query(filter): Query<StatsFilter>,
) -> Result<Json<Vec<AuthorCount>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tag_ids = parse_tag_ids(&filter.tags);
    let ids = filtered_book_ids(&mut conn, &tag_ids, &filter.start, &filter.end)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let all_authors: Vec<(i32, String, String)> = authors::table
        .select((authors::id, authors::first_name, authors::last_name))
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut results = Vec::new();
    for (author_id, first_name, last_name) in all_authors {
        let c: i64 = book_authors::table
            .filter(book_authors::author_id.eq(author_id))
            .filter(book_authors::book_id.eq_any(&ids))
            .count()
            .get_result(&mut conn)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if c > 0 {
            results.push(AuthorCount { first_name, last_name, count: c });
        }
    }
    results.sort_by(|a, b| a.last_name.cmp(&b.last_name));
    Ok(Json(results))
}

// --- /stats/growth ---

#[derive(Deserialize)]
pub struct GrowthFilter {
    group_by: Option<String>,
    #[serde(flatten)]
    base: StatsFilter,
}

#[derive(Serialize)]
struct GrowthBucket {
    period: String,
    count: i64,
}

async fn growth(
    State(pool): State<DbPool>,
    Query(filter): Query<GrowthFilter>,
) -> Result<Json<Vec<GrowthBucket>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tag_ids = parse_tag_ids(&filter.base.tags);
    let ids = filtered_book_ids(&mut conn, &tag_ids, &filter.base.start, &filter.base.end)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dates: Vec<String> = books::table
        .filter(books::id.eq_any(&ids))
        .select(books::scan_date)
        .order(books::scan_date.asc())
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let truncate = match filter.group_by.as_deref().unwrap_or("month") {
        "day" => 10,
        "year" => 4,
        _ => 7,
    };

    let mut buckets: Vec<GrowthBucket> = Vec::new();
    for date in dates {
        let period: String = date.chars().take(truncate).collect();
        if let Some(last) = buckets.last_mut() {
            if last.period == period {
                last.count += 1;
                continue;
            }
        }
        buckets.push(GrowthBucket { period, count: 1 });
    }

    Ok(Json(buckets))
}
