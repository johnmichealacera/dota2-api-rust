use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

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
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

const DEFAULT_OPEN_DOTA_API_URL: &str = "https://api.opendota.com/api";
const HERO_URL_BASE: &str = "https://cdn.cloudflare.steamstatic.com/apps/dota2/images/dota_react/heroes";

#[derive(Clone)]
struct AppState {
    client: Client,
    open_dota_api_url: String,
    cache: Arc<RwLock<HashMap<String, Value>>>,
}

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
        .and_then(|value| value.parse().ok())
        .unwrap_or(8000);

    let state = AppState {
        client: Client::new(),
        open_dota_api_url,
        cache: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/heroes", get(get_heroes))
        .route("/hero/:id", get(get_hero_by_id))
        .route("/hero-matchup/:id", get(get_hero_matchup))
        .route("/pro-players", get(get_pro_players))
        .route("/pro-teams", get(get_pro_teams))
        .route("/team/:id", get(get_team_by_id))
        .route("/team-matchup/:id", get(get_team_matchup))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(build_cors_layer());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("dota-api-rust-backend running on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn build_cors_layer() -> CorsLayer {
    let default_origins = String::from(
        "http://localhost:8080,https://dota2-companion.vercel.app,https://dota2-companion.johnmichealacera.com",
    );
    let raw = env::var("DOTA_SITE").unwrap_or(default_origins);
    let origins: Vec<HeaderValue> = raw
        .split(',')
        .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
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

async fn get_hero_matchup(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<MatchupDto>>, ApiError> {
    let key = format!("dotaHeroMatchup-{id}");
    let cached = try_get_cache::<Vec<MatchupDto>>(&state, &key).await?;
    let data = if let Some(value) = cached {
        value
    } else {
        let url = format!("{}/heroes/{id}/matchups", state.open_dota_api_url);
        let matchups: Vec<HeroMatchupRaw> = state.client.get(url).send().await?.json().await?;
        let heroes = fetch_heroes(&state).await?;

        let mut mapped: Vec<MatchupDto> = matchups
            .into_iter()
            .map(|matchup| {
                let hero = heroes.iter().find(|hero| hero.id == matchup.hero_id);
                let games = matchup.games_played;
                let wins = matchup.wins;
                let win_rate = if games == 0 {
                    0.0
                } else {
                    (wins as f64 / games as f64) * 100.0
                };
                MatchupDto {
                    id: matchup.hero_id,
                    name: hero.map_or(String::new(), |item| item.name.clone()),
                    img: hero.map_or(String::new(), |item| item.img.clone()),
                    wins,
                    games_played: games,
                    win_rate,
                }
            })
            .collect();

        mapped.sort_by(|a, b| b.win_rate.partial_cmp(&a.win_rate).unwrap_or(Ordering::Equal));
        set_cache(&state, &key, &mapped).await?;
        mapped
    };

    Ok(Json(paginate(data, query)))
}

async fn get_pro_players(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ProPlayerDto>>, ApiError> {
    let key = "proPlayers";
    let cached = try_get_cache::<Vec<ProPlayerDto>>(&state, key).await?;
    let players = if let Some(value) = cached {
        value
    } else {
        let url = format!("{}/proPlayers", state.open_dota_api_url);
        let raw: Vec<ProPlayerRaw> = state.client.get(url).send().await?.json().await?;
        let mut mapped: Vec<ProPlayerDto> = raw
            .into_iter()
            .filter(|p| p.account_id.is_some())
            .map(map_pro_player)
            .filter(|p| !p.name.is_empty())
            .collect();
        mapped.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        set_cache(&state, key, &mapped).await?;
        mapped
    };

    Ok(Json(paginate(players, query)))
}

async fn get_pro_teams(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<TeamCardDto>>, ApiError> {
    let key = "proTeams";
    let cached = try_get_cache::<Vec<TeamCardDto>>(&state, key).await?;
    let teams = if let Some(value) = cached {
        value
    } else {
        let url = format!("{}/teams", state.open_dota_api_url);
        let teams_raw: Vec<Value> = state.client.get(url).send().await?.json().await?;
        let mapped: Vec<TeamCardDto> = teams_raw
            .iter()
            .map(map_team_card)
            .collect();
        set_cache(&state, key, &mapped).await?;
        mapped
    };

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
    let team: Value = state.client.get(url).send().await?.json().await?;
    let dto = map_team(&team);
    set_cache(&state, &key, &dto).await?;
    Ok(Json(dto))
}

async fn get_team_matchup(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<TeamMatchupDto>>, ApiError> {
    let key = format!("dotaTeamMatchup-{id}");
    let cached = try_get_cache::<Vec<TeamMatchupDto>>(&state, &key).await?;
    let matchups = if let Some(value) = cached {
        value
    } else {
        let url = format!("{}/teams/{id}/matches", state.open_dota_api_url);
        let matches: Vec<TeamMatchRaw> = state.client.get(url).send().await?.json().await?;
        let grouped = group_team_matchups(matches);
        set_cache(&state, &key, &grouped).await?;
        grouped
    };

    Ok(Json(paginate(matchups, query)))
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

async fn fetch_heroes(state: &AppState) -> Result<Vec<HeroDto>, ApiError> {
    let key = "dotaHeroes";
    if let Some(cached) = try_get_cache::<Vec<HeroDto>>(state, key).await? {
        return Ok(cached);
    }

    let url = format!("{}/constants/heroes", state.open_dota_api_url);
    let payload: HashMap<String, HeroRaw> = state.client.get(url).send().await?.json().await?;
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
    heroes.sort_by_key(|hero| hero.id);

    set_cache(state, key, &heroes).await?;
    Ok(heroes)
}

async fn try_get_cache<T>(state: &AppState, key: &str) -> Result<Option<T>, ApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let map = state.cache.read().await;
    if let Some(value) = map.get(key) {
        let item = serde_json::from_value(value.clone())?;
        return Ok(Some(item));
    }
    Ok(None)
}

async fn set_cache<T>(state: &AppState, key: &str, value: &T) -> Result<(), ApiError>
where
    T: Serialize,
{
    let mut map = state.cache.write().await;
    map.insert(key.to_string(), serde_json::to_value(value)?);
    Ok(())
}

fn paginate<T>(items: Vec<T>, query: PaginationQuery) -> PaginatedResponse<T>
where
    T: Clone,
{
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1);
    let total_items = items.len();
    let total_pages = if total_items == 0 { 1 } else { total_items.div_ceil(page_size) };
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
