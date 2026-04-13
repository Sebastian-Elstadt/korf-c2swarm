use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::AppContext;
use serde::Serialize;
use std::sync::Arc;

mod app;
mod nodes;

pub fn router() -> Router<Arc<AppContext>> {
    Router::new()
        .route("/api/health", axum::routing::get(app::check_health))
        .route("/api/nodes", axum::routing::get(nodes::list_nodes))
        .route(
            "/api/nodes/{node_id}/logs",
            axum::routing::get(nodes::get_node_logs),
        )
        .route(
            "/api/nodes/{node_id}/commands",
            axum::routing::get(nodes::get_node_command_queue),
        )
        .route(
            "/api/nodes/{node_id}/commands",
            axum::routing::post(nodes::add_node_command),
        )
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

    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            error: "not_found",
            message: msg.into(),
        }
    }

    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error: "bad_request",
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
