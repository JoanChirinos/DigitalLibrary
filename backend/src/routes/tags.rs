use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use diesel::prelude::*;
use serde::Deserialize;
use crate::auth::{extract_library_id, TokenStore};
use crate::db::DbPool;
use crate::models::{Tag, NewTag, NewTagRequest};
use crate::schema::tags;

pub fn router() -> Router<(DbPool, TokenStore)> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/{id}", get(get_tag).delete(delete_tag))
}

#[derive(Deserialize)]
pub struct TagFilter {
    kind: Option<String>,
}

async fn list_tags(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Query(filter): Query<TagFilter>,
) -> Result<Json<Vec<Tag>>, StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut query = tags::table.filter(tags::library_id.eq(library_id)).into_boxed();
    if let Some(kind) = filter.kind {
        query = query.filter(tags::kind.eq(kind));
    }
    let results = query.load::<Tag>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(results))
}

async fn create_tag(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Json(req): Json<NewTagRequest>,
) -> Result<(StatusCode, Json<Tag>), StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let new_tag = NewTag {
        name: req.name,
        kind: req.kind,
        library_id,
    };
    diesel::insert_into(tags::table)
        .values(&new_tag)
        .execute(&mut conn)
        .map_err(|_| StatusCode::CONFLICT)?;
    let tag = tags::table
        .filter(tags::library_id.eq(library_id))
        .order(tags::id.desc())
        .first::<Tag>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(tag)))
}

async fn get_tag(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    auth: TypedHeader<Authorization<Bearer>>,
    Path(id): Path<i32>,
) -> Result<Json<Tag>, StatusCode> {
    let library_id = extract_library_id(&token_store, auth)?;
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tag = tags::table
        .filter(tags::library_id.eq(library_id))
        .find(id)
        .first::<Tag>(&mut conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(tag))
}

async fn delete_tag(
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
        tags::table
            .filter(tags::library_id.eq(library_id))
            .filter(tags::id.eq(id))
    )
    .execute(&mut conn)
    .unwrap_or(0);
    if deleted > 0 { StatusCode::NO_CONTENT } else { StatusCode::NOT_FOUND }
}
