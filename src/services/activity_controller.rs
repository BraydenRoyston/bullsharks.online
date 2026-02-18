use std::{collections::HashMap, sync::Arc};

use crate::{error::ApiError, models::{athlete::Athlete, athlete_training_data::{AllAthletesTrainingData, AthleteTrainingData, AthleteWeeklyData, RiskyWeek}, bullshark::BullSharkActivity, club::ClubActivity, injury_risk::InjuryRiskType, team_stats::{TeamData, TeamStats, WeekData}}, services::{database::Database, strava_client::StravaClient}};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDateTime, Offset, TimeZone, Utc};
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
        let mut bulls_week_data: HashMap<NaiveDateTime, WeekData> = HashMap::new();
        let mut sharks_athlete_kilometers: HashMap<String, f64> = HashMap::new();
        let mut sharks_week_data: HashMap<NaiveDateTime, WeekData> = HashMap::new();

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

            let start_of_week = self.get_start_of_week_for_activity(&activity);

            // Update weekly kilometers for that week
            let weekly_kilometers = match team.as_str() {
                "bulls" => &mut bulls_week_data,
                "sharks" => &mut sharks_week_data,
                _ => continue,
            };

            let pacific_dt = Los_Angeles.from_local_datetime(&start_of_week).single()
                .ok_or_else(|| ApiError::InternalConversionError(format!("Invalid datetime conversion for week start: {}", start_of_week)))?;
            let week_start = pacific_dt.with_timezone(&pacific_dt.offset().fix());

            let week_data = weekly_kilometers.entry(start_of_week).or_insert(WeekData { 
                week_start: week_start, 
                weekly_team_kilometers: 0.0, 
                weekly_running_sum: 0.0, 
                weekly_athlete_kilometers: HashMap::new() 
            });

            week_data.weekly_team_kilometers += distance_kilometers;
            *week_data.weekly_athlete_kilometers.entry(athlete_name.to_string()).or_insert(0.0) += distance_kilometers;
        }

        // Convert to vec, compute running sums, sort entries, etc. 
        let bulls_weekly_vec = self.convert_weekly_map_to_vec(bulls_week_data)?;
        let sharks_weekly_vec = self.convert_weekly_map_to_vec(sharks_week_data)?;

        let team_stats = TeamStats {
            bulls: TeamData {
                athlete_kilometers: bulls_athlete_kilometers,
                weekly_kilometers: bulls_weekly_vec,
            },
            sharks: TeamData {
                athlete_kilometers: sharks_athlete_kilometers,
                weekly_kilometers: sharks_weekly_vec,
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
        let start_date_naive = chrono::NaiveDate::from_ymd_opt(2025, 12, 29)
            .ok_or_else(|| ApiError::InternalConversionError("Invalid start date".to_string()))?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ApiError::InternalConversionError("Invalid start time".to_string()))?;
        let start_date_pacific = Los_Angeles.from_local_datetime(&start_date_naive).single()
            .ok_or_else(|| ApiError::InternalConversionError("Invalid start date time".to_string()))?;
        let start_date_utc = start_date_pacific.with_timezone(&Utc);

        let end_date_utc = Utc::now();

        Ok((start_date_utc, end_date_utc))
    }

    fn get_start_of_week_for_activity(&self, activity: &BullSharkActivity) -> NaiveDateTime {
        let activity_date = activity.date;
        let activity_date_naive = activity_date.naive_local();
        let days_since_monday = activity_date_naive.weekday().num_days_from_monday();
        let start_of_week = activity_date_naive.date()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            - Duration::days(days_since_monday as i64);
        start_of_week
    }

    fn convert_weekly_map_to_vec(&self, weekly_map: HashMap<NaiveDateTime, WeekData>) -> Result<Vec<WeekData>, ApiError> {
        let mut running_sum: f64 = 0.0;
        let mut weekly_vec: Vec<(NaiveDateTime, WeekData)> = weekly_map
            .into_iter()
            .collect::<Vec<(NaiveDateTime, WeekData)>>();
        weekly_vec.sort_by(|a, b| a.0.cmp(&b.0));

        let week_data_vec = weekly_vec
            .into_iter()
            .map(|(_naive_dt, mut week_data)| {
                running_sum += week_data.weekly_team_kilometers;
                week_data.weekly_running_sum += running_sum;
                Ok(week_data)
            })
            .collect::<Result<Vec<WeekData>, ApiError>>()?;

        Ok(week_data_vec)
    }

    pub async fn read_all_athletes(&self) -> Result<Vec<Athlete>, ApiError> {
        let result = self.db.read_all_athletes().await?;
        Ok(result)
    }

    fn analyze_injury_risks(&self, athlete_name: String, weekly_kilometers: &HashMap<String, f64>, activities: &Vec<BullSharkActivity>) -> Vec<RiskyWeek> {
        let mut risky_weeks: HashMap<String, RiskyWeek> = HashMap::new();

        // SSRD30 Analysis: Analyze each activity against the longest run in the preceding 30 days
        self.analyze_ssrd30_risks(&athlete_name, activities, &mut risky_weeks);

        // 10% Rule Analysis: Check weekly volume increases
        self.analyze_ten_percent_rule(&athlete_name, weekly_kilometers, &mut risky_weeks);

        // Return only weeks with risks detected
        risky_weeks.into_values()
            .filter(|entry| entry.risk_count > 0)
            .collect()
    }

    /// SSRD30 Analysis: Session Specific Running Distance in last 30 days
    /// For each activity, compare its distance against the longest run in the 30 days preceding it
    fn analyze_ssrd30_risks(&self, athlete_name: &str, activities: &Vec<BullSharkActivity>, risky_weeks: &mut HashMap<String, RiskyWeek>) {
        // Filter and sort activities chronologically for this athlete
        let mut athlete_activities: Vec<&BullSharkActivity> = activities
            .iter()
            .filter(|activity| {
                activity.athlete_name.as_deref() == Some(athlete_name) 
                && activity.sport_type.as_deref() == Some("Run")
                && activity.distance.is_some()
            })
            .collect();

        // Sort by date (chronologically)
        athlete_activities.sort_by_key(|activity| activity.date);

        // For each activity, find max distance in preceding 30 days
        for (i, current_activity) in athlete_activities.iter().enumerate() {
            let current_distance = current_activity.distance.unwrap_or(0.0);
            let current_date = current_activity.date;
            
            // Find maximum distance in the 30 days before this activity
            let thirty_days_ago = current_date - Duration::days(30);
            let max_distance_30d = athlete_activities
                .iter()
                .take(i) // Only consider activities before current one
                .filter(|prev_activity| prev_activity.date >= thirty_days_ago)
                .filter_map(|activity| activity.distance)
                .fold(0.0_f64, |max, distance| max.max(distance));

            // Skip if no prior activities in 30 days (no baseline to compare against)
            if max_distance_30d == 0.0 {
                continue;
            }

            // Calculate risk level based on percentage increase
            let growth_ratio = current_distance / max_distance_30d;
            let growth_percentage = growth_ratio - 1.0;

            let risk_type = match growth_percentage {
                x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk, // Account for floating point precision
                x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
                x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
                _ => InjuryRiskType::SSRD30LargeRisk,
            };

            // Add risk if detected
            if risk_type != InjuryRiskType::SSRD30NoRisk {
                let week_start = self.get_start_of_week_for_activity(current_activity);
                let week_string = week_start.format("%Y-%m-%d").to_string();

                let risky_week = risky_weeks.entry(week_string.clone()).or_insert_with(|| RiskyWeek {
                    week: week_string.clone(),
                    risk_count: 0,
                    risks: Vec::new(),
                });

                risky_week.risk_count += 1;
                let risk_message = format!(
                    "{}: {:.1}km run on {} exceeded max distance in prior 30 days ({:.1}km) by {:.1}%",
                    risk_type,
                    current_distance / 1000.0,
                    current_activity.date.format("%Y-%m-%d"),
                    max_distance_30d / 1000.0,
                    growth_percentage * 100.0
                );
                risky_week.risks.push(risk_message);
            }
        }
    }

    /// 10% Rule Analysis: Check for week-over-week volume increases > 10%
    fn analyze_ten_percent_rule(&self, _athlete_name: &str, weekly_kilometers: &HashMap<String, f64>, risky_weeks: &mut HashMap<String, RiskyWeek>) {
        // Sort weeks chronologically using proper date parsing
        let mut weeks: Vec<(&String, &f64)> = weekly_kilometers.iter().collect();
        weeks.sort_by_key(|(week_str, _)| {
            chrono::NaiveDate::parse_from_str(week_str, "%Y-%m-%d").unwrap_or_default()
        });

        // Configuration constants
        const MIN_WEEKLY_KM_THRESHOLD: f64 = 20.0;
        const SPIKE_THRESHOLD_MULTIPLIER: f64 = 1.10; // 10% increase

        // Check each week against the previous week
        for window in weeks.windows(2) {
            let (_prev_week, prev_km) = window[0];
            let (current_week, current_km) = window[1];

            let spike_threshold = prev_km * SPIKE_THRESHOLD_MULTIPLIER;
            
            if *current_km > spike_threshold && *current_km > MIN_WEEKLY_KM_THRESHOLD {
                let risky_week = risky_weeks.entry(current_week.clone()).or_insert_with(|| RiskyWeek {
                    week: current_week.clone(),
                    risk_count: 0,
                    risks: Vec::new(),
                });

                risky_week.risk_count += 1;
                let increase_percentage = (current_km / prev_km - 1.0) * 100.0;
                let risk_message = format!(
                    "{}: Weekly volume increased from {:.1}km to {:.1}km ({:.1}% increase exceeds 10% rule)",
                    InjuryRiskType::HighVolumeSpike,
                    prev_km,
                    current_km,
                    increase_percentage
                );
                risky_week.risks.push(risk_message);
            }
        }
    }

    pub async fn get_all_athletes_training_data(&self) -> Result<AllAthletesTrainingData, ApiError> {
        // Fetch all athletes from database
        let athletes = self.db.read_all_athletes().await?;

        // Create lookup map: athlete_name -> Athlete
        let mut athlete_map: HashMap<String, Athlete> = HashMap::new();
        for athlete in athletes {
            athlete_map.insert(athlete.name.clone(), athlete);
        }

        // Fetch all activities from database (all-time data)
        let activities = self.db.get_all_activities().await?;

        // Create nested map: athlete_name -> (week_date_string -> kilometers)
        let mut athlete_weekly_km: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for activity in &activities {
            // Filter for valid activities (sport_type == "Run")
            if !self.valid_activity(&activity) {
                continue;
            }

            // Get athlete name
            let athlete_name = match &activity.athlete_name {
                Some(name) => name,
                None => continue,
            };

            // Skip if athlete not in our database
            if !athlete_map.contains_key(athlete_name) {
                continue;
            }

            // Get activity distance in kilometers
            let distance_meters = match activity.distance {
                Some(d) => d,
                None => continue,
            };
            let distance_kilometers = distance_meters / 1000.0;

            // Calculate week start (Monday)
            let start_of_week = self.get_start_of_week_for_activity(&activity);

            // Format as ISO date string (YYYY-MM-DD)
            let week_date_string = start_of_week.format("%Y-%m-%d").to_string();

            // Add to nested map
            let weekly_map = athlete_weekly_km
                .entry(athlete_name.clone())
                .or_insert_with(HashMap::new);

            *weekly_map.entry(week_date_string).or_insert(0.0) += distance_kilometers;
        }

        // Build response for all athletes
        let mut result: Vec<AthleteTrainingData> = Vec::new();

        for (athlete_name, athlete) in athlete_map {
            let weekly_kilometers = athlete_weekly_km
                .get(&athlete_name)
                .cloned()
                .unwrap_or_else(HashMap::new);

            // Analyze injury risks for this athlete
            let risky_weeks = self.analyze_injury_risks(athlete_name, &weekly_kilometers, &activities);

            result.push(AthleteTrainingData {
                id: athlete.id,
                name: athlete.name,
                team: athlete.team,
                event: athlete.event,
                training_data: AthleteWeeklyData {
                    weekly_kilometers,
                    risky_weeks,
                },
            });
        }

        // Sort by athlete name for consistent ordering
        result.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(AllAthletesTrainingData {
            athletes: result,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, NaiveDate};
    use std::collections::HashMap;

    // Helper function to create a test activity
    fn create_test_activity(
        date_str: &str,
        athlete_name: &str,
        distance_km: f64,
    ) -> BullSharkActivity {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let fixed_offset = FixedOffset::east_opt(0).unwrap();
        let date_with_tz = fixed_offset.from_local_datetime(&date).unwrap();

        BullSharkActivity {
            id: format!("test_{}", date_str),
            date: date_with_tz,
            athlete_name: Some(athlete_name.to_string()),
            resource_state: Some(1),
            name: Some(format!("Test Run {}", date_str)),
            distance: Some(distance_km * 1000.0), // Convert to meters
            moving_time: Some(3600),
            elapsed_time: Some(3600),
            total_elevation_gain: Some(0.0),
            sport_type: Some("Run".to_string()),
            workout_type: Some(1),
            device_name: Some("Test Device".to_string()),
        }
    }

    // Helper to test SSRD30 logic without full ActivityController setup
    fn analyze_ssrd30_test(athlete_name: &str, activities: &[BullSharkActivity]) -> HashMap<String, RiskyWeek> {
        // Filter and sort activities chronologically for this athlete
        let mut athlete_activities: Vec<&BullSharkActivity> = activities
            .iter()
            .filter(|activity| {
                activity.athlete_name.as_deref() == Some(athlete_name) 
                && activity.sport_type.as_deref() == Some("Run")
                && activity.distance.is_some()
            })
            .collect();

        athlete_activities.sort_by_key(|activity| activity.date);

        let mut risky_weeks: HashMap<String, RiskyWeek> = HashMap::new();

        // For each activity, find max distance in preceding 30 days
        for (i, current_activity) in athlete_activities.iter().enumerate() {
            let current_distance = current_activity.distance.unwrap_or(0.0);
            let current_date = current_activity.date;
            
            let thirty_days_ago = current_date - Duration::days(30);
            let max_distance_30d = athlete_activities
                .iter()
                .take(i)
                .filter(|prev_activity| prev_activity.date >= thirty_days_ago)
                .filter_map(|activity| activity.distance)
                .fold(0.0_f64, |max, distance| max.max(distance));

            if max_distance_30d == 0.0 {
                continue;
            }

            let growth_ratio = current_distance / max_distance_30d;
            let growth_percentage = growth_ratio - 1.0;

            let risk_type = match growth_percentage {
                x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk, // Account for floating point precision
                x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
                x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
                _ => InjuryRiskType::SSRD30LargeRisk,
            };

            if risk_type != InjuryRiskType::SSRD30NoRisk {
                // Calculate week start for the activity
                let activity_date_naive = current_activity.date.naive_local();
                let days_since_monday = activity_date_naive.weekday().num_days_from_monday();
                let start_of_week = activity_date_naive.date()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    - Duration::days(days_since_monday as i64);
                let week_string = start_of_week.format("%Y-%m-%d").to_string();

                let risky_week = risky_weeks.entry(week_string.clone()).or_insert_with(|| RiskyWeek {
                    week: week_string.clone(),
                    risk_count: 0,
                    risks: Vec::new(),
                });

                risky_week.risk_count += 1;
                let risk_message = format!(
                    "{}: {:.1}km run on {} exceeded max distance in prior 30 days ({:.1}km) by {:.1}%",
                    risk_type,
                    current_distance / 1000.0,
                    current_activity.date.format("%Y-%m-%d"),
                    max_distance_30d / 1000.0,
                    growth_percentage * 100.0
                );
                risky_week.risks.push(risk_message);
            }
        }

        risky_weeks
    }

    #[test]
    fn test_ssrd30_no_risk_scenario() {
        let activities = vec![
            create_test_activity("2024-01-01", "John Doe", 5.0),  // 5km baseline
            create_test_activity("2024-01-15", "John Doe", 5.5),  // 5.5km - 10% increase, should be no risk
        ];

        let risky_weeks = analyze_ssrd30_test("John Doe", &activities);
        
        // Should have no risky weeks since 5.5km is only 10% increase from 5km (threshold)
        assert_eq!(risky_weeks.len(), 0);
    }

    #[test]
    fn test_ssrd30_small_risk_scenario() {
        let activities = vec![
            create_test_activity("2024-01-01", "John Doe", 10.0), // 10km baseline
            create_test_activity("2024-01-15", "John Doe", 12.5), // 12.5km - 25% increase, should be small risk
        ];

        let risky_weeks = analyze_ssrd30_test("John Doe", &activities);
        
        assert_eq!(risky_weeks.len(), 1);
        let risky_week = risky_weeks.values().next().unwrap();
        assert_eq!(risky_week.risk_count, 1);
        assert!(risky_week.risks[0].contains("SSRD30_SMALL_RISK"));
        assert!(risky_week.risks[0].contains("10.0km")); // Should reference baseline
        assert!(risky_week.risks[0].contains("25.0%")); // Should show percentage
    }

    #[test]
    fn test_ssrd30_moderate_risk_scenario() {
        let activities = vec![
            create_test_activity("2024-01-01", "John Doe", 10.0), // 10km baseline
            create_test_activity("2024-01-15", "John Doe", 18.0), // 18km - 80% increase, should be moderate risk
        ];

        let risky_weeks = analyze_ssrd30_test("John Doe", &activities);
        
        assert_eq!(risky_weeks.len(), 1);
        let risky_week = risky_weeks.values().next().unwrap();
        assert!(risky_week.risks[0].contains("SSRD30_MODERATE_RISK"));
        assert!(risky_week.risks[0].contains("80.0%"));
    }

    #[test]
    fn test_ssrd30_large_risk_scenario() {
        let activities = vec![
            create_test_activity("2024-01-01", "John Doe", 10.0), // 10km baseline
            create_test_activity("2024-01-15", "John Doe", 25.0), // 25km - 150% increase, should be large risk
        ];

        let risky_weeks = analyze_ssrd30_test("John Doe", &activities);
        
        assert_eq!(risky_weeks.len(), 1);
        let risky_week = risky_weeks.values().next().unwrap();
        assert!(risky_week.risks[0].contains("SSRD30_LARGE_RISK"));
        assert!(risky_week.risks[0].contains("150.0%"));
    }

    #[test]
    fn test_ssrd30_thirty_day_window() {
        let activities = vec![
            create_test_activity("2024-01-01", "John Doe", 20.0), // 20km long run
            create_test_activity("2024-01-16", "John Doe", 10.0), // 10km medium run
            create_test_activity("2024-02-14", "John Doe", 15.0), // 15km run - exactly 29 days later
            // This should compare against the 10km run (Jan 16), not the 20km run (Jan 1)
            // because the 20km run is > 30 days ago (45 days)
        ];

        let risky_weeks = analyze_ssrd30_test("John Doe", &activities);
        
        // Should have risk because 15km vs 10km baseline (50% increase)
        assert_eq!(risky_weeks.len(), 1);
        let risky_week = risky_weeks.values().next().unwrap();
        assert!(risky_week.risks[0].contains("SSRD30_MODERATE_RISK"));
        assert!(risky_week.risks[0].contains("10.0km")); // Should reference 10km baseline, not 20km
        assert!(risky_week.risks[0].contains("50.0%")); // Should show 50% increase
    }

    #[test]
    fn test_ten_percent_rule_calculation() {
        let prev_week_km = 20.0;
        let current_week_km = 25.0;
        let spike_threshold = prev_week_km * 1.10; // 22.0km
        
        // Should trigger risk since 25.0 > 22.0 and 25.0 > 20.0 (min threshold)
        assert!(current_week_km > spike_threshold);
        assert!(current_week_km > 20.0);
        
        // Calculate increase percentage
        let increase_percentage = (current_week_km / prev_week_km - 1.0) * 100.0;
        assert_eq!(increase_percentage, 25.0); // 25% increase
    }

    #[test]
    fn test_risk_type_classification() {
        // Test the SSRD30 risk classification logic
        assert_eq!(
            match 0.05 { // 5% increase
                x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk,
                x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
                x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
                _ => InjuryRiskType::SSRD30LargeRisk,
            },
            InjuryRiskType::SSRD30NoRisk
        );

        assert_eq!(
            match 0.25 { // 25% increase
                x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk,
                x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
                x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
                _ => InjuryRiskType::SSRD30LargeRisk,
            },
            InjuryRiskType::SSRD30SmallRisk
        );

        assert_eq!(
            match 0.8 { // 80% increase
                x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk,
                x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
                x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
                _ => InjuryRiskType::SSRD30LargeRisk,
            },
            InjuryRiskType::SSRD30ModerateRisk
        );

        assert_eq!(
            match 1.5 { // 150% increase
                x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk,
                x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
                x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
                _ => InjuryRiskType::SSRD30LargeRisk,
            },
            InjuryRiskType::SSRD30LargeRisk
        );
    }

    #[test]
    fn test_risk_type_string_conversion() {
        assert_eq!(InjuryRiskType::SSRD30NoRisk.as_str(), "SSRD30_NO_RISK");
        assert_eq!(InjuryRiskType::SSRD30SmallRisk.as_str(), "SSRD30_SMALL_RISK");
        assert_eq!(InjuryRiskType::SSRD30ModerateRisk.as_str(), "SSRD30_MODERATE_RISK");
        assert_eq!(InjuryRiskType::SSRD30LargeRisk.as_str(), "SSRD30_LARGE_RISK");
        assert_eq!(InjuryRiskType::HighVolumeSpike.as_str(), "HIGH_VOLUME_SPIKE");
    }
}
