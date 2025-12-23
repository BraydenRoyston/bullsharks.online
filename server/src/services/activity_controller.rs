use std::sync::Arc;

use crate::{error::ApiError, models::{bullshark::BullSharkActivity, club::ClubActivity}, services::{database::Database, strava_client::{StravaClient}}};
use chrono::{DateTime, FixedOffset, Utc};
use sha2::{Digest, Sha256};

pub struct ActivityController {
    db: Arc<Database>,
    strava_client: StravaClient, 
}

impl ActivityController {
    pub fn new(db: Arc<Database>, strava_client: StravaClient) -> Self {
        ActivityController { 
            db,
            strava_client
        }
    }

    pub async fn populate_new_activities(&self) -> Result<(), ApiError> {
        println!("Populating new activities...");
        let new_activities = self.strava_client.read_last_100_activities().await?;
        println!("Found {} new activities...", new_activities.len());
        let new_bullshark_activities = self.convert_activities(&new_activities)?;
        println!("Inserting bullshark activities to the database...");
        self.db.insert_activities(&new_bullshark_activities).await?;
        println!("Populate new activities complete.");
        Ok(())
    }

    pub fn convert_activities(&self, club_activities: &[ClubActivity]) -> Result<Vec<BullSharkActivity>, ApiError> {
        // Get current UTC time and convert to FixedOffset for model compatibility
        let batch_time = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());

        club_activities
            .iter()
            .map(|activity| self.convert_activity_to_bullshark_activity(activity, batch_time))
            .collect()
    }

    pub fn convert_activity_to_bullshark_activity(&self, club_activity: &ClubActivity, time: DateTime<FixedOffset>) -> Result<BullSharkActivity, ApiError> {
        let hash = self.create_hash_for_activity(club_activity)?;
        let athlete = club_activity.athlete
            .as_ref()
            .ok_or(ApiError::ExternalAPIError("Strava athlete did not contain first/last name".to_string()))?;
        let athlete_name = format!(
            "{} {}",
            athlete.first_name.as_deref().unwrap_or("Unknown"),
            athlete.last_name.as_deref().unwrap_or("Unknown")
        );

        Ok(BullSharkActivity {
            id: hash,
            date: time, 
            athlete_name: Some(athlete_name),
            resource_state: club_activity.resource_state,
            name: club_activity.name.clone(),
            distance: club_activity.distance,
            moving_time: club_activity.moving_time,
            elapsed_time: club_activity.elapsed_time,
            total_elevation_gain: club_activity.total_elevation_gain,
            sport_type: club_activity.sport_type.clone(),
            workout_type: club_activity.workout_type,
            device_name: club_activity.device_name.clone()
        })
    }

    pub fn create_hash_for_activity(&self, club_activity: &ClubActivity) -> Result<String, ApiError> {
        let athlete = club_activity.athlete
          .as_ref()
          .ok_or_else(|| ApiError::InternalConversionError("Activity missing athlete".to_string()))?;

      let first_name = athlete.first_name
          .as_ref()
          .ok_or_else(|| ApiError::InternalConversionError("Athlete missing first name".to_string()))?;

      let last_name = athlete.last_name
          .as_ref()
          .ok_or_else(|| ApiError::InternalConversionError("Athlete missing last name".to_string()))?;

      let distance = club_activity.distance
          .ok_or_else(|| ApiError::InternalConversionError("Activity missing distance".to_string()))?;

      let moving_time = club_activity.moving_time
          .ok_or_else(|| ApiError::InternalConversionError("Activity missing moving time".to_string()))?;

      let elapsed_time = club_activity.elapsed_time
          .ok_or_else(|| ApiError::InternalConversionError("Activity missing elapsed time".to_string()))?;

      let composite = format!(
          "{}|{}|{}|{}|{}",
          first_name,
          last_name,
          distance,
          moving_time,
          elapsed_time
      );

        let mut hasher = Sha256::new();
        hasher.update(composite.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub async fn health_check_strava(&self) -> Result<(), ApiError> {
        // Attempt to verify we can get a valid Strava auth token
        self.strava_client.health_check().await
    }
}
