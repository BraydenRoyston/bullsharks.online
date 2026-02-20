use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a week with detected injury risks
#[derive(Serialize, Deserialize, Debug)]
pub struct RiskyWeek {
    /// Week start date in ISO 8601 format (YYYY-MM-DD)
    pub week: String,
    /// Number of injury risks detected for this week
    #[serde(rename = "riskCount")]
    pub risk_count: usize,
    /// Array of injury risk identifiers (e.g., "HIGH_VOLUME_SPIKE")
    pub risks: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AthleteWeeklyData {
    /// Maps week start date (YYYY-MM-DD format) to kilometers
    #[serde(rename = "weeklyKilometers")]
    pub weekly_kilometers: HashMap<String, f64>,
    /// Array of weeks with detected injury risks
    #[serde(rename = "riskyWeeks")]
    pub risky_weeks: Vec<RiskyWeek>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AthleteTrainingData {
    /// Athlete metadata
    pub id: String,
    pub name: String,
    pub team: String,
    pub event: String,
    /// Weekly training data
    #[serde(rename = "trainingData")]
    pub training_data: AthleteWeeklyData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AllAthletesTrainingData {
    pub athletes: Vec<AthleteTrainingData>,
}
