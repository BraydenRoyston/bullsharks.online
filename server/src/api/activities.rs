use std::sync::Arc;

use axum::{Json, extract::State, http::{StatusCode, HeaderMap}};

use crate::{error::ApiError, models::bullshark::BullSharkActivity, services::{database::Database, activity_controller::ActivityController}};

pub async fn read_activities(
    State(db): State<Arc<Database>>
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    let activities = db.get_all_activities().await?;
    Ok(Json(activities))
}

pub async fn populate_activities(
    headers: HeaderMap,
    State(controller): State<Arc<ActivityController>>
) -> Result<StatusCode, ApiError> {
    // Security: Check for secret token
    let cron_secret = std::env::var("CRON_SECRET")
        .unwrap_or_else(|_| "".to_string());

    if !cron_secret.is_empty() {
        let auth_header = headers
            .get("X-CloudScheduler-Token")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        if auth_header != cron_secret {
            println!("Unauthorized populate attempt");
            return Err(ApiError::Unauthorized("Invalid token".to_string()));
        }
    }

    println!("Manual populate triggered via /populate endpoint");
    controller.populate_new_activities().await?;

    Ok(StatusCode::OK)
}
