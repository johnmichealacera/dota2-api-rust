use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use dotenvy::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

const DEFAULT_OPEN_DOTA_API_URL: &str = "https://api.opendota.com/api";
const HERO_URL_BASE: &str =
    "https://cdn.cloudflare.steamstatic.com/apps/dota2/images/dota_react/heroes";

// Cache TTLs — lists refresh every 6 h; per-entity matchup data every 24 h
const LIST_TTL: Duration = Duration::from_secs(6 * 3600);
const MATCHUP_TTL: Duration = Duration::from_secs(24 * 3600);
const FEED_TTL: Duration = Duration::from_secs(5 * 60); // pro matches — high velocity content

// ── Cache ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

// ── App state ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    client: Client,
    open_dota_api_url: String,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
enum ApiError {
    Upstream(reqwest::Error),
    Parse(serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::Upstream(err) => {
                error!("upstream request failed: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Error occurred" })),
                )
                    .into_response()
            }
            Self::Parse(err) => {
                error!("failed to parse upstream payload: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Error occurred" })),
                )
                    .into_response()
            }
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Upstream(value)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value)
    }
}

// ── Pagination ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<usize>,
    #[serde(rename = "pageSize")]
    page_size: Option<usize>,
}

#[derive(Debug, Serialize)]
struct PaginationMeta {
    #[serde(rename = "totalItems")]
    total_items: usize,
    #[serde(rename = "currentPage")]
    current_page: usize,
    #[serde(rename = "pageSize")]
    page_size: usize,
    #[serde(rename = "totalPages")]
    total_pages: usize,
}

