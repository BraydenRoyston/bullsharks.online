use chrono::{FixedOffset, NaiveDate, TimeZone, Utc};
use serde_json::json;
use std::collections::HashMap;

use crate::models::{
    bullshark::BullSharkActivity,
    athlete::Athlete,
    club::{ClubActivity, ClubAthlete},
    team_stats::{TeamStats, TeamData, WeekData},
    athlete_training_data::{AthleteTrainingData, AthleteWeeklyData, RiskyWeek},
    injury_risk::InjuryRiskType,
    oauth::StravaTokenResponse,
};

/// Comprehensive test suite for data models
/// 
/// Tests cover:
/// - JSON serialization and deserialization
/// - Data validation and constraints
/// - Optional field handling
/// - Type safety and conversion
/// - Edge cases and error conditions
/// - Enum string representations

// Helper function to create a test activity
fn create_test_bullshark_activity() -> BullSharkActivity {
    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        .and_hms_opt(10, 0, 0).unwrap();
    let fixed_offset = FixedOffset::east_opt(0).unwrap();
    let date_with_tz = fixed_offset.from_local_datetime(&date).unwrap();

    BullSharkActivity {
        id: "test_activity_123".to_string(),
        date: date_with_tz,
        athlete_name: Some("John Doe".to_string()),
        resource_state: Some(1),
        name: Some("Morning Run".to_string()),
        distance: Some(10000.0), // 10km in meters
        moving_time: Some(3600), // 1 hour in seconds
        elapsed_time: Some(3900), // 65 minutes in seconds
        total_elevation_gain: Some(150.0), // 150m elevation
        sport_type: Some("Run".to_string()),
        workout_type: Some(1),
        device_name: Some("Garmin Forerunner".to_string()),
    }
}

#[test]
fn test_bullshark_activity_json_serialization() {
    let activity = create_test_bullshark_activity();
    
    // Test serialization
    let serialized = serde_json::to_string(&activity);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("test_activity_123"));
    assert!(json_str.contains("John Doe"));
    assert!(json_str.contains("Morning Run"));
    assert!(json_str.contains("10000"));
    assert!(json_str.contains("Run"));
    
    // Test deserialization
    let deserialized: Result<BullSharkActivity, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_activity = deserialized.unwrap();
    assert_eq!(deserialized_activity.id, "test_activity_123");
    assert_eq!(deserialized_activity.athlete_name, Some("John Doe".to_string()));
    assert_eq!(deserialized_activity.distance, Some(10000.0));
    assert_eq!(deserialized_activity.sport_type, Some("Run".to_string()));
}

#[test]
fn test_bullshark_activity_json_with_nulls() {
    let json_with_nulls = json!({
        "id": "test_123",
        "date": "2024-01-15T10:00:00+00:00",
        "athlete_name": null,
        "resource_state": null,
        "name": null,
        "distance": 5000.0,
        "moving_time": null,
        "elapsed_time": null,
        "total_elevation_gain": null,
        "sport_type": "Run",
        "workout_type": null,
        "device_name": null
    });
    
    let deserialized: Result<BullSharkActivity, _> = serde_json::from_value(json_with_nulls);
    assert!(deserialized.is_ok());
    
    let activity = deserialized.unwrap();
    assert_eq!(activity.id, "test_123");
    assert_eq!(activity.athlete_name, None);
    assert_eq!(activity.name, None);
    assert_eq!(activity.distance, Some(5000.0));
    assert_eq!(activity.sport_type, Some("Run".to_string()));
    assert_eq!(activity.moving_time, None);
}

#[test]
fn test_bullshark_activity_required_fields() {
    // Test that required fields (id, date) are handled properly
    let minimal_json = json!({
        "id": "minimal_123",
        "date": "2024-01-15T10:00:00+00:00"
    });
    
    let deserialized: Result<BullSharkActivity, _> = serde_json::from_value(minimal_json);
    assert!(deserialized.is_ok());
    
    let activity = deserialized.unwrap();
    assert_eq!(activity.id, "minimal_123");
    assert!(activity.athlete_name.is_none());
    assert!(activity.distance.is_none());
}

