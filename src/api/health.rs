use axum::{Json, extract::State};
use serde::Serialize;

use crate::utils::startup_utils::AppState;

#[derive(Serialize)]
pub struct HealthStatus {
    pub database: String,
    pub strava: String,
    pub overall: String,
}

pub async fn health_check(State(state): State<AppState>) -> Json<HealthStatus> {
    let db_status = match state.db.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(e) => format!("unhealthy: {:?}", e),
    };

    let strava_status = match state.activity_controller.health_check_strava().await {
        Ok(_) => "healthy".to_string(),
        Err(e) => format!("unhealthy: {:?}", e),
    };

    let overall = if db_status == "healthy" && strava_status == "healthy" {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    Json(HealthStatus {
        database: db_status,
        strava: strava_status,
        overall,
    })
}
