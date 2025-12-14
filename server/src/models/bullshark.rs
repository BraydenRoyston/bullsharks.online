/*
These are internal models that we define.
*/

use chrono::{DateTime, Utc};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct BullSharkActivity {
    pub id: String,
    pub date: DateTime<Utc>,
    pub athlete_name: Option<String>,
    pub resource_state: Option<i64>,
    pub name: Option<String>,
    pub distance: Option<f64>,
    pub moving_time: Option<i64>,
    pub elapsed_time: Option<i64>,
    pub total_elevation_gain: Option<f64>,
    pub sport_type: Option<String>,
    pub workout_type: Option<i64>,
    pub device_name: Option<String>
}
