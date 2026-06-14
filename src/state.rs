use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use reqwest::Client;
use serde_json::Value;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheEntry {
    pub value: Value,
    pub expires_at: Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub open_dota_api_url: String,
    pub cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}
