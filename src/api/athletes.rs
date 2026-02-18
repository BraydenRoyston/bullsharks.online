use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    error::ApiError,
    models::{athlete::Athlete, athlete_training_data::AllAthletesTrainingData},
    services::{activity_controller::ActivityController, database::Database},
};

/// Retrieve all registered athletes and their team assignments.
///
/// This endpoint returns the complete list of athletes who are registered in the
/// BullSharks running club system, including their team assignment (Bulls or Sharks)
/// and event information.
///
/// # Team Assignment
///
/// Athletes are assigned to teams through the database configuration:
/// - **Bulls**: One team in the competition
/// - **Sharks**: The opposing team in the competition
///
/// Team assignments are used to calculate team statistics and determine
/// which activities count toward which team's totals.
///
/// # Returns
///
/// * `Json<Vec<Athlete>>` - JSON array of all registered athletes
///
/// # Response Structure
///
/// ```json
/// [
///   {
///     "id": 1,
///     "name": "John Doe",
///     "team": "bulls",
///     "event": "marathon"
///   },
///   {
///     "id": 2,
///     "name": "Jane Smith",
///     "team": "sharks",
///     "event": "half-marathon"
///   }
/// ]
/// ```
///
/// # Errors
///
/// * `ApiError::DatabaseError` - Database query failure
///
/// # Usage
///
/// This endpoint is used to:
/// - Display team rosters in the UI
/// - Map activity data to team assignments
/// - Show individual athlete information
pub async fn get_athletes(State(db): State<Arc<Database>>) -> Result<Json<Vec<Athlete>>, ApiError> {
    let result = db.read_all_athletes().await?;
    Ok(Json(result))
}

/// Retrieve comprehensive training data and injury risk analysis for all athletes.
///
/// This endpoint performs sophisticated analysis on each athlete's training patterns,
/// calculating injury risk using two established algorithms:
///
/// # Risk Analysis Algorithms
///
/// ## SSRD30 (Spike in Single Run Distance - 30 Day)
/// - Compares each run against the longest run in the preceding 30 days
/// - **Small Risk**: 10-30% increase over baseline
/// - **Moderate Risk**: 30-100% increase over baseline  
/// - **Large Risk**: >100% increase over baseline
///
/// ## 10% Rule (Weekly Volume Spike)
/// - Detects week-over-week training volume increases >10%
/// - Identifies rapid training progression that may lead to injury
/// - Only triggers for athletes with >20km weekly volume
///
/// # Time Zone Handling
///
/// - Week boundaries calculated in Pacific Time (America/Los_Angeles)
/// - Monday 00:00:00 to Sunday 23:59:59 defines each training week
/// - Historical analysis covers all available activity data
///
/// # Returns
///
/// * `Json<AllAthletesTrainingData>` - Comprehensive training analysis for all athletes
///
/// # Response Structure
///
/// ```json
/// {
///   "athletes_training_data": [
///     {
///       "athlete_name": "John Doe",
///       "weekly_kilometers": [[week_start, total_km], ...],
///       "risky_weeks": {
///         "2024-01-15": {
///           "risks": ["SSRD30_MODERATE_RISK: 45.2% increase over 30-day baseline"]
///         }
///       }
///     }
///   ]
/// }
/// ```
///
/// # Errors
///
/// * `ApiError::DatabaseError` - Database query failure
/// * `ApiError::InternalConversionError` - Date calculation or analysis errors
///
/// # Usage
///
/// This endpoint powers:
/// - Individual athlete training dashboards
/// - Injury risk monitoring and alerts
/// - Training load progression tracking
/// - Coach/athlete communication about training safety
pub async fn get_athletes_training_data(
    State(activity_controller): State<Arc<ActivityController>>,
) -> Result<Json<AllAthletesTrainingData>, ApiError> {
    let training_data = activity_controller.get_all_athletes_training_data().await?;
    Ok(Json(training_data))
}
