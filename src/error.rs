use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tracing::error;

#[derive(Debug)]
pub enum ApiError {
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