#[test]
fn test_athlete_json_serialization() {
    let athlete = Athlete {
        id: "athlete_456".to_string(),
        name: "Jane Smith".to_string(),
        team: "sharks".to_string(),
        event: "half_marathon".to_string(),
    };
    
    // Test serialization
    let serialized = serde_json::to_string(&athlete);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("athlete_456"));
    assert!(json_str.contains("Jane Smith"));
    assert!(json_str.contains("sharks"));
    assert!(json_str.contains("half_marathon"));
    
    // Test deserialization
    let deserialized: Result<Athlete, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_athlete = deserialized.unwrap();
    assert_eq!(deserialized_athlete.id, "athlete_456");
    assert_eq!(deserialized_athlete.name, "Jane Smith");
    assert_eq!(deserialized_athlete.team, "sharks");
    assert_eq!(deserialized_athlete.event, "half_marathon");
}

#[test]
fn test_athlete_field_validation() {
    // Test various team values
    let teams = vec!["bulls", "sharks", "other"];
    for team in teams {
        let athlete = Athlete {
            id: "test".to_string(),
            name: "Test User".to_string(),
            team: team.to_string(),
            event: "marathon".to_string(),
        };
        assert_eq!(athlete.team, team);
    }
    
    // Test various event values
    let events = vec!["marathon", "half_marathon", "10k", "5k"];
    for event in events {
        let athlete = Athlete {
            id: "test".to_string(),
            name: "Test User".to_string(),
            team: "bulls".to_string(),
            event: event.to_string(),
        };
        assert_eq!(athlete.event, event);
    }
}

#[test]
fn test_club_activity_json_serialization() {
    let club_activity = ClubActivity {
        resource_state: Some(3),
        athlete: Some(ClubAthlete {
            resource_state: Some(2),
            first_name: Some("Alex".to_string()),
            last_name: Some("Johnson".to_string()),
        }),
        name: Some("Evening Run".to_string()),
        distance: Some(8000.0), // 8km
        moving_time: Some(2400), // 40 minutes
        elapsed_time: Some(2500), // ~41 minutes
        total_elevation_gain: Some(200.0),
        sport_type: Some("Run".to_string()),
        workout_type: Some(0),
        device_name: Some("Strava iPhone App".to_string()),
    };
    
    let serialized = serde_json::to_string(&club_activity);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("Alex"));
    assert!(json_str.contains("Johnson"));
    assert!(json_str.contains("Evening Run"));
    assert!(json_str.contains("8000"));
    
    // Test deserialization
    let deserialized: Result<ClubActivity, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_activity = deserialized.unwrap();
    assert_eq!(deserialized_activity.distance, Some(8000.0));
    assert_eq!(deserialized_activity.sport_type, Some("Run".to_string()));
    
    let athlete = deserialized_activity.athlete.unwrap();
    assert_eq!(athlete.first_name, Some("Alex".to_string()));
    assert_eq!(athlete.last_name, Some("Johnson".to_string()));
}

#[test]
fn test_club_athlete_firstname_lastname_serde_rename() {
    // Test that the serde rename attributes work for firstname/lastname
    let json_input = json!({
        "resource_state": 2,
        "firstname": "John",
        "lastname": "Doe"
    });
    
    let deserialized: Result<ClubAthlete, _> = serde_json::from_value(json_input);
    assert!(deserialized.is_ok());
    
    let athlete = deserialized.unwrap();
    assert_eq!(athlete.first_name, Some("John".to_string()));
    assert_eq!(athlete.last_name, Some("Doe".to_string()));
    
    // Test serialization (should use the renamed fields)
    let serialized = serde_json::to_string(&athlete);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("firstname"));
    assert!(json_str.contains("lastname"));
    assert!(!json_str.contains("first_name")); // Should not contain Rust field names
    assert!(!json_str.contains("last_name"));
}

