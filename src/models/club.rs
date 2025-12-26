/*
These are external models defined by Strava.
*/

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ClubActivity {
    pub resource_state: Option<i64>,
    pub athlete: Option<ClubAthlete>,
    pub name: Option<String>,
    pub distance: Option<f64>,
    pub moving_time: Option<i64>,
    pub elapsed_time: Option<i64>,
    pub total_elevation_gain: Option<f64>,
    pub sport_type: Option<String>,
    pub workout_type: Option<i64>,
    pub device_name: Option<String>
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ClubAthlete {
    pub resource_state: Option<i64>,
    #[serde(rename = "firstname")]
    pub first_name: Option<String>,
    #[serde(rename = "lastname")]
    pub last_name: Option<String>,
}
