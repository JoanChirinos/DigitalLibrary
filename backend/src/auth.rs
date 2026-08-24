use axum::{
    http::StatusCode,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// How long an issued token stays valid. Long-lived so users aren't forced to
/// re-authenticate often; tokens are also cleared whenever the server restarts.
const TOKEN_TTL: Duration = Duration::from_secs(90 * 24 * 60 * 60);

pub struct TokenEntry {
    library_id: i32,
    expires_at: Instant,
}

pub type TokenStore = Arc<RwLock<HashMap<String, TokenEntry>>>;

pub fn create_token_store() -> TokenStore {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn generate_token(store: &TokenStore, library_id: i32) -> String {
    let token = Uuid::new_v4().to_string();
    store.write().unwrap().insert(
        token.clone(),
        TokenEntry { library_id, expires_at: Instant::now() + TOKEN_TTL },
    );
    token
}

pub fn validate_token(store: &TokenStore, token: &str) -> Option<i32> {
    {
        let store = store.read().unwrap();
        match store.get(token) {
            Some(entry) if entry.expires_at > Instant::now() => return Some(entry.library_id),
            Some(_) => {} // expired: fall through and remove it
            None => return None,
        }
    }
    store.write().unwrap().remove(token);
    None
}

/// Invalidate a token server-side (used by logout).
pub fn revoke_token(store: &TokenStore, token: &str) {
    store.write().unwrap().remove(token);
}

pub fn extract_library_id(
    store: &TokenStore,
    auth: TypedHeader<Authorization<Bearer>>,
) -> Result<i32, StatusCode> {
    validate_token(store, auth.token()).ok_or(StatusCode::UNAUTHORIZED)
}
