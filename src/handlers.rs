use std::cmp::Ordering;
use std::collections::HashMap;
use std::env;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;
use tower_http::cors::{Any, CorsLayer};

use crate::cache::{set_cache, try_get_cache};
use crate::config::*;
use crate::error::ApiError;
use crate::fetch::*;
use crate::models::*;
use crate::state::AppState;

// ── CORS ──────────────────────────────────────────────────────────────────────

pub fn build_cors_layer() -> CorsLayer {
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

pub async fn root() -> &'static str {
    "Dota API Rust backend is running."
}

pub async fn get_heroes(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<HeroDto>>, ApiError> {
    let heroes = fetch_heroes(&state).await?;
    Ok(Json(paginate(heroes, query)))
}

pub async fn get_hero_by_id(
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

pub async fn get_hero_stats(
    State(state): State<AppState>,
    Query(query): Query<HeroStatsQuery>,
) -> Result<Json<HashMap<String, HeroStatDto>>, ApiError> {
    let bracket = normalize_hero_stats_bracket(query.bracket.as_deref());
    let stats = fetch_hero_stats(&state, bracket).await?;
    Ok(Json(stats))
}

pub async fn get_hero_matchup(
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

pub async fn get_hero_benchmarks(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<HeroBenchmarksDto>, ApiError> {
    let benchmarks = fetch_hero_benchmarks(&state, id).await?;
    Ok(Json(benchmarks))
}

pub async fn get_hero_rankings(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<HeroRankingsDto>, ApiError> {
    let rankings = fetch_hero_rankings(&state, id).await?;
    Ok(Json(rankings))
}

pub async fn get_item_timings(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ItemTimingsDto>, ApiError> {
    let timings = fetch_item_timings(&state, id).await?;
    Ok(Json(timings))
}

pub async fn get_lane_roles(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<LaneRolesDto>, ApiError> {
    let roles = fetch_lane_roles(&state, id).await?;
    Ok(Json(roles))
}

pub async fn get_pro_players(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<ProPlayerDto>>, ApiError> {
    let players = fetch_pro_players(&state).await?;
    Ok(Json(paginate(players, query)))
}

pub async fn get_player_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<PlayerProfileDto>, ApiError> {
    let profile = fetch_player_profile(&state, id).await?;
    Ok(Json(profile))
}

pub async fn get_player_recent_matches(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<PlayerRecentMatchDto>>, ApiError> {
    let matches = fetch_player_recent_matches(&state, id).await?;
    Ok(Json(matches))
}

pub async fn get_player_heroes(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<PlayerHeroStatDto>>, ApiError> {
    let heroes = fetch_player_heroes(&state, id).await?;
    Ok(Json(heroes))
}

pub async fn get_player_ratings(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<PlayerRatingDto>>, ApiError> {
    let ratings = fetch_player_ratings(&state, id).await?;
    Ok(Json(ratings))
}

pub async fn get_pro_matches(
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

pub async fn get_match_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<MatchDetailDto>, ApiError> {
    let key = format!("matchDetail-{id}");
    if let Some(cached) = try_get_cache::<MatchDetailDto>(&state, &key).await? {
        return Ok(Json(cached));
    }

    let url = format!("{}/matches/{id}", state.open_dota_api_url);
    let raw: Value = fetch_url(&state.client, &url).await?;
    let heroes = fetch_heroes(&state).await?;
    let items = fetch_items(&state).await?;
    let pro_players = fetch_pro_players(&state).await?;
    let hero_lookup: HashMap<i64, HeroDto> = heroes.into_iter().map(|h| (h.id, h)).collect();
    let pro_names: HashMap<i64, String> = pro_players
        .into_iter()
        .map(|p| (p.account_id, p.name))
        .collect();
    let dto = map_match_detail(&raw, &hero_lookup, &items, &pro_names);
    set_cache(&state, &key, &dto, MATCH_TTL).await?;
    Ok(Json(dto))
}

pub async fn get_pro_teams(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<TeamCardDto>>, ApiError> {
    let teams = fetch_pro_teams(&state).await?;
    Ok(Json(paginate(teams, query)))
}

pub async fn get_team_by_id(
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

pub async fn get_team_matchup(
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

pub async fn get_team_players(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<TeamPlayerDto>>, ApiError> {
    let players = fetch_team_players(&state, id).await?;
    Ok(Json(players))
}

pub async fn get_team_heroes(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<TeamHeroDto>>, ApiError> {
    let heroes = fetch_team_heroes(&state, id).await?;
    Ok(Json(heroes))
}

pub async fn get_team_matches(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<ProMatchDto>>, ApiError> {
    let matches = fetch_team_matches(&state, id).await?;
    Ok(Json(matches))
}

pub async fn get_search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchPlayerDto>>, ApiError> {
    let results = fetch_search(&state, &query.q).await?;
    Ok(Json(results))
}

pub async fn get_leagues(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<LeagueDto>>, ApiError> {
    let leagues = fetch_leagues(&state).await?;
    Ok(Json(paginate(leagues, query)))
}

pub async fn get_league_teams(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<TeamCardDto>>, ApiError> {
    let teams = fetch_league_teams(&state, id).await?;
    Ok(Json(teams))
}

pub async fn get_league_matches(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<ProMatchDto>>, ApiError> {
    let matches = fetch_league_matches(&state, id).await?;
    Ok(Json(matches))
}

pub async fn get_record_by_field(
    State(state): State<AppState>,
    Path(field): Path<String>,
) -> Result<Response, ApiError> {
    let normalized = field.trim().to_lowercase();
    if !RECORD_FIELDS.contains(&normalized.as_str()) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid record field" })),
        )
            .into_response());
    }

    match fetch_record(&state, &normalized).await? {
        Some(record) => Ok(Json(record).into_response()),
        None => Ok((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "No record found" })),
        )
            .into_response()),
    }
}

pub async fn get_live_games(
    State(state): State<AppState>,
) -> Result<Json<Vec<LiveGameDto>>, ApiError> {
    let games = fetch_live_games(&state).await?;
    Ok(Json(games))
}


// ── Pagination ────────────────────────────────────────────────────────────────

pub fn paginate<T: Clone>(items: Vec<T>, query: PaginationQuery) -> PaginatedResponse<T> {
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


