use axum::{
    http::StatusCode,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub type TokenStore = Arc<RwLock<HashMap<String, i32>>>;

pub fn create_token_store() -> TokenStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn generate_token(store: &TokenStore, library_id: i32) -> String {
    let token = Uuid::new_v4().to_string();
    store.write().unwrap().insert(token.clone(), library_id);
    token
}

pub fn validate_token(store: &TokenStore, token: &str) -> Option<i32> {
    store.read().unwrap().get(token).copied()
}

pub fn extract_library_id(
    store: &TokenStore,
    auth: TypedHeader<Authorization<Bearer>>,
) -> Result<i32, StatusCode> {
    validate_token(store, auth.token()).ok_or(StatusCode::UNAUTHORIZED)
}
