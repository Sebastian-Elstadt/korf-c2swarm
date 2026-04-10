use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use domain::AppContext;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

pub async fn health(State(app_ctx): State<Arc<AppContext>>) -> impl IntoResponse {
    if app_ctx.health_port.ping_db().await {
        return (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                database: "ok",
            }),
        );
    }

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(HealthResponse {
            status: "degraded",
            database: "error",
        }),
    )
}
