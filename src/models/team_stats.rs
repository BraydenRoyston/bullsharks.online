use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WeekData {
   #[serde(rename = "weekStart")] 
   pub week_start: DateTime<FixedOffset>,
   #[serde(rename = "weeklyTeamKilometers")]
   pub weekly_team_kilometers: f64,
   #[serde(rename = "weeklyRunningSum")]
   pub weekly_running_sum: f64,
   #[serde(rename = "weeklyAthleteKilometers")]
   pub weekly_athlete_kilometers: HashMap<String, f64>
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