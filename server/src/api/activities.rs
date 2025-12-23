use std::sync::Arc;

use axum::{Json, extract::State, http::{StatusCode, HeaderMap}};
use chrono::{Datelike, Duration, TimeZone, Utc};
use chrono_tz::America::Los_Angeles;

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

pub async fn get_activities_from_this_week(
    State(db): State<Arc<Database>>
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    // Get current time in Pacific timezone
    let now_pacific = Los_Angeles.from_utc_datetime(&Utc::now().naive_utc());

    // Calculate start of week (Sunday 00:00:00) in Pacific
    let days_since_monday= now_pacific.weekday().num_days_from_monday();
    let start_of_week_pacific = now_pacific
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        - Duration::days(days_since_monday as i64);
    let start_of_week_pacific = Los_Angeles.from_local_datetime(&start_of_week_pacific).single()
        .ok_or_else(|| ApiError::InternalConversionError("Invalid start of week time".to_string()))?;

    // Calculate end of week (Saturday 23:59:59) in Pacific
    let end_of_week_pacific = start_of_week_pacific
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        + Duration::days(6);
    let end_of_week_pacific = Los_Angeles.from_local_datetime(&end_of_week_pacific).single()
        .ok_or_else(|| ApiError::InternalConversionError("Invalid end of week time".to_string()))?;

    // Convert to UTC for database query
    let start_utc = start_of_week_pacific.with_timezone(&Utc);
    let end_utc = end_of_week_pacific.with_timezone(&Utc);

    println!("[API] get_activities_from_this_week: Querying from {} to {}", start_utc, end_utc);

    // Query database
    let activities = db.get_activities_from_window(start_utc, end_utc).await?;
    Ok(Json(activities))
}
