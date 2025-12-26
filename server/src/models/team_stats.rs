use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WeekData {
   #[serde(rename = "weekStart")] 
   pub week_start: DateTime<FixedOffset>,
   pub weekly_team_kilometers: f64,
   pub weekly_running_sum: f64,
}

// Response structures for get_team_stats
#[derive(Serialize, Deserialize, Debug)]
pub struct TeamData {
    #[serde(rename = "athleteKilometers")]
    pub athlete_kilometers: HashMap<String, f64>,
    #[serde(rename = "weeklyKilometers")]
    pub weekly_kilometers: Vec<WeekData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamStats {
    pub bulls: TeamData,
    pub sharks: TeamData,
}