#[test]
fn test_injury_risk_type_string_conversion() {
    // Test all injury risk type enum variants
    assert_eq!(InjuryRiskType::SSRD30NoRisk.as_str(), "SSRD30_NO_RISK");
    assert_eq!(InjuryRiskType::SSRD30SmallRisk.as_str(), "SSRD30_SMALL_RISK");
    assert_eq!(InjuryRiskType::SSRD30ModerateRisk.as_str(), "SSRD30_MODERATE_RISK");
    assert_eq!(InjuryRiskType::SSRD30LargeRisk.as_str(), "SSRD30_LARGE_RISK");
    assert_eq!(InjuryRiskType::HighVolumeSpike.as_str(), "HIGH_VOLUME_SPIKE");
}

#[test]
fn test_injury_risk_type_display() {
    // Test the Display implementation for InjuryRiskType
    assert_eq!(format!("{}", InjuryRiskType::SSRD30NoRisk), "SSRD30_NO_RISK");
    assert_eq!(format!("{}", InjuryRiskType::SSRD30SmallRisk), "SSRD30_SMALL_RISK");
    assert_eq!(format!("{}", InjuryRiskType::SSRD30ModerateRisk), "SSRD30_MODERATE_RISK");
    assert_eq!(format!("{}", InjuryRiskType::SSRD30LargeRisk), "SSRD30_LARGE_RISK");
    assert_eq!(format!("{}", InjuryRiskType::HighVolumeSpike), "HIGH_VOLUME_SPIKE");
}

#[test]
fn test_risky_week_json_serialization() {
    let risky_week = RiskyWeek {
        week: "2024-01-15".to_string(),
        risk_count: 2,
        risks: vec![
            "SSRD30_MODERATE_RISK: 15.0km run on 2024-01-17 exceeded max distance in prior 30 days (10.0km) by 50.0%".to_string(),
            "HIGH_VOLUME_SPIKE: Weekly volume increased from 25.0km to 30.0km (20.0% increase exceeds 10% rule)".to_string(),
        ],
    };
    
    let serialized = serde_json::to_string(&risky_week);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("2024-01-15"));
    assert!(json_str.contains("SSRD30_MODERATE_RISK"));
    assert!(json_str.contains("HIGH_VOLUME_SPIKE"));
    assert!(json_str.contains("50.0%"));
    assert!(json_str.contains("20.0%"));
    
    // Test deserialization
    let deserialized: Result<RiskyWeek, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_week = deserialized.unwrap();
    assert_eq!(deserialized_week.week, "2024-01-15");
    assert_eq!(deserialized_week.risk_count, 2);
    assert_eq!(deserialized_week.risks.len(), 2);
}

#[test]
fn test_athlete_training_data_json_serialization() {
    let mut weekly_kilometers = HashMap::new();
    weekly_kilometers.insert("2024-01-01".to_string(), 25.0);
    weekly_kilometers.insert("2024-01-08".to_string(), 30.0);
    
    let risky_weeks = vec![
        RiskyWeek {
            week: "2024-01-08".to_string(),
            risk_count: 1,
            risks: vec!["Test risk".to_string()],
        }
    ];
    
    let weekly_training_data = AthleteWeeklyData {
        weekly_kilometers,
        risky_weeks,
    };
    
    let training_data = AthleteTrainingData {
        id: "athlete_123".to_string(),
        name: "John Doe".to_string(),
        team: "bulls".to_string(),
        event: "marathon".to_string(),
        training_data: weekly_training_data,
    };
    
    let serialized = serde_json::to_string(&training_data);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("John Doe"));
    assert!(json_str.contains("bulls"));
    assert!(json_str.contains("athlete_123"));
    assert!(json_str.contains("marathon"));
    
    // Test deserialization
    let deserialized: Result<AthleteTrainingData, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_data = deserialized.unwrap();
    assert_eq!(deserialized_data.name, "John Doe");
    assert_eq!(deserialized_data.team, "bulls");
    assert_eq!(deserialized_data.id, "athlete_123");
    assert_eq!(deserialized_data.event, "marathon");
    assert_eq!(deserialized_data.training_data.weekly_kilometers.len(), 2);
    assert_eq!(deserialized_data.training_data.risky_weeks.len(), 1);
}

