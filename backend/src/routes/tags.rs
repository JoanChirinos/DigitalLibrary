use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use crate::db::DbPool;
use crate::models::{Tag, NewTag};
use crate::schema::tags;

pub fn router() -> Router<DbPool> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/{id}", get(get_tag).delete(delete_tag))
}

#[derive(Deserialize)]
pub struct TagFilter {
    kind: Option<String>,
}

async fn list_tags(
    State(pool): State<DbPool>,
    Query(filter): Query<TagFilter>,
) -> Result<Json<Vec<Tag>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut query = tags::table.into_boxed();
    if let Some(kind) = filter.kind {
        query = query.filter(tags::kind.eq(kind));
    }
    let results = query.load::<Tag>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(results))
}

async fn create_tag(
    State(pool): State<DbPool>,
    Json(new_tag): Json<NewTag>,
) -> Result<(StatusCode, Json<Tag>), StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    diesel::insert_into(tags::table)
        .values(&new_tag)
        .execute(&mut conn)
        .map_err(|_| StatusCode::CONFLICT)?;
    let tag = tags::table
        .order(tags::id.desc())
        .first::<Tag>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(tag)))
}

async fn get_tag(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
) -> Result<Json<Tag>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tag = tags::table
        .find(id)
        .first::<Tag>(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(tag))
}

async fn delete_tag(
    State(pool): State<DbPool>,
    Path(id): Path<i32>,
) -> StatusCode {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let deleted = diesel::delete(tags::table.find(id))
        .execute(&mut conn)
        .unwrap_or(0);
    if deleted > 0 { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}