#[derive(Debug, Serialize)]
struct PaginatedResponse<T> {
    items: Vec<T>,
    pagination: PaginationMeta,
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct HeroRaw {
    id: i64,
    localized_name: String,
    primary_attr: String,
    attack_type: String,
    roles: Vec<String>,
    img: String,
    base_health: i64,
    base_str: i64,
    base_agi: i64,
    base_int: i64,
    base_mana: i64,
    base_armor: f64,
    base_mr: i64,
    attack_range: i64,
    attack_rate: f64,
    move_speed: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HeroDto {
    id: i64,
    name: String,
    #[serde(rename = "primaryAttr")]
    primary_attr: String,
    #[serde(rename = "attackType")]
    attack_type: String,
    roles: Vec<String>,
    img: String,
    icon: String,
    health: i64,
    #[serde(rename = "baseStr")]
    base_str: i64,
    #[serde(rename = "baseAgi")]
    base_agi: i64,
    #[serde(rename = "baseInt")]
    base_int: i64,
    #[serde(rename = "baseMana")]
    base_mana: i64,
    #[serde(rename = "baseArmor")]
    base_armor: f64,
    #[serde(rename = "baseMr")]
    base_mr: i64,
    #[serde(rename = "attackRange")]
    attack_range: i64,
    #[serde(rename = "attackRate")]
    attack_rate: f64,
    #[serde(rename = "moveSpeed")]
    move_speed: i64,
    #[serde(rename = "hoverFirst")]
    hover_first: i64,
    #[serde(rename = "hoverSecond")]
    hover_second: i64,
    #[serde(rename = "hoverThird")]
    hover_third: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TeamDto {
    id: i64,
    name: String,
    rating: f64,
    wins: i64,
    losses: i64,
    #[serde(rename = "lastMatchTime")]
    last_match_time: String,
    tag: String,
    img: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TeamCardDto {
    id: i64,
    name: String,
    rating: f64,
    wins: i64,
    losses: i64,
    #[serde(rename = "last_match_time")]
    last_match_time: Option<String>,
    tag: String,
    img: String,
    #[serde(rename = "hoverFirst")]
    hover_first: f64,
    #[serde(rename = "hoverSecond")]
    hover_second: i64,
    #[serde(rename = "hoverThird")]
    hover_third: i64,
}

#[derive(Debug, Deserialize)]
struct HeroMatchupRaw {
    hero_id: i64,
    games_played: i64,
    wins: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MatchupDto {
    id: i64,
    name: String,
    img: String,
    wins: i64,
    #[serde(rename = "gamesPlayed")]
    games_played: i64,
    #[serde(rename = "winRate")]
    win_rate: f64,
}

#[derive(Debug, Deserialize)]
struct ProPlayerRaw {
    account_id: Option<i64>,
    name: Option<String>,
    personaname: Option<String>,
    avatarfull: Option<String>,
    team_name: Option<String>,
    team_tag: Option<String>,
    country_code: Option<String>,
    fantasy_role: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProPlayerDto {
    #[serde(rename = "accountId")]
    account_id: i64,
    name: String,
    #[serde(rename = "teamName")]
    team_name: String,
    #[serde(rename = "teamTag")]
    team_tag: String,
    #[serde(rename = "countryCode")]
    country_code: String,
    #[serde(rename = "fantasyRole")]
    fantasy_role: i64,
    avatar: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HeroStatDto {
    #[serde(rename = "heroId")]
    hero_id: i64,
    #[serde(rename = "winRate")]
    win_rate: f64,
    #[serde(rename = "totalPicks")]
    total_picks: i64,
    #[serde(rename = "totalBans")]
    total_bans: i64,
}

#[derive(Debug, Deserialize)]
struct ProMatchRaw {
    match_id: Option<i64>,
    duration: Option<i64>,
    start_time: Option<i64>,
    radiant_team_id: Option<i64>,
    radiant_name: Option<String>,
    dire_team_id: Option<i64>,
    dire_name: Option<String>,
    league_id: Option<i64>,
    league_name: Option<String>,
    radiant_score: Option<i64>,
    dire_score: Option<i64>,
    radiant_win: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ProMatchDto {
    #[serde(rename = "matchId")]
    match_id: i64,
    duration: i64,
    #[serde(rename = "startTime")]
    start_time: i64,
    #[serde(rename = "radiantTeamId")]
    radiant_team_id: i64,
    #[serde(rename = "radiantName")]
    radiant_name: String,
    #[serde(rename = "direTeamId")]
    dire_team_id: i64,
    #[serde(rename = "direName")]
    dire_name: String,
    #[serde(rename = "leagueId")]
    league_id: i64,
    #[serde(rename = "leagueName")]
    league_name: String,
    #[serde(rename = "radiantScore")]
    radiant_score: i64,
    #[serde(rename = "direScore")]
    dire_score: i64,
    #[serde(rename = "radiantWin")]
    radiant_win: bool,
}

#[derive(Debug, Deserialize)]
struct TeamMatchRaw {
    opposing_team_id: Option<i64>,
    opposing_team_name: Option<String>,
    opposing_team_logo: Option<String>,
    league_name: Option<String>,
    radiant: Option<bool>,
    radiant_win: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TeamMatchupDto {
    id: i64,
    wins: i64,
    #[serde(rename = "gamesPlayed")]
    games_played: i64,
    #[serde(rename = "winRate")]
    win_rate: f64,
    name: String,
    img: String,
    #[serde(rename = "leagueName")]
    league_name: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

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
            warm!("hero-stats", fetch_hero_stats(&s));
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
        .route("/pro-players", get(get_pro_players))
        .route("/pro-matches", get(get_pro_matches))
        .route("/pro-teams", get(get_pro_teams))
        .route("/team/:id", get(get_team_by_id))
        .route("/team-matchup/:id", get(get_team_matchup))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(build_cors_layer());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("dota-api-rust-backend running on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ── CORS ──────────────────────────────────────────────────────────────────────

fn build_cors_layer() -> CorsLayer {
    let default_origins = String::from(
        "http://localhost:8080,https://dota2-companion.vercel.app,https://dota2-companion.johnmichealacera.com",
    );
    let raw = env::var("DOTA_SITE").unwrap_or(default_origins);
    let origins: Vec<HeaderValue> = raw
        .split(',')
        .filter_map(|o| HeaderValue::from_str(o.trim()).ok())
        .collect();

    let base = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_headers(Any);

    if origins.is_empty() {
        base.allow_origin(Any)
    } else {
        base.allow_origin(origins)
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn root() -> &'static str {
    "Dota API Rust backend is running."
}

async fn get_heroes(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<HeroDto>>, ApiError> {
    let heroes = fetch_heroes(&state).await?;
    Ok(Json(paginate(heroes, query)))
}

async fn get_hero_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<HeroDto>, ApiError> {
    let heroes = fetch_heroes(&state).await?;
    let hero = heroes
        .into_iter()
        .find(|item| item.id == id)
        .unwrap_or_else(empty_hero);
    Ok(Json(hero))
}

async fn get_hero_stats(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, HeroStatDto>>, ApiError> {
    let stats = fetch_hero_stats(&state).await?;
    Ok(Json(stats))
}

async fn get_hero_matchup(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<MatchupDto>>, ApiError> {
    let key = format!("dotaHeroMatchup-{id}");
    let data = if let Some(cached) = try_get_cache::<Vec<MatchupDto>>(&state, &key).await? {
        cached
    } else {
        let url = format!("{}/heroes/{id}/matchups", state.open_dota_api_url);
        let matchups: Vec<HeroMatchupRaw> = fetch_url(&state.client, &url).await?;
        let heroes = fetch_heroes(&state).await?;

        let mut mapped: Vec<MatchupDto> = matchups
            .into_iter()
            .map(|m| {
                let hero = heroes.iter().find(|h| h.id == m.hero_id);
                let wr = if m.games_played == 0 {
                    0.0
                } else {
                    (m.wins as f64 / m.games_played as f64) * 100.0
                };
                MatchupDto {
                    id: m.hero_id,
                    name: hero.map_or(String::new(), |h| h.name.clone()),
                    img: hero.map_or(String::new(), |h| h.img.clone()),
                    wins: m.wins,
                    games_played: m.games_played,
                    win_rate: wr,
                }
            })
            .collect();

        mapped.sort_by(|a, b| b.win_rate.partial_cmp(&a.win_rate).unwrap_or(Ordering::Equal));
        set_cache(&state, &key, &mapped, MATCHUP_TTL).await?;
        mapped
    };

    Ok(Json(paginate(data, query)))
}

async fn get_pro_players(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ProPlayerDto>>, ApiError> {
    let players = fetch_pro_players(&state).await?;
    Ok(Json(paginate(players, query)))
}

async fn get_pro_matches(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ProMatchDto>>, ApiError> {
    let key = "proMatches";
    let matches = if let Some(cached) = try_get_cache::<Vec<ProMatchDto>>(&state, key).await? {
        cached
    } else {
        let url = format!("{}/proMatches", state.open_dota_api_url);
        let raw: Vec<ProMatchRaw> = fetch_url(&state.client, &url).await?;
        let mapped: Vec<ProMatchDto> = raw.into_iter().filter_map(map_pro_match).collect();
        set_cache(&state, key, &mapped, FEED_TTL).await?;
        mapped
    };
    Ok(Json(paginate(matches, query)))
}

async fn get_pro_teams(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<TeamCardDto>>, ApiError> {
    let teams = fetch_pro_teams(&state).await?;
    Ok(Json(paginate(teams, query)))
}

async fn get_team_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<TeamDto>, ApiError> {
    let key = format!("dotaInfoTeam-{id}");
    if let Some(cached) = try_get_cache::<TeamDto>(&state, &key).await? {
        return Ok(Json(cached));
    }
    let url = format!("{}/teams/{id}", state.open_dota_api_url);
    let team: Value = fetch_url(&state.client, &url).await?;
    let dto = map_team(&team);
    set_cache(&state, &key, &dto, MATCHUP_TTL).await?;
    Ok(Json(dto))
}

async fn get_team_matchup(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<TeamMatchupDto>>, ApiError> {
    let key = format!("dotaTeamMatchup-{id}");
    let matchups = if let Some(cached) = try_get_cache::<Vec<TeamMatchupDto>>(&state, &key).await? {
        cached
    } else {
        let url = format!("{}/teams/{id}/matches", state.open_dota_api_url);
        let matches: Vec<TeamMatchRaw> = fetch_url(&state.client, &url).await?;
        let grouped = group_team_matchups(matches);
        set_cache(&state, &key, &grouped, MATCHUP_TTL).await?;
        grouped
    };

    Ok(Json(paginate(matchups, query)))
}

// ── Internal fetch functions (shared by handlers + warmup) ────────────────────

async fn fetch_heroes(state: &AppState) -> Result<Vec<HeroDto>, ApiError> {
    let key = "dotaHeroes";
    if let Some(cached) = try_get_cache::<Vec<HeroDto>>(state, key).await? {
        return Ok(cached);
    }

    let url = format!("{}/constants/heroes", state.open_dota_api_url);
    let payload: HashMap<String, HeroRaw> = fetch_url(&state.client, &url).await?;
    let mut heroes: Vec<HeroDto> = payload
        .into_values()
        .map(|hero| {
            let image_file = hero.img.rsplit('/').next().unwrap_or_default();
            HeroDto {
                id: hero.id,
                name: hero.localized_name,
                primary_attr: hero.primary_attr,
                attack_type: hero.attack_type,
                roles: hero.roles,
                img: format!("{HERO_URL_BASE}/{image_file}"),
                icon: format!("{HERO_URL_BASE}/{image_file}"),
                health: hero.base_health,
                base_str: hero.base_str,
                base_agi: hero.base_agi,
                base_int: hero.base_int,
                base_mana: hero.base_mana,
                base_armor: hero.base_armor,
                base_mr: hero.base_mr,
                attack_range: hero.attack_range,
                attack_rate: hero.attack_rate,
                move_speed: hero.move_speed,
                hover_first: hero.base_str,
                hover_second: hero.base_agi,
                hover_third: hero.base_int,
            }
        })
        .collect();
    heroes.sort_by_key(|h| h.id);
    set_cache(state, key, &heroes, LIST_TTL).await?;
    Ok(heroes)
}

async fn fetch_hero_stats(state: &AppState) -> Result<HashMap<String, HeroStatDto>, ApiError> {
    let key = "heroStats";
    if let Some(cached) = try_get_cache::<HashMap<String, HeroStatDto>>(state, key).await? {
        return Ok(cached);
    }

    let url = format!("{}/heroStats", state.open_dota_api_url);
    let raw: Vec<Value> = fetch_url(&state.client, &url).await?;

    let mut map: HashMap<String, HeroStatDto> = HashMap::new();
    for v in &raw {
        let Some(id) = v.get("id").and_then(|x| x.as_i64()) else { continue };

        // Use pro-circuit stats exclusively so pick/win/ban are from the same population
        let total_picks = v.get("pro_pick").and_then(|x| x.as_i64()).unwrap_or(0);
        let total_wins  = v.get("pro_win" ).and_then(|x| x.as_i64()).unwrap_or(0);
        let total_bans  = v.get("pro_ban" ).and_then(|x| x.as_i64()).unwrap_or(0);

        let win_rate = if total_picks == 0 { 50.0 } else {
            (total_wins as f64 / total_picks as f64) * 100.0
        };

        map.insert(id.to_string(), HeroStatDto { hero_id: id, win_rate, total_picks, total_bans });
    }

    set_cache(state, key, &map, LIST_TTL).await?;
    Ok(map)
}

async fn fetch_pro_players(state: &AppState) -> Result<Vec<ProPlayerDto>, ApiError> {
    let key = "proPlayers";
    if let Some(cached) = try_get_cache::<Vec<ProPlayerDto>>(state, key).await? {
        return Ok(cached);
    }
    let url = format!("{}/proPlayers", state.open_dota_api_url);
    let raw: Vec<ProPlayerRaw> = fetch_url(&state.client, &url).await?;
    let mut mapped: Vec<ProPlayerDto> = raw
        .into_iter()
        .filter(|p| p.account_id.is_some())
        .map(map_pro_player)
        .filter(|p| !p.name.is_empty())
        .collect();
    mapped.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    set_cache(state, key, &mapped, LIST_TTL).await?;
    Ok(mapped)
}

async fn fetch_pro_teams(state: &AppState) -> Result<Vec<TeamCardDto>, ApiError> {
    let key = "proTeams";
    if let Some(cached) = try_get_cache::<Vec<TeamCardDto>>(state, key).await? {
        return Ok(cached);
    }
    let url = format!("{}/teams", state.open_dota_api_url);
    let raw: Vec<Value> = fetch_url(&state.client, &url).await?;
    let mapped: Vec<TeamCardDto> = raw.iter().map(map_team_card).collect();
    set_cache(state, key, &mapped, LIST_TTL).await?;
    Ok(mapped)
}

// ── Upstream fetch with retry ─────────────────────────────────────────────────

async fn fetch_url<T>(client: &Client, url: &str) -> Result<T, ApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let result = client.get(url).send().await;
    let response = match result {
        Ok(r) => r,
        Err(e) if e.is_timeout() || e.is_connect() => {
            info!("transient error on {url}, retrying in 3s…");
            tokio::time::sleep(Duration::from_secs(3)).await;
            client.get(url).send().await?
        }
        Err(e) => return Err(ApiError::Upstream(e)),
    };
    Ok(response.json::<T>().await?)
}

// ── Cache helpers ─────────────────────────────────────────────────────────────

async fn try_get_cache<T>(state: &AppState, key: &str) -> Result<Option<T>, ApiError>
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

async fn set_cache<T>(state: &AppState, key: &str, value: &T, ttl: Duration) -> Result<(), ApiError>
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

// ── Pagination ────────────────────────────────────────────────────────────────

fn paginate<T: Clone>(items: Vec<T>, query: PaginationQuery) -> PaginatedResponse<T> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1);
    let total_items = items.len();
    let total_pages = if total_items == 0 {
        1
    } else {
        total_items.div_ceil(page_size)
    };
    let start_index = (page - 1) * page_size;
    let paged_items = items.into_iter().skip(start_index).take(page_size).collect();
    PaginatedResponse {
        items: paged_items,
        pagination: PaginationMeta {
            total_items,
            current_page: page,
            page_size,
            total_pages,
        },
    }
}

// ── Mapping helpers ───────────────────────────────────────────────────────────

fn map_pro_match(raw: ProMatchRaw) -> Option<ProMatchDto> {
    let match_id = raw.match_id?;
    Some(ProMatchDto {
        match_id,
        duration: raw.duration.unwrap_or(0),
        start_time: raw.start_time.unwrap_or(0),
        radiant_team_id: raw.radiant_team_id.unwrap_or(0),
        radiant_name: raw.radiant_name.unwrap_or_else(|| "Radiant".to_string()),
        dire_team_id: raw.dire_team_id.unwrap_or(0),
        dire_name: raw.dire_name.unwrap_or_else(|| "Dire".to_string()),
        league_id: raw.league_id.unwrap_or(0),
        league_name: raw.league_name.unwrap_or_default(),
        radiant_score: raw.radiant_score.unwrap_or(0),
        dire_score: raw.dire_score.unwrap_or(0),
        radiant_win: raw.radiant_win.unwrap_or(false),
    })
}

fn map_pro_player(raw: ProPlayerRaw) -> ProPlayerDto {
    let display_name = raw
        .name
        .filter(|s| !s.is_empty())
        .or(raw.personaname)
        .unwrap_or_default();
    ProPlayerDto {
        account_id: raw.account_id.unwrap_or_default(),
        name: display_name,
        team_name: raw.team_name.unwrap_or_default(),
        team_tag: raw.team_tag.unwrap_or_default(),
        country_code: raw.country_code.unwrap_or_default(),
        fantasy_role: raw.fantasy_role.unwrap_or(-1),
        avatar: raw.avatarfull.unwrap_or_default(),
    }
}

fn group_team_matchups(matches: Vec<TeamMatchRaw>) -> Vec<TeamMatchupDto> {
    let mut grouped: HashMap<i64, TeamMatchupDto> = HashMap::new();

    for item in matches {
        let id = item.opposing_team_id.unwrap_or_default();
        let entry = grouped.entry(id).or_insert(TeamMatchupDto {
            id,
            wins: 0,
            games_played: 0,
            win_rate: 0.0,
            name: item.opposing_team_name.clone().unwrap_or_default(),
            img: item.opposing_team_logo.clone().unwrap_or_default(),
            league_name: item.league_name.clone().unwrap_or_default(),
        });

        if item.radiant == item.radiant_win {
            entry.wins += 1;
        }
        entry.games_played += 1;
        if entry.games_played > 0 {
            entry.win_rate = (entry.wins as f64 / entry.games_played as f64) * 100.0;
        }
        if entry.name.is_empty() {
            entry.name = item.opposing_team_name.unwrap_or_default();
        }
        if entry.img.is_empty() {
            entry.img = item.opposing_team_logo.unwrap_or_default();
        }
        if entry.league_name.is_empty() {
            entry.league_name = item.league_name.unwrap_or_default();
        }
    }

    grouped.into_values().collect()
}

fn map_team_card(item: &Value) -> TeamCardDto {
    let rating = value_to_f64(item.get("rating"));
    let wins = value_to_i64(item.get("wins"));
    let losses = value_to_i64(item.get("losses"));
    TeamCardDto {
        id: value_to_i64(item.get("team_id")),
        name: value_to_string(item.get("name")),
        rating,
        wins,
        losses,
        last_match_time: value_to_optional_string(item.get("last_match_time")),
        tag: value_to_string(item.get("tag")),
        img: value_to_string(item.get("logo_url")),
        hover_first: rating,
        hover_second: wins,
        hover_third: losses,
    }
}

fn map_team(item: &Value) -> TeamDto {
    TeamDto {
        id: value_to_i64(item.get("team_id")),
        name: value_to_string(item.get("name")),
        rating: value_to_f64(item.get("rating")),
        wins: value_to_i64(item.get("wins")),
        losses: value_to_i64(item.get("losses")),
        last_match_time: value_to_optional_string(item.get("last_match_time")).unwrap_or_default(),
        tag: value_to_string(item.get("tag")),
        img: value_to_string(item.get("logo_url")),
    }
}

fn empty_hero() -> HeroDto {
    HeroDto {
        id: 0,
        name: String::new(),
        primary_attr: String::new(),
        attack_type: String::new(),
        roles: vec![],
        img: String::new(),
        icon: String::new(),
        health: 0,
        base_str: 0,
        base_agi: 0,
        base_int: 0,
        base_mana: 0,
        base_armor: 0.0,
        base_mr: 0,
        attack_range: 0,
        attack_rate: 0.0,
        move_speed: 0,
        hover_first: 0,
        hover_second: 0,
        hover_third: 0,
    }
}

fn value_to_i64(value: Option<&Value>) -> i64 {
    match value {
        Some(Value::Number(n)) => n.as_i64().unwrap_or_default(),
        Some(Value::String(s)) => s.parse::<i64>().unwrap_or_default(),
        _ => 0,
    }
}

fn value_to_f64(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Number(n)) => n.as_f64().unwrap_or_default(),
        Some(Value::String(s)) => s.parse::<f64>().unwrap_or_default(),
        _ => 0.0,
    }
}

fn value_to_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn value_to_optional_string(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::Null) | None => None,
        Some(v) => Some(value_to_string(Some(v))),
    }
}
