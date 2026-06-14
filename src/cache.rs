use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::{AppState, CacheEntry};

// ── Cache helpers ─────────────────────────────────────────────────────────────

pub async fn try_get_cache<T>(state: &AppState, key: &str) -> Result<Option<T>, ApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let map = state.cache.read().await;
    if let Some(entry) = map.get(key) {
        if Instant::now() < entry.expires_at {
            let item = serde_json::from_value(entry.value.clone())?;
            return Ok(Some(item));
        }
    }
    Ok(None)
}

pub async fn set_cache<T>(state: &AppState, key: &str, value: &T, ttl: Duration) -> Result<(), ApiError>
where
    T: Serialize,
{
    let mut map = state.cache.write().await;
    map.insert(
        key.to_string(),
        CacheEntry {
            value: serde_json::to_value(value)?,
            expires_at: Instant::now() + ttl,
        },
    );
    Ok(())
}

