# Dota API Rust Backend

Rust conversion of the Node.js Dota API backend, preserving the existing REST contract used by the Vue frontend.

## Endpoints

- `GET /heroes?page=1&pageSize=30`
- `GET /hero/:id`
- `GET /hero-matchup/:id?page=1&pageSize=18`
- `GET /pro-players`
- `GET /pro-matches?page=1&pageSize=30`
- `GET /match/:id`
- `GET /pro-teams?page=1&pageSize=30`
- `GET /team/:id`
- `GET /team-matchup/:id?page=1&pageSize=18`

## Tech

- Axum
- Reqwest
- Tokio
- Serde
- Tower HTTP (CORS + tracing)

## Setup

1. Copy env file:
   - `cp .env.example .env`
2. Run:
   - `cargo run`

Server runs on `http://localhost:8000` by default.

### CORS

Set `DOTA_SITE` as a comma-separated origin allowlist, for example:
`DOTA_SITE=http://localhost:8080,https://dota2-companion.vercel.app,https://dota2-companion.johnmichealacera.com`

## Notes

- Response shapes are aligned with your existing Node backend contract to avoid frontend changes.
- Uses lightweight in-memory caching per process (no Redis required for this first migration pass).
