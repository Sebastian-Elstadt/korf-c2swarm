use axum::{Json, Router, http::StatusCode, response::{IntoResponse, Response}};
use domain::AppContext;
use serde::Serialize;
use std::sync::Arc;

mod app;
mod nodes;

pub fn router() -> Router<Arc<AppContext>> {
    Router::new()
        .route("/api/health", axum::routing::get(app::check_health))
        .route("/api/nodes", axum::routing::get(nodes::list_nodes))
}

#[derive(Serialize)]
struct ApiErrorBody {
    error: &'static str,
    message: String,
}

struct ApiError {
    status: StatusCode,
    error: &'static str,
    message: String,
}

impl ApiError {
    fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "internal_error",
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: self.error,
                message: self.message,
            }),
        )
            .into_response()
    }
}
