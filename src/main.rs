mod cache;
mod config;
mod error;
mod fetch;
mod handlers;
mod models;
mod state;

use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::routing::get;
use axum::Router;
use dotenvy::dotenv;
use reqwest::Client;
use tokio::sync::RwLock;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::DEFAULT_OPEN_DOTA_API_URL;
use crate::fetch::{fetch_hero_stats, fetch_heroes, fetch_pro_players, fetch_pro_teams};
use crate::handlers::*;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let open_dota_api_url =
        env::var("OPEN_DOTA_API_URL").unwrap_or_else(|_| DEFAULT_OPEN_DOTA_API_URL.to_string());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);

    let state = AppState {
        client: Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("dota-mate/1.0")
            .pool_max_idle_per_host(4) // keep connections modest to avoid overwhelming OpenDota
            .build()
            .expect("failed to build HTTP client"),
        open_dota_api_url,
        cache: Arc::new(RwLock::new(HashMap::new())),
    };

    // Pre-warm the cache sequentially with gaps to stay within OpenDota's rate limit.
    // Parallel warmup saturates the free-tier concurrency and causes user requests to time out.
    {
        let s = state.clone();
        tokio::spawn(async move {
            macro_rules! warm {
                ($label:expr, $fut:expr) => {
                    match $fut.await {
                        Ok(_)  => info!("warm-up: {} ✓", $label),
                        Err(e) => info!("warm-up: {} failed — {:?}", $label, e),
                    }
                    tokio::time::sleep(Duration::from_millis(400)).await;
                };
            }
            warm!("heroes",     fetch_heroes(&s));
            warm!("hero-stats", fetch_hero_stats(&s, "pro"));
            warm!("hero-stats-all", fetch_hero_stats(&s, "all"));
            warm!("teams",      fetch_pro_teams(&s));
            warm!("players",    fetch_pro_players(&s));
            info!("startup warm-up complete");
        });
    }

    // On Render free tier: self-ping every 14 min to prevent the 15-min idle spin-down
    if let Ok(hostname) = env::var("RENDER_EXTERNAL_HOSTNAME") {
        let ping_url = format!("https://{hostname}/");
        let ping_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(14 * 60));
            interval.tick().await; // skip the immediate first tick
            loop {
                interval.tick().await;
                match ping_client.get(&ping_url).send().await {
                    Ok(_)  => info!("keep-alive ping OK → {ping_url}"),
                    Err(e) => info!("keep-alive ping failed: {e}"),
                }
            }
        });
        info!("keep-alive task started → https://{hostname}/");
    }

    let app = Router::new()
        .route("/", get(root))
        .route("/heroes", get(get_heroes))
        .route("/hero-stats", get(get_hero_stats))
        .route("/hero/:id", get(get_hero_by_id))
        .route("/hero-matchup/:id", get(get_hero_matchup))
        .route("/hero-benchmarks/:id", get(get_hero_benchmarks))
        .route("/hero-rankings/:id", get(get_hero_rankings))
        .route("/item-timings/:id", get(get_item_timings))
        .route("/lane-roles/:id", get(get_lane_roles))
        .route("/pro-players", get(get_pro_players))
        .route("/player/:id", get(get_player_by_id))
        .route("/player-recent-matches/:id", get(get_player_recent_matches))
        .route("/player-heroes/:id", get(get_player_heroes))
        .route("/player-ratings/:id", get(get_player_ratings))
        .route("/pro-matches", get(get_pro_matches))
        .route("/match/:id", get(get_match_by_id))
        .route("/pro-teams", get(get_pro_teams))
        .route("/team/:id", get(get_team_by_id))
        .route("/team-matchup/:id", get(get_team_matchup))
        .route("/team-players/:id", get(get_team_players))
        .route("/team-heroes/:id", get(get_team_heroes))
        .route("/team-matches/:id", get(get_team_matches))
        .route("/search", get(get_search))
        .route("/global-search", get(get_global_search))
        .route("/leagues", get(get_leagues))
        .route("/league-teams/:id", get(get_league_teams))
        .route("/league-matches/:id", get(get_league_matches))
        .route("/records/:field", get(get_record_by_field))
        .route("/live", get(get_live_games))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(build_cors_layer());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("dota-api-rust-backend running on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

