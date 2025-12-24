use std::{sync::Arc, collections::HashMap};

use axum::{Json, extract::State, http::{StatusCode, HeaderMap}};
use chrono::{Datelike, Duration, TimeZone, Utc};
use chrono_tz::America::Los_Angeles;
use dashmap::DashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::{error::ApiError, models::{athlete::Athlete, bullshark::BullSharkActivity}, services::{activity_controller::ActivityController, database::Database}};

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

pub async fn get_activities_from_this_month(
    State(db): State<Arc<Database>>
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    // Get current time in Pacific timezone
    let now_pacific = Los_Angeles.from_utc_datetime(&Utc::now().naive_utc());

    // Calculate start of month (1st day at 00:00:00) in Pacific
    let start_of_month_pacific = now_pacific
        .date_naive()
        .with_day(1)
        .ok_or_else(|| ApiError::InternalConversionError("Invalid start of month date".to_string()))?
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let start_of_month_pacific = Los_Angeles.from_local_datetime(&start_of_month_pacific).single()
        .ok_or_else(|| ApiError::InternalConversionError("Invalid start of month time".to_string()))?;

    // Calculate end of month (last day at 23:59:59) in Pacific
    // Get the first day of next month, then subtract 1 second to get end of current month
    let next_month = if now_pacific.month() == 12 {
        now_pacific.date_naive()
            .with_year(now_pacific.year() + 1)
            .and_then(|d| d.with_month(1))
    } else {
        now_pacific.date_naive()
            .with_month(now_pacific.month() + 1)
    }
    .ok_or_else(|| ApiError::InternalConversionError("Invalid next month date".to_string()))?
    .and_hms_opt(0, 0, 0)
    .unwrap();

    let end_of_month_pacific = Los_Angeles.from_local_datetime(&next_month).single()
        .ok_or_else(|| ApiError::InternalConversionError("Invalid end of month time".to_string()))?
        - Duration::seconds(1);

    // Convert to UTC for database query
    let start_utc = start_of_month_pacific.with_timezone(&Utc);
    let end_utc = end_of_month_pacific.with_timezone(&Utc);

    println!("[API] get_activities_from_this_month: Querying from {} to {}", start_utc, end_utc);

    // Query database
    let activities = db.get_activities_from_window(start_utc, end_utc).await?;
    Ok(Json(activities))
}

pub async fn update_athletes(
    State(db): State<Arc<Database>>
) -> Result<String, ApiError> {
    let activities = db.get_all_activities().await?;
    let dashmap: DashMap<String, i64> = DashMap::new();

    for activity in activities {
        let athlete_name = activity.athlete_name.expect("update athlete error");

        if dashmap.contains_key(&athlete_name) {
            continue;
        }

        let athlete: Athlete = Athlete { 
            id: Uuid::new_v4().to_string(), 
            name: athlete_name.clone(), 
            team: "bulls".to_string(), 
            event: "placeholder".to_string() 
        };

        let _result = db.insert_athlete(&athlete).await?;
        dashmap.insert(athlete_name, 1);
    }

    Ok("ok".to_string())
}

// Response structures for get_team_stats
#[derive(Serialize, Deserialize, Debug)]
pub struct TeamData {
    #[serde(rename = "athleteMiles")]
    pub athlete_miles: HashMap<String, f64>,
    #[serde(rename = "weeklyMiles")]
    pub weekly_miles: HashMap<String, f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamStats {
    pub bulls: TeamData,
    pub sharks: TeamData,
}

pub async fn get_team_stats(
    State(db): State<Arc<Database>>
) -> Result<Json<TeamStats>, ApiError> {
    // Get all athletes and build hashmap of athlete_name -> team
    let athletes = db.read_all_athletes().await?;
    let mut athlete_teams: HashMap<String, String> = HashMap::new();
    for athlete in athletes {
        athlete_teams.insert(athlete.name.clone(), athlete.team.clone());
    }

    // Create start date: January 1st, 2026 at 00:00:00 Pacific
    let start_date_naive = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
        .ok_or_else(|| ApiError::InternalConversionError("Invalid start date".to_string()))?
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| ApiError::InternalConversionError("Invalid start time".to_string()))?;
    let start_date_pacific = Los_Angeles.from_local_datetime(&start_date_naive).single()
        .ok_or_else(|| ApiError::InternalConversionError("Invalid start date time".to_string()))?;
    let start_date_utc = start_date_pacific.with_timezone(&Utc);

    // Get current time for end date
    let end_date_utc = Utc::now();

    println!("[API] get_team_stats: Querying activities from {} to {}", start_date_utc, end_date_utc);

    // Get all activities from January 1st, 2026 onwards
    let activities = db.get_activities_from_window(start_date_utc, end_date_utc).await?;

    // Initialize team data structures
    let mut bulls_athlete_miles: HashMap<String, f64> = HashMap::new();
    let mut bulls_weekly_miles: HashMap<String, f64> = HashMap::new();
    let mut sharks_athlete_miles: HashMap<String, f64> = HashMap::new();
    let mut sharks_weekly_miles: HashMap<String, f64> = HashMap::new();

    // Process each activity
    for activity in activities {
        // Check if it's a run - reject if not
        if let Some(sport_type) = &activity.sport_type {
            if sport_type != "Run" {
                continue;
            }
        } else {
            continue;
        }

        // Get athlete name
        let athlete_name = match &activity.athlete_name {
            Some(name) => name,
            None => continue,
        };

        // Find athlete team from hashmap
        let team = match athlete_teams.get(athlete_name) {
            Some(t) => t,
            None => continue,
        };

        // Get distance in meters and convert to miles
        let distance_meters = match activity.distance {
            Some(d) => d,
            None => continue,
        };
        let distance_miles = distance_meters / 1609.34;

        // Update athlete miles for the athlete
        let athlete_miles = match team.as_str() {
            "bulls" => &mut bulls_athlete_miles,
            "sharks" => &mut sharks_athlete_miles,
            _ => continue,
        };
        *athlete_miles.entry(athlete_name.clone()).or_insert(0.0) += distance_miles;

        // Find week that activity belongs to (start of week - Monday)
        let activity_date = activity.date;
        let activity_date_naive = activity_date.naive_local();
        let days_since_monday = activity_date_naive.weekday().num_days_from_monday();
        let start_of_week = activity_date_naive.date()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            - Duration::days(days_since_monday as i64);

        // Format week as "Month Day" (e.g., "December 22")
        let week_key = start_of_week.format("%B %-d").to_string();

        // Update weekly miles for that week
        let weekly_miles = match team.as_str() {
            "bulls" => &mut bulls_weekly_miles,
            "sharks" => &mut sharks_weekly_miles,
            _ => continue,
        };
        *weekly_miles.entry(week_key).or_insert(0.0) += distance_miles;
    }

    // Build response
    let team_stats = TeamStats {
        bulls: TeamData {
            athlete_miles: bulls_athlete_miles,
            weekly_miles: bulls_weekly_miles,
        },
        sharks: TeamData {
            athlete_miles: sharks_athlete_miles,
            weekly_miles: sharks_weekly_miles,
        },
    };

    println!("[API] get_team_stats: Successfully calculated team stats");
    Ok(Json(team_stats))
}
