use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Pagination ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<usize>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Deserialize)]
pub struct HeroStatsQuery {
    pub bracket: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    #[serde(rename = "totalItems")]
    pub total_items: usize,
    #[serde(rename = "currentPage")]
    pub current_page: usize,
    #[serde(rename = "pageSize")]
    pub page_size: usize,
    #[serde(rename = "totalPages")]
    pub total_pages: usize,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct HeroRaw {
    pub id: i64,
    pub localized_name: String,
    pub primary_attr: String,
    pub attack_type: String,
    pub roles: Vec<String>,
    pub img: String,
    pub base_health: i64,
    pub base_str: i64,
    pub base_agi: i64,
    pub base_int: i64,
    pub base_mana: i64,
    pub base_armor: f64,
    pub base_mr: i64,
    pub attack_range: i64,
    pub attack_rate: f64,
    pub move_speed: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeroDto {
    pub id: i64,
    pub name: String,
    #[serde(rename = "primaryAttr")]
    pub primary_attr: String,
    #[serde(rename = "attackType")]
    pub attack_type: String,
    pub roles: Vec<String>,
    pub img: String,
    pub icon: String,
    pub health: i64,
    #[serde(rename = "baseStr")]
    pub base_str: i64,
    #[serde(rename = "baseAgi")]
    pub base_agi: i64,
    #[serde(rename = "baseInt")]
    pub base_int: i64,
    #[serde(rename = "baseMana")]
    pub base_mana: i64,
    #[serde(rename = "baseArmor")]
    pub base_armor: f64,
    #[serde(rename = "baseMr")]
    pub base_mr: i64,
    #[serde(rename = "attackRange")]
    pub attack_range: i64,
    #[serde(rename = "attackRate")]
    pub attack_rate: f64,
    #[serde(rename = "moveSpeed")]
    pub move_speed: i64,
    #[serde(rename = "hoverFirst")]
    pub hover_first: i64,
    #[serde(rename = "hoverSecond")]
    pub hover_second: i64,
    #[serde(rename = "hoverThird")]
    pub hover_third: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamDto {
    pub id: i64,
    pub name: String,
    pub rating: f64,
    pub wins: i64,
    pub losses: i64,
    #[serde(rename = "lastMatchTime")]
    pub last_match_time: String,
    pub tag: String,
    pub img: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamCardDto {
    pub id: i64,
    pub name: String,
    pub rating: f64,
    pub wins: i64,
    pub losses: i64,
    #[serde(rename = "last_match_time")]
    pub last_match_time: Option<String>,
    pub tag: String,
    pub img: String,
    #[serde(rename = "hoverFirst")]
    pub hover_first: f64,
    #[serde(rename = "hoverSecond")]
    pub hover_second: i64,
    #[serde(rename = "hoverThird")]
    pub hover_third: i64,
}

#[derive(Debug, Deserialize)]
pub struct HeroMatchupRaw {
    pub hero_id: i64,
    pub games_played: i64,
    pub wins: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchupDto {
    pub id: i64,
    pub name: String,
    pub img: String,
    pub wins: i64,
    #[serde(rename = "gamesPlayed")]
    pub games_played: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct ProPlayerRaw {
    pub account_id: Option<i64>,
    pub name: Option<String>,
    pub personaname: Option<String>,
    pub avatarfull: Option<String>,
    pub team_name: Option<String>,
    pub team_tag: Option<String>,
    pub country_code: Option<String>,
    pub fantasy_role: Option<i64>,
    pub team_id: Option<i64>,
    pub is_pro: Option<bool>,
    pub last_match_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProPlayerDto {
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub name: String,
    #[serde(rename = "teamName")]
    pub team_name: String,
    #[serde(rename = "teamTag")]
    pub team_tag: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "fantasyRole")]
    pub fantasy_role: i64,
    pub avatar: String,
    #[serde(rename = "teamId")]
    pub team_id: i64,
    #[serde(rename = "isPro")]
    pub is_pro: bool,
    #[serde(rename = "lastMatchTime")]
    pub last_match_time: String,
    #[serde(rename = "rankTier")]
    pub rank_tier: Option<i64>,
    pub mmr: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeroStatDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
    #[serde(rename = "totalPicks")]
    pub total_picks: i64,
    #[serde(rename = "totalBans")]
    pub total_bans: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BenchmarkPointDto {
    pub percentile: f64,
    pub value: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BenchmarkMetricDto {
    pub key: String,
    pub label: String,
    pub points: Vec<BenchmarkPointDto>,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub max: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeroBenchmarksDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    pub metrics: Vec<BenchmarkMetricDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemTimingDto {
    pub item: String,
    #[serde(rename = "itemName")]
    pub item_name: String,
    #[serde(rename = "itemImg")]
    pub item_img: String,
    #[serde(rename = "timeSecs")]
    pub time_secs: i64,
    #[serde(rename = "timeLabel")]
    pub time_label: String,
    pub games: i64,
    pub wins: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemTimingsDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    pub timings: Vec<ItemTimingDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LaneRoleTimingDto {
    #[serde(rename = "laneRole")]
    pub lane_role: i64,
    #[serde(rename = "laneLabel")]
    pub lane_label: String,
    #[serde(rename = "timeSecs")]
    pub time_secs: i64,
    #[serde(rename = "timeLabel")]
    pub time_label: String,
    pub games: i64,
    pub wins: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LaneRolesDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    pub roles: Vec<LaneRoleTimingDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeroRankingPlayerDto {
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub name: String,
    pub avatar: String,
    #[serde(rename = "rankTier")]
    pub rank_tier: Option<i64>,
    #[serde(rename = "rankLabel")]
    pub rank_label: String,
    #[serde(rename = "rankIcon")]
    pub rank_icon: String,
    #[serde(rename = "rankStarIcon")]
    pub rank_star_icon: String,
    pub score: f64,
    #[serde(rename = "rankPosition")]
    pub rank_position: i64,
    pub percentile: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeroRankingsDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    pub players: Vec<HeroRankingPlayerDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerProfileDto {
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub name: String,
    pub avatar: String,
    #[serde(rename = "rankTier")]
    pub rank_tier: Option<i64>,
    #[serde(rename = "rankLabel")]
    pub rank_label: String,
    #[serde(rename = "rankIcon")]
    pub rank_icon: String,
    #[serde(rename = "rankStarIcon")]
    pub rank_star_icon: String,
    #[serde(rename = "leaderboardRank")]
    pub leaderboard_rank: Option<i64>,
    pub mmr: Option<i64>,
    pub wins: i64,
    pub losses: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
    #[serde(rename = "teamName")]
    pub team_name: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerRecentMatchDto {
    #[serde(rename = "matchId")]
    pub match_id: i64,
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    #[serde(rename = "heroName")]
    pub hero_name: String,
    #[serde(rename = "heroImg")]
    pub hero_img: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub duration: i64,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    pub won: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerHeroStatDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    #[serde(rename = "heroName")]
    pub hero_name: String,
    #[serde(rename = "heroImg")]
    pub hero_img: String,
    pub games: i64,
    pub wins: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
    #[serde(rename = "lastPlayed")]
    pub last_played: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerRatingDto {
    #[serde(rename = "rankTier")]
    pub rank_tier: i64,
    #[serde(rename = "leaderboardRank")]
    pub leaderboard_rank: Option<i64>,
    #[serde(rename = "recordedAt")]
    pub recorded_at: i64,
    pub mmr: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamPlayerDto {
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub name: String,
    #[serde(rename = "gamesPlayed")]
    pub games_played: i64,
    pub wins: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
    #[serde(rename = "isCurrent")]
    pub is_current: bool,
    pub avatar: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    #[serde(rename = "fantasyRole")]
    pub fantasy_role: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamHeroDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    #[serde(rename = "heroName")]
    pub hero_name: String,
    #[serde(rename = "heroImg")]
    pub hero_img: String,
    #[serde(rename = "gamesPlayed")]
    pub games_played: i64,
    pub wins: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchPlayerDto {
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub name: String,
    pub avatar: String,
    #[serde(rename = "lastMatchTime")]
    pub last_match_time: i64,
}

#[derive(Debug, Deserialize)]
pub struct LeagueRaw {
    pub leagueid: Option<i64>,
    pub name: Option<String>,
    pub tier: Option<String>,
    pub banner: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeagueDto {
    #[serde(rename = "leagueId")]
    pub league_id: i64,
    pub name: String,
    pub tier: String,
    pub banner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeagueMatchRaw {
    pub match_id: Option<i64>,
    pub radiant_win: Option<bool>,
    pub start_time: Option<i64>,
    pub duration: Option<i64>,
    pub radiant_score: Option<i64>,
    pub dire_score: Option<i64>,
    pub radiant_team_id: Option<i64>,
    pub radiant_team_name: Option<String>,
    pub dire_team_id: Option<i64>,
    pub dire_team_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordRaw {
    pub match_id: Option<i64>,
    pub start_time: Option<i64>,
    pub hero_id: Option<i64>,
    pub score: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordDto {
    pub field: String,
    #[serde(rename = "matchId")]
    pub match_id: i64,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    #[serde(rename = "heroId")]
    pub hero_id: Option<i64>,
    #[serde(rename = "heroName")]
    pub hero_name: String,
    #[serde(rename = "heroImg")]
    pub hero_img: String,
    pub score: f64,
}

#[derive(Debug, Deserialize)]
pub struct LivePlayerRaw {
    pub hero_id: Option<i64>,
    pub team: Option<i64>,
    pub team_slot: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LiveGameRaw {
    pub match_id: Option<Value>,
    pub game_time: Option<i64>,
    pub average_mmr: Option<f64>,
    pub radiant_score: Option<i64>,
    pub dire_score: Option<i64>,
    pub players: Option<Vec<LivePlayerRaw>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiveHeroDto {
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    #[serde(rename = "heroName")]
    pub hero_name: String,
    #[serde(rename = "heroImg")]
    pub hero_img: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiveGameDto {
    #[serde(rename = "matchId")]
    pub match_id: i64,
    #[serde(rename = "gameTime")]
    pub game_time: i64,
    #[serde(rename = "averageMmr")]
    pub average_mmr: f64,
    #[serde(rename = "radiantScore")]
    pub radiant_score: i64,
    #[serde(rename = "direScore")]
    pub dire_score: i64,
    #[serde(rename = "radiantHeroes")]
    pub radiant_heroes: Vec<LiveHeroDto>,
    #[serde(rename = "direHeroes")]
    pub dire_heroes: Vec<LiveHeroDto>,
}

#[derive(Debug, Deserialize)]
pub struct ProMatchRaw {
    pub match_id: Option<i64>,
    pub duration: Option<i64>,
    pub start_time: Option<i64>,
    pub radiant_team_id: Option<i64>,
    pub radiant_name: Option<String>,
    pub dire_team_id: Option<i64>,
    pub dire_name: Option<String>,
    pub league_id: Option<i64>,
    pub league_name: Option<String>,
    pub radiant_score: Option<i64>,
    pub dire_score: Option<i64>,
    pub radiant_win: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProMatchDto {
    #[serde(rename = "matchId")]
    pub match_id: i64,
    pub duration: i64,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    #[serde(rename = "radiantTeamId")]
    pub radiant_team_id: i64,
    #[serde(rename = "radiantName")]
    pub radiant_name: String,
    #[serde(rename = "direTeamId")]
    pub dire_team_id: i64,
    #[serde(rename = "direName")]
    pub dire_name: String,
    #[serde(rename = "leagueId")]
    pub league_id: i64,
    #[serde(rename = "leagueName")]
    pub league_name: String,
    #[serde(rename = "radiantScore")]
    pub radiant_score: i64,
    #[serde(rename = "direScore")]
    pub dire_score: i64,
    #[serde(rename = "radiantWin")]
    pub radiant_win: bool,
}

#[derive(Debug, Deserialize)]
pub struct TeamMatchRaw {
    pub match_id: Option<i64>,
    pub radiant_score: Option<i64>,
    pub dire_score: Option<i64>,
    pub duration: Option<i64>,
    pub start_time: Option<i64>,
    pub opposing_team_id: Option<i64>,
    pub opposing_team_name: Option<String>,
    pub opposing_team_logo: Option<String>,
    pub league_name: Option<String>,
    pub radiant: Option<bool>,
    pub radiant_win: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchItemDto {
    pub id: i64,
    pub name: String,
    pub img: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchPlayerDto {
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub name: String,
    #[serde(rename = "heroId")]
    pub hero_id: i64,
    #[serde(rename = "heroName")]
    pub hero_name: String,
    #[serde(rename = "heroImg")]
    pub hero_img: String,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub gpm: i64,
    pub xpm: i64,
    #[serde(rename = "netWorth")]
    pub net_worth: i64,
    pub items: Vec<MatchItemDto>,
    pub radiant: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchDetailDto {
    #[serde(rename = "matchId")]
    pub match_id: i64,
    pub duration: i64,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    #[serde(rename = "radiantWin")]
    pub radiant_win: bool,
    #[serde(rename = "radiantScore")]
    pub radiant_score: i64,
    #[serde(rename = "direScore")]
    pub dire_score: i64,
    #[serde(rename = "radiantName")]
    pub radiant_name: String,
    #[serde(rename = "direName")]
    pub dire_name: String,
    #[serde(rename = "leagueId")]
    pub league_id: i64,
    #[serde(rename = "leagueName")]
    pub league_name: String,
    pub patch: i64,
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    pub radiant: Vec<MatchPlayerDto>,
    pub dire: Vec<MatchPlayerDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamMatchupDto {
    pub id: i64,
    pub wins: i64,
    #[serde(rename = "gamesPlayed")]
    pub games_played: i64,
    #[serde(rename = "winRate")]
    pub win_rate: f64,
    pub name: String,
    pub img: String,
    #[serde(rename = "leagueName")]
    pub league_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSearchHeroDto {
    pub id: i64,
    pub name: String,
    pub img: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSearchTeamDto {
    pub id: i64,
    pub name: String,
    pub img: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSearchPlayerDto {
    #[serde(rename = "accountId")]
    pub account_id: i64,
    pub name: String,
    pub avatar: String,
    #[serde(rename = "teamName")]
    pub team_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSearchDto {
    pub heroes: Vec<GlobalSearchHeroDto>,
    pub teams: Vec<GlobalSearchTeamDto>,
    pub players: Vec<GlobalSearchPlayerDto>,
}
