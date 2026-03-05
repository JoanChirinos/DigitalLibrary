use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::auth::{generate_token, TokenStore};
use crate::db::DbPool;
use crate::models::{Library, NewLibrary};
use crate::schema::libraries;

pub fn router() -> Router<(DbPool, TokenStore)> {
    Router::new()
        .route("/libraries", get(list_libraries))
        .route("/create", post(create_library))
        .route("/login", post(login))
}

#[derive(Deserialize)]
pub struct CreateLibraryRequest {
    name: String,
    passkey: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    name: String,
    passkey: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    library_id: i32,
    library_name: String,
}

async fn list_libraries(
    State((pool, _)): State<(DbPool, TokenStore)>,
) -> Result<Json<Vec<Library>>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let libs = libraries::table
        .load::<Library>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(libs))
}

async fn create_library(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    Json(req): Json<CreateLibraryRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let hash = bcrypt::hash(&req.passkey, bcrypt::DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::insert_into(libraries::table)
        .values(&NewLibrary { name: req.name.clone(), passkey_hash: hash })
        .execute(&mut conn)
        .map_err(|_| StatusCode::CONFLICT)?;

    let library = libraries::table
        .order(libraries::id.desc())
        .first::<Library>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = generate_token(&token_store, library.id);

    Ok((StatusCode::CREATED, Json(AuthResponse {
        token,
        library_id: library.id,
        library_name: library.name,
    })))
}

async fn login(
    State((pool, token_store)): State<(DbPool, TokenStore)>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let mut conn = pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let library = libraries::table
        .filter(libraries::name.eq(&req.name))
        .first::<Library>(&mut conn)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let valid = bcrypt::verify(&req.passkey, &library.passkey_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = generate_token(&token_store, library.id);

    Ok(Json(AuthResponse {
        token,
        library_id: library.id,
        library_name: library.name,
    }))
}
