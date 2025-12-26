use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// Response structures for get_team_stats
#[derive(Serialize, Deserialize, Debug)]
pub struct TeamData {
    #[serde(rename = "athleteKilometers")]
    pub athlete_kilometers: HashMap<String, f64>,
    #[serde(rename = "weeklyKilometers")]
    pub weekly_kilometers: HashMap<String, f64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamStats {
    pub bulls: TeamData,
    pub sharks: TeamData,
}