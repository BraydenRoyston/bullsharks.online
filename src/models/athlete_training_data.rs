use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AthleteWeeklyData {
    /// Maps week start date (YYYY-MM-DD format) to kilometers
    #[serde(rename = "weeklyKilometers")]
    pub weekly_kilometers: HashMap<String, f64>,
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