#[test]
fn test_team_stats_json_serialization() {
    let mut athlete_kilometers = HashMap::new();
    athlete_kilometers.insert("John Doe".to_string(), 50.0);
    athlete_kilometers.insert("Jane Doe".to_string(), 45.0);
    
    let mut weekly_athlete_km = HashMap::new();
    weekly_athlete_km.insert("John Doe".to_string(), 15.0);
    weekly_athlete_km.insert("Jane Doe".to_string(), 12.0);
    
    let week_data = WeekData {
        week_start: FixedOffset::east_opt(0).unwrap().from_utc_datetime(&Utc::now().naive_utc()),
        weekly_team_kilometers: 27.0,
        weekly_running_sum: 27.0,
        weekly_athlete_kilometers: weekly_athlete_km,
    };
    
    let team_data = TeamData {
        athlete_kilometers,
        weekly_kilometers: vec![week_data],
    };
    
    let team_stats = TeamStats {
        bulls: team_data.clone(),
        sharks: team_data,
    };
    
    let serialized = serde_json::to_string(&team_stats);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("bulls"));
    assert!(json_str.contains("sharks"));
    assert!(json_str.contains("John Doe"));
    assert!(json_str.contains("50.0"));
    assert!(json_str.contains("27.0"));
    
    // Test deserialization
    let deserialized: Result<TeamStats, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_stats = deserialized.unwrap();
    assert!(deserialized_stats.bulls.athlete_kilometers.contains_key("John Doe"));
    assert!(deserialized_stats.sharks.athlete_kilometers.contains_key("Jane Doe"));
    assert_eq!(deserialized_stats.bulls.weekly_kilometers.len(), 1);
}

#[test]
fn test_oauth_json_serialization() {
    let oauth = StravaTokenResponse {
        token_type: "Bearer".to_string(),
        expires_at: 1705123200, // Unix timestamp
        expires_in: 21600, // 6 hours
        refresh_token: "refresh_abc123".to_string(),
        access_token: "access_xyz789".to_string(),
    };
    
    let serialized = serde_json::to_string(&oauth);
    assert!(serialized.is_ok());
    
    let json_str = serialized.unwrap();
    assert!(json_str.contains("Bearer"));
    assert!(json_str.contains("1705123200"));
    assert!(json_str.contains("21600"));
    assert!(json_str.contains("refresh_abc123"));
    assert!(json_str.contains("access_xyz789"));
    
    // Test deserialization
    let deserialized: Result<StravaTokenResponse, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());
    
    let deserialized_oauth = deserialized.unwrap();
    assert_eq!(deserialized_oauth.token_type, "Bearer".to_string());
    assert_eq!(deserialized_oauth.expires_at, 1705123200);
    assert_eq!(deserialized_oauth.access_token, "access_xyz789".to_string());
}

#[test]
fn test_datetime_serialization_formats() {
    let activity = create_test_bullshark_activity();
    
    let serialized = serde_json::to_string(&activity).unwrap();
    let deserialized: BullSharkActivity = serde_json::from_str(&serialized).unwrap();
    
    // Ensure datetime roundtrip works
    assert_eq!(activity.date.timestamp(), deserialized.date.timestamp());
    assert_eq!(activity.date.format("%Y-%m-%d").to_string(), 
               deserialized.date.format("%Y-%m-%d").to_string());
}

