use std::time::Duration;

pub const DEFAULT_OPEN_DOTA_API_URL: &str = "https://api.opendota.com/api";
pub const HERO_URL_BASE: &str =
    "https://cdn.cloudflare.steamstatic.com/apps/dota2/images/dota_react/heroes";
pub const ITEM_CDN_BASE: &str = "https://cdn.cloudflare.steamstatic.com";
pub const RANK_ICON_BASE: &str = "https://www.opendota.com/assets/images/dota2/rank_icons";

// Cache TTLs — lists refresh every 6 h; per-entity matchup data every 24 h
pub const LIST_TTL: Duration = Duration::from_secs(6 * 3600);
pub const MATCHUP_TTL: Duration = Duration::from_secs(24 * 3600);
pub const FEED_TTL: Duration = Duration::from_secs(5 * 60); // pro matches — high velocity content
pub const LIVE_TTL: Duration = Duration::from_secs(2 * 60); // live games — refresh frequently
pub const MATCH_TTL: Duration = Duration::from_secs(7 * 24 * 3600); // match details are immutable
pub const RECORD_TTL: Duration = Duration::from_secs(24 * 3600);

pub const RECORD_FIELDS: &[&str] = &[
    "kills",
    "deaths",
    "assists",
    "gold_per_min",
    "xp_per_min",
    "last_hits",
    "tower_damage",
    "hero_healing",
    "hero_damage",
    "duration",
];
