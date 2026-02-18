use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use chrono_tz::America::Los_Angeles;
use serde::Deserialize;

use crate::{
    error::ApiError,
    models::{bullshark::BullSharkActivity, team_stats::TeamStats},
    services::{activity_controller::ActivityController, database::Database},
};

/// Retrieve all stored BullShark activities from the database.
///
/// This endpoint returns all activities that have been synchronized from Strava,
/// converted to the internal BullSharkActivity format. Activities are stored in UTC
/// but contain timezone information for display purposes.
///
/// # Returns
///
/// * `Json<Vec<BullSharkActivity>>` - JSON array of all stored activities
///
/// # Errors
///
/// Returns `ApiError::DatabaseError` if the database query fails.
///
/// # Example Response
///
/// ```json
/// [
///   {
///     "id": "activity_123",
///     "date": "2024-01-15T10:00:00-08:00",
///     "athlete_name": "John Doe",
///     "distance": 5000.0,
///     "sport_type": "Run"
///   }
/// ]
/// ```
pub async fn read_activities(
    State(db): State<Arc<Database>>,
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    let activities = db.get_all_activities().await?;
    Ok(Json(activities))
}

/// Manually trigger synchronization of new activities from Strava.
///
/// This endpoint fetches the latest activities from the Strava Club API and stores
/// any new activities in the database. It includes deduplication logic to prevent
/// storing the same activity multiple times.
///
/// # Authentication
///
/// This endpoint is protected by an optional `X-CloudScheduler-Token` header.
/// If the `CRON_SECRET` environment variable is set, the token must match it.
/// This allows the endpoint to be called by automated systems like Google Cloud Scheduler.
///
/// # Parameters
///
/// * `headers` - HTTP headers, may contain `X-CloudScheduler-Token` for authentication
/// * `controller` - ActivityController for managing the synchronization process
///
/// # Returns
///
/// * `StatusCode::OK` - Activities successfully synchronized
///
/// # Errors
///
/// * `ApiError::Unauthorized` - Invalid or missing authentication token
/// * `ApiError::ExternalAPIError` - Strava API connection or rate limit issues
/// * `ApiError::DatabaseError` - Database storage issues
/// * `ApiError::InternalConversionError` - Data transformation errors
///
/// # Usage
///
/// This endpoint is typically called:
/// - By Google Cloud Scheduler every 2 minutes for automatic syncing
/// - Manually for immediate updates during development/testing
pub async fn populate_activities(
    headers: HeaderMap,
    State(controller): State<Arc<ActivityController>>,
) -> Result<StatusCode, ApiError> {
    // Security: Check for secret token
    let cron_secret = std::env::var("CRON_SECRET").unwrap_or_else(|_| "".to_string());

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

/// Retrieve all activities from the current week (Monday to Sunday).
///
/// This endpoint calculates the current week boundaries in Pacific Time (Los Angeles timezone)
/// and returns all activities that occurred within this time window. The week runs from
/// Monday 00:00:00 to Sunday 23:59:59 in Pacific Time.
///
/// # Time Zone Handling
///
/// - Week boundaries calculated in Pacific Time (America/Los_Angeles)
/// - Database queries performed in UTC
/// - Returned activities contain original timezone information
///
/// # Returns
///
/// * `Json<Vec<BullSharkActivity>>` - JSON array of activities from the current week
///
/// # Errors
///
/// * `ApiError::InternalConversionError` - Invalid timezone conversion
/// * `ApiError::DatabaseError` - Database query failure
///
/// # Example
///
/// If called on Wednesday, January 17, 2024, returns activities from:
/// - Start: Monday, January 15, 2024 00:00:00 PST
/// - End: Sunday, January 21, 2024 23:59:59 PST
pub async fn get_activities_from_this_week(
    State(db): State<Arc<Database>>,
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    // Get current time in Pacific timezone
    let now_pacific = Los_Angeles.from_utc_datetime(&Utc::now().naive_utc());

    // Calculate start of week (Sunday 00:00:00) in Pacific
    let days_since_monday = now_pacific.weekday().num_days_from_monday();
    let start_of_week_pacific = now_pacific.date_naive().and_hms_opt(0, 0, 0).unwrap()
        - Duration::days(days_since_monday as i64);
    let start_of_week_pacific = Los_Angeles
        .from_local_datetime(&start_of_week_pacific)
        .single()
        .ok_or_else(|| {
            ApiError::InternalConversionError("Invalid start of week time".to_string())
        })?;

    // Calculate end of week (Saturday 23:59:59) in Pacific
    let end_of_week_pacific = start_of_week_pacific
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        + Duration::days(6);
    let end_of_week_pacific = Los_Angeles
        .from_local_datetime(&end_of_week_pacific)
        .single()
        .ok_or_else(|| ApiError::InternalConversionError("Invalid end of week time".to_string()))?;

    // Convert to UTC for database query
    let start_utc = start_of_week_pacific.with_timezone(&Utc);
    let end_utc = end_of_week_pacific.with_timezone(&Utc);

    println!(
        "[API] get_activities_from_this_week: Querying from {} to {}",
        start_utc, end_utc
    );

    // Query database
    let activities = db.get_activities_from_window(start_utc, end_utc).await?;
    Ok(Json(activities))
}

/// Retrieve all activities from the current month.
///
/// This endpoint calculates the current month boundaries in Pacific Time and returns
/// all activities that occurred from the 1st day of the month at 00:00:00 through
/// the last day of the month at 23:59:59 in Pacific Time.
///
/// # Time Zone Handling
///
/// - Month boundaries calculated in Pacific Time (America/Los_Angeles)  
/// - Handles month transitions and leap years correctly
/// - Database queries performed in UTC
/// - Returned activities contain original timezone information
///
/// # Returns
///
/// * `Json<Vec<BullSharkActivity>>` - JSON array of activities from the current month
///
/// # Errors
///
/// * `ApiError::InternalConversionError` - Invalid date calculations or timezone conversion
/// * `ApiError::DatabaseError` - Database query failure
///
/// # Example
///
/// If called in January 2024, returns activities from:
/// - Start: January 1, 2024 00:00:00 PST
/// - End: January 31, 2024 23:59:59 PST
///
/// Correctly handles edge cases like February in leap years and December to January transitions.
pub async fn get_activities_from_this_month(
    State(db): State<Arc<Database>>,
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    // Get current time in Pacific timezone
    let now_pacific = Los_Angeles.from_utc_datetime(&Utc::now().naive_utc());

    // Calculate start of month (1st day at 00:00:00) in Pacific
    let start_of_month_pacific = now_pacific
        .date_naive()
        .with_day(1)
        .ok_or_else(|| {
            ApiError::InternalConversionError("Invalid start of month date".to_string())
        })?
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let start_of_month_pacific = Los_Angeles
        .from_local_datetime(&start_of_month_pacific)
        .single()
        .ok_or_else(|| {
            ApiError::InternalConversionError("Invalid start of month time".to_string())
        })?;

    // Calculate end of month (last day at 23:59:59) in Pacific
    // Get the first day of next month, then subtract 1 second to get end of current month
    let next_month = if now_pacific.month() == 12 {
        now_pacific
            .date_naive()
            .with_year(now_pacific.year() + 1)
            .and_then(|d| d.with_month(1))
    } else {
        now_pacific.date_naive().with_month(now_pacific.month() + 1)
    }
    .ok_or_else(|| ApiError::InternalConversionError("Invalid next month date".to_string()))?
    .and_hms_opt(0, 0, 0)
    .unwrap();

    let end_of_month_pacific = Los_Angeles
        .from_local_datetime(&next_month)
        .single()
        .ok_or_else(|| {
            ApiError::InternalConversionError("Invalid end of month time".to_string())
        })?
        - Duration::seconds(1);

    // Convert to UTC for database query
    let start_utc = start_of_month_pacific.with_timezone(&Utc);
    let end_utc = end_of_month_pacific.with_timezone(&Utc);

    println!(
        "[API] get_activities_from_this_month: Querying from {} to {}",
        start_utc, end_utc
    );

    // Query database
    let activities = db.get_activities_from_window(start_utc, end_utc).await?;
    Ok(Json(activities))
}

/// Query parameters for retrieving activities within a custom time window.
///
/// Both start and end times must be provided in RFC3339 format (ISO 8601).
/// Times should be in UTC or include timezone information.
///
/// # Fields
///
/// * `start` - Start datetime in RFC3339 format (e.g., "2024-01-01T00:00:00Z")
/// * `end` - End datetime in RFC3339 format (e.g., "2024-01-31T23:59:59Z")
///
/// # Example URL
///
/// ```
/// /activities/window?start=2024-01-01T00:00:00Z&end=2024-01-31T23:59:59Z
/// ```
#[derive(Deserialize)]
pub struct WindowQuery {
    start: String,
    end: String,
}

/// Retrieve activities within a custom time window specified by query parameters.
///
/// This endpoint allows clients to specify exact start and end times for activity retrieval.
/// Both parameters are required and must be in RFC3339 format (ISO 8601).
///
/// # Query Parameters
///
/// * `start` - Start datetime in RFC3339 format (inclusive)
/// * `end` - End datetime in RFC3339 format (inclusive)  
///
/// # Time Format
///
/// Times must be in RFC3339 format. Examples:
/// - UTC: `2024-01-01T00:00:00Z`
/// - With timezone: `2024-01-01T00:00:00-08:00`
/// - Milliseconds: `2024-01-01T00:00:00.000Z`
///
/// # Returns
///
/// * `Json<Vec<BullSharkActivity>>` - JSON array of activities within the specified window
///
/// # Errors
///
/// * `ApiError::BadRequest` - Invalid datetime format or missing parameters
/// * `ApiError::DatabaseError` - Database query failure
///
/// # Example Request
///
/// ```
/// GET /activities/window?start=2024-01-01T00:00:00Z&end=2024-01-31T23:59:59Z
/// ```
///
/// Returns all activities from January 2024.
pub async fn get_activities_from_custom_window(
    Query(params): Query<WindowQuery>,
    State(db): State<Arc<Database>>,
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    // Parse the datetime strings into DateTime<Utc>
    let start_utc = params.start.parse::<DateTime<Utc>>()
        .map_err(|e| ApiError::BadRequest(format!("Invalid start datetime format: {}. Expected RFC3339 format (e.g., 2024-01-01T00:00:00Z)", e)))?;

    let end_utc = params.end.parse::<DateTime<Utc>>().map_err(|e| {
        ApiError::BadRequest(format!(
            "Invalid end datetime format: {}. Expected RFC3339 format (e.g., 2024-01-31T23:59:59Z)",
            e
        ))
    })?;

    println!(
        "[API] get_activities_from_custom_window: Querying from {} to {}",
        start_utc, end_utc
    );

    // Query database
    let activities = db.get_activities_from_window(start_utc, end_utc).await?;
    Ok(Json(activities))
}

/// Retrieve Bulls vs Sharks team competition statistics.
///
/// This endpoint calculates comprehensive team statistics for the Bulls vs Sharks
/// running competition, including weekly breakdowns, running totals, and individual
/// athlete contributions. Statistics are calculated across all stored activities.
///
/// # Team Competition Rules
///
/// - Teams are assigned based on athlete records in the database
/// - Only "Run" activities are counted (other sports filtered out)
/// - Week boundaries: Monday 00:00:00 to Sunday 23:59:59 Pacific Time
/// - Statistics include both current week and historical data
///
/// # Returns
///
/// * `Json<TeamStats>` - Comprehensive team competition statistics
///
/// # Response Structure
///
/// ```json
/// {
///   "bulls": {
///     "weekly_kilometers": [[week_start, total_km], ...],
///     "athlete_kilometers": {"John Doe": 45.2, ...}
///   },
///   "sharks": {
///     "weekly_kilometers": [[week_start, total_km], ...],
///     "athlete_kilometers": {"Jane Smith": 38.7, ...}
///   }
/// }
/// ```
///
/// # Errors
///
/// * `ApiError::DatabaseError` - Database query failure
/// * `ApiError::InternalConversionError` - Team assignment or calculation errors
///
/// # Usage
///
/// This endpoint powers the main competition dashboard showing weekly progress
/// and leaderboards for both teams.
pub async fn get_team_stats(
    State(activity_controller): State<Arc<ActivityController>>,
) -> Result<Json<TeamStats>, ApiError> {
    let team_stats = activity_controller.get_team_stats().await?;

    Ok(Json(team_stats))
}