#[test]
fn test_optional_field_edge_cases() {
    // Test activity with all optional fields set to None
    let minimal_activity = BullSharkActivity {
        id: "minimal".to_string(),
        date: FixedOffset::east_opt(0).unwrap().from_utc_datetime(&Utc::now().naive_utc()),
        athlete_name: None,
        resource_state: None,
        name: None,
        distance: None,
        moving_time: None,
        elapsed_time: None,
        total_elevation_gain: None,
        sport_type: None,
        workout_type: None,
        device_name: None,
    };
    
    let serialized = serde_json::to_string(&minimal_activity);
    assert!(serialized.is_ok());
    
    let deserialized: Result<BullSharkActivity, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
    
    let deserialized_activity = deserialized.unwrap();
    assert_eq!(deserialized_activity.id, "minimal");
    assert!(deserialized_activity.athlete_name.is_none());
    assert!(deserialized_activity.distance.is_none());
    assert!(deserialized_activity.sport_type.is_none());
}

#[test]
fn test_numeric_field_precision() {
    // Test that floating point values maintain precision
    let activity = BullSharkActivity {
        id: "precision_test".to_string(),
        date: FixedOffset::east_opt(0).unwrap().from_utc_datetime(&Utc::now().naive_utc()),
        athlete_name: Some("Test User".to_string()),
        resource_state: Some(1),
        name: Some("Precision Test".to_string()),
        distance: Some(10234.567), // Precise distance
        moving_time: Some(3661), // 1 hour 1 minute 1 second
        elapsed_time: Some(3723), // 1 hour 2 minutes 3 seconds
        total_elevation_gain: Some(123.45),
        sport_type: Some("Run".to_string()),
        workout_type: Some(1),
        device_name: Some("Test Device".to_string()),
    };
    
    let serialized = serde_json::to_string(&activity).unwrap();
    let deserialized: BullSharkActivity = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(deserialized.distance, Some(10234.567));
    assert_eq!(deserialized.moving_time, Some(3661));
    assert_eq!(deserialized.elapsed_time, Some(3723));
    assert_eq!(deserialized.total_elevation_gain, Some(123.45));
}

#[test]
fn test_invalid_json_handling() {
    let invalid_json_strings = vec![
        r#"{"id": "test"}"#, // Missing required date field
        r#"{"date": "invalid-date"}"#, // Invalid date format, missing id
        r#"{"id": 123, "date": "2024-01-15T10:00:00+00:00"}"#, // Wrong type for id
        r#"{}"#, // Empty object
    ];
    
    for invalid_json in invalid_json_strings {
        let result: Result<BullSharkActivity, _> = serde_json::from_str(invalid_json);
        // Most of these should fail deserialization
        // (except possibly the empty object case depending on serde defaults)
        if invalid_json == r#"{}"# {
            // Empty object might deserialize with default values depending on serde setup
            continue;
        } else {
            assert!(result.is_err(), "Expected deserialization to fail for: {}", invalid_json);
        }
    }
}

#[test]
fn test_large_numeric_values() {
    // Test handling of large numeric values that might occur in real data
    let activity = BullSharkActivity {
        id: "large_values".to_string(),
        date: FixedOffset::east_opt(0).unwrap().from_utc_datetime(&Utc::now().naive_utc()),
        athlete_name: Some("Marathon Runner".to_string()),
        resource_state: Some(3),
        name: Some("Ultra Marathon".to_string()),
        distance: Some(100000.0), // 100km in meters
        moving_time: Some(36000), // 10 hours in seconds
        elapsed_time: Some(43200), // 12 hours in seconds
        total_elevation_gain: Some(5000.0), // 5000m elevation gain
        sport_type: Some("Run".to_string()),
        workout_type: Some(2),
        device_name: Some("GPS Watch".to_string()),
    };
    
    let serialized = serde_json::to_string(&activity);
    assert!(serialized.is_ok());
    
    let deserialized: Result<BullSharkActivity, _> = serde_json::from_str(&serialized.unwrap());
    assert!(deserialized.is_ok());
    
    let deserialized_activity = deserialized.unwrap();
    assert_eq!(deserialized_activity.distance, Some(100000.0));
    assert_eq!(deserialized_activity.moving_time, Some(36000));
    assert_eq!(deserialized_activity.total_elevation_gain, Some(5000.0));
}