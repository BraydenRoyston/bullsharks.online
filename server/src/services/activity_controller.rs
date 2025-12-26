use std::{collections::HashMap, sync::Arc};

use crate::{error::ApiError, models::{bullshark::BullSharkActivity, club::ClubActivity, team_stats::{TeamData, TeamStats}}, services::{database::Database, strava_client::StravaClient}};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDateTime, TimeZone, Utc};
use chrono_tz::America::Los_Angeles;
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

    pub async fn get_team_stats(&self) -> Result<TeamStats, ApiError> {
        let athlete_teams = self.build_athlete_team_map().await?;
        let (start_date, end_date) = self.get_team_stat_dates()?;

        println!("[ACTIVITY_CONTROLLER]: getting team stats from {} to {}", start_date, end_date);

        let activities = self.db.get_activities_from_window(start_date, end_date).await?;

        let mut bulls_athlete_kilometers: HashMap<String, f64> = HashMap::new();
        let mut bulls_weekly_kilometers: HashMap<String, f64> = HashMap::new();
        let mut sharks_athlete_kilometers: HashMap<String, f64> = HashMap::new();
        let mut sharks_weekly_kilometers: HashMap<String, f64> = HashMap::new();

        // O(n) over each activity
        for activity in activities {
            if !self.valid_activity(&activity) {
                continue;
            }

            // Get athlete name
            let athlete_name = match &activity.athlete_name {
                Some(name) => name,
                None => continue,
            };

            // Get athlete team
            let team = match athlete_teams.get(athlete_name) {
                Some(t) => t,
                None => continue,
            };

            // Get activity distance (kilometers) 
            let distance_meters = match activity.distance {
                Some(d) => d,
                None => continue,
            };
            let distance_kilometers = distance_meters / 1000.0;

            // find the right hashmap for this athlete
            let athlete_kilometers = match team.as_str() {
                "bulls" => &mut bulls_athlete_kilometers,
                "sharks" => &mut sharks_athlete_kilometers,
                _ => continue,
            };
            // update athlete hashmap
            *athlete_kilometers.entry(athlete_name.clone()).or_insert(0.0) += distance_kilometers;

            let start_of_week = self.get_start_of_week_for_activity(activity);

            let week_key = start_of_week.format("%B %-d").to_string();

            // Update weekly kilometers for that week
            let weekly_kilometers = match team.as_str() {
                "bulls" => &mut bulls_weekly_kilometers,
                "sharks" => &mut sharks_weekly_kilometers,
                _ => continue,
            };
            *weekly_kilometers.entry(week_key).or_insert(0.0) += distance_kilometers;
        }

        let team_stats = TeamStats {
            bulls: TeamData {
                athlete_kilometers: bulls_athlete_kilometers,
                weekly_kilometers: bulls_weekly_kilometers,
            },
            sharks: TeamData {
                athlete_kilometers: sharks_athlete_kilometers,
                weekly_kilometers: sharks_weekly_kilometers,
            },
        };

        println!("[API] get_team_stats: Successfully calculated team stats");
        Ok(team_stats)
    }

    pub fn valid_activity(&self, activity: &BullSharkActivity) -> bool {
        if let Some(sport_type) = &activity.sport_type {
            if sport_type != "Run" {
                return false;
            }
        } else {
            return false; 
        }
        return true;
    }

    pub async fn build_athlete_team_map(&self) -> Result<HashMap<String, String>, ApiError> {
        let athletes = self.db.read_all_athletes().await?;
        let mut athlete_teams: HashMap<String, String> = HashMap::new();
        for athlete in athletes {
            athlete_teams.insert(athlete.name.clone(), athlete.team.clone());
        }

        Ok(athlete_teams)
    }

    // Hard coding team stat dates for now - club competition stats December 29th.
    fn get_team_stat_dates(&self) -> Result<(DateTime<Utc>, DateTime<Utc>), ApiError> {
        let start_date_naive = chrono::NaiveDate::from_ymd_opt(2025, 12, 1)
            .ok_or_else(|| ApiError::InternalConversionError("Invalid start date".to_string()))?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ApiError::InternalConversionError("Invalid start time".to_string()))?;
        let start_date_pacific = Los_Angeles.from_local_datetime(&start_date_naive).single()
            .ok_or_else(|| ApiError::InternalConversionError("Invalid start date time".to_string()))?;
        let start_date_utc = start_date_pacific.with_timezone(&Utc);

        let end_date_utc = Utc::now();

        Ok((start_date_utc, end_date_utc))
    }

    fn get_start_of_week_for_activity(&self, activity: BullSharkActivity) -> NaiveDateTime {
        let activity_date = activity.date;
        let activity_date_naive = activity_date.naive_local();
        let days_since_monday = activity_date_naive.weekday().num_days_from_monday();
        let start_of_week = activity_date_naive.date()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            - Duration::days(days_since_monday as i64);
        start_of_week
    }

}
