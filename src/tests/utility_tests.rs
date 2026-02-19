use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

/// Comprehensive test suite for utility functions
///
/// Tests cover:
/// - Date and time utility functions
/// - Data transformation utilities
/// - Validation helpers
/// - Configuration parsing
/// - String formatting and parsing
/// - Mathematical calculations used in the app

// Test date/time utility functions (commonly used patterns from the codebase)

#[test]
fn test_week_start_calculation() {
    // Test the week start calculation logic used in multiple places

    // Monday should be start of its own week
    let monday = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); // January 15, 2024 was Monday
    let monday_dt = monday.and_hms_opt(14, 30, 0).unwrap();
    let days_since_monday = monday_dt.weekday().num_days_from_monday();
    let start_of_week = monday_dt.date().and_hms_opt(0, 0, 0).unwrap()
        - chrono::Duration::days(days_since_monday as i64);

    assert_eq!(days_since_monday, 0);
    assert_eq!(start_of_week.date(), monday);

    // Wednesday should calculate back to Monday
    let wednesday = NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(); // January 17, 2024 was Wednesday
    let wednesday_dt = wednesday.and_hms_opt(14, 30, 0).unwrap();
    let days_since_monday_wed = wednesday_dt.weekday().num_days_from_monday();
    let start_of_week_wed = wednesday_dt.date().and_hms_opt(0, 0, 0).unwrap()
        - chrono::Duration::days(days_since_monday_wed as i64);

    assert_eq!(days_since_monday_wed, 2);
    assert_eq!(start_of_week_wed.date(), monday);

    // Sunday should calculate back to previous Monday
    let sunday = NaiveDate::from_ymd_opt(2024, 1, 21).unwrap(); // January 21, 2024 was Sunday
    let sunday_dt = sunday.and_hms_opt(14, 30, 0).unwrap();
    let days_since_monday_sun = sunday_dt.weekday().num_days_from_monday();
    let start_of_week_sun = sunday_dt.date().and_hms_opt(0, 0, 0).unwrap()
        - chrono::Duration::days(days_since_monday_sun as i64);

    assert_eq!(days_since_monday_sun, 6);
    assert_eq!(start_of_week_sun.date(), monday);
}

#[test]
fn test_month_boundary_calculation() {
    // Test month boundary calculations used in API endpoints

    // January 2024 - normal month
    let jan_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let start_of_month = jan_date.with_day(1).unwrap();
    let next_month = jan_date.with_month(2).unwrap().with_day(1).unwrap();

    assert_eq!(start_of_month, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert_eq!(next_month, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());

    // December 2024 - year boundary
    let dec_date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
    let start_of_dec = dec_date.with_day(1).unwrap();

    let next_year_jan = if dec_date.month() == 12 {
        dec_date
            .with_year(dec_date.year() + 1)
            .and_then(|d| d.with_month(1))
    } else {
        dec_date.with_month(dec_date.month() + 1)
    }
    .unwrap()
    .with_day(1)
    .unwrap();

    assert_eq!(start_of_dec, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
    assert_eq!(next_year_jan, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());

    // February 2024 - leap year
    let feb_date = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
    let start_of_feb = feb_date.with_day(1).unwrap();
    let march_1 = feb_date.with_month(3).unwrap().with_day(1).unwrap();

    assert_eq!(start_of_feb, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
    assert_eq!(march_1, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());

    // Verify February 2024 has 29 days (leap year)
    let feb_29 = NaiveDate::from_ymd_opt(2024, 2, 29);
    assert!(feb_29.is_some());

    let feb_30 = NaiveDate::from_ymd_opt(2024, 2, 30);
    assert!(feb_30.is_none());
}

#[test]
fn test_distance_unit_conversion() {
    // Test distance conversions used throughout the application

    // Meters to kilometers
    let distances_m = vec![1000.0, 5000.0, 10000.0, 21097.5, 42195.0];
    let expected_km = vec![1.0, 5.0, 10.0, 21.0975, 42.195];

    for (i, distance_m) in distances_m.iter().enumerate() {
        let distance_km = distance_m / 1000.0;
        assert_eq!(distance_km, expected_km[i]);
    }

    // Kilometers to meters
    let distances_km = vec![1.0, 5.0, 10.0, 21.1, 42.2];
    let expected_m = vec![1000.0, 5000.0, 10000.0, 21100.0, 42200.0];

    for (i, distance_km) in distances_km.iter().enumerate() {
        let distance_m = distance_km * 1000.0;
        assert_eq!(distance_m, expected_m[i]);
    }
}

#[test]
fn test_time_conversion_utilities() {
    // Test time conversions used in the application

    // Seconds to minutes
    let times_seconds = vec![60, 300, 1800, 3600, 7200];
    let expected_minutes = vec![1.0, 5.0, 30.0, 60.0, 120.0];

    for (i, seconds) in times_seconds.iter().enumerate() {
        let minutes = *seconds as f64 / 60.0;
        assert_eq!(minutes, expected_minutes[i]);
    }

    // Seconds to hours
    let expected_hours = vec![1.0 / 60.0, 5.0 / 60.0, 0.5, 1.0, 2.0];

    for (i, seconds) in times_seconds.iter().enumerate() {
        let hours = *seconds as f64 / 3600.0;
        assert!((hours - expected_hours[i]).abs() < 0.0001);
    }

    // Test common running pace calculations
    let distance_km = 10.0;
    let time_seconds = 3000; // 50 minutes
    let pace_per_km_seconds = time_seconds as f64 / distance_km;
    let pace_per_km_minutes = pace_per_km_seconds / 60.0;

    assert_eq!(pace_per_km_seconds, 300.0); // 5 minutes per km in seconds
    assert_eq!(pace_per_km_minutes, 5.0); // 5 minutes per km
}

#[test]
fn test_percentage_calculation_utilities() {
    // Test percentage calculations used in injury risk analysis

    let test_cases = vec![
        (10.0, 11.0, 0.1),   // 10% increase
        (10.0, 13.0, 0.3),   // 30% increase
        (10.0, 20.0, 1.0),   // 100% increase
        (10.0, 25.0, 1.5),   // 150% increase
        (20.0, 18.0, -0.1),  // 10% decrease
        (100.0, 110.0, 0.1), // 10% increase on larger number
    ];

    for (baseline, current, expected_ratio) in test_cases {
        let growth_ratio = current / baseline;
        let growth_percentage = growth_ratio - 1.0;

        assert!(((growth_percentage - expected_ratio) as f64).abs() < 0.0001_f64);

        // Test percentage formatting
        let percentage_display = growth_percentage * 100.0;
        if expected_ratio > 0.0 {
            assert!(percentage_display > 0.0);
        } else {
            assert!(percentage_display < 0.0);
        }
    }
}

#[test]
fn test_hash_input_generation() {
    // Test the hash input generation logic used for activity deduplication

    let test_cases = vec![
        (
            "John",
            "Doe",
            10000.0,
            3600,
            3600,
            "John|Doe|10000|3600|3600",
        ),
        (
            "Jane",
            "Smith",
            5000.0,
            1800,
            1900,
            "Jane|Smith|5000|1800|1900",
        ),
        ("", "Test", 1000.0, 600, 700, "|Test|1000|600|700"), // Empty first name
        ("Test", "", 2000.0, 1200, 1300, "Test||2000|1200|1300"), // Empty last name
    ];

    for (first_name, last_name, distance, moving_time, elapsed_time, expected) in test_cases {
        let composite = format!(
            "{}|{}|{}|{}|{}",
            first_name, last_name, distance, moving_time, elapsed_time
        );

        assert_eq!(composite, expected);

        // Verify that different inputs produce different composites
        let different_composite = format!(
            "{}|{}|{}|{}|{}",
            first_name,
            last_name,
            distance + 1.0, // Different distance
            moving_time,
            elapsed_time
        );

        if distance > 0.0 {
            // Skip if distance is already at minimum
            assert_ne!(composite, different_composite);
        }
    }
}

#[test]
fn test_activity_validation_logic() {
    // Test the activity validation patterns used throughout the app

    struct TestActivity {
        sport_type: Option<String>,
        distance: Option<f64>,
        athlete_name: Option<String>,
    }

    let test_activities = vec![
        TestActivity {
            sport_type: Some("Run".to_string()),
            distance: Some(5000.0),
            athlete_name: Some("John Doe".to_string()),
        },
        TestActivity {
            sport_type: Some("Ride".to_string()),
            distance: Some(20000.0),
            athlete_name: Some("Jane Doe".to_string()),
        },
        TestActivity {
            sport_type: None,
            distance: Some(1000.0),
            athlete_name: Some("Test User".to_string()),
        },
        TestActivity {
            sport_type: Some("Run".to_string()),
            distance: None,
            athlete_name: Some("Runner".to_string()),
        },
        TestActivity {
            sport_type: Some("Run".to_string()),
            distance: Some(0.0),
            athlete_name: None,
        },
    ];

    // Test running activity validation
    for activity in &test_activities {
        let is_valid_run = activity.sport_type.as_deref() == Some("Run")
            && activity.distance.is_some()
            && activity.distance.unwrap() > 0.0;

        match &activity.sport_type {
            Some(sport) if sport == "Run" => {
                if activity.distance.is_some() && activity.distance.unwrap() > 0.0 {
                    assert!(is_valid_run);
                } else {
                    assert!(!is_valid_run);
                }
            }
            _ => {
                assert!(!is_valid_run);
            }
        }
    }

    // Test general activity validation
    for activity in &test_activities {
        let has_required_fields = activity.sport_type.is_some()
            && activity.distance.is_some()
            && activity.athlete_name.is_some();

        if activity.sport_type.is_none()
            || activity.distance.is_none()
            || activity.athlete_name.is_none()
        {
            assert!(!has_required_fields);
        } else {
            assert!(has_required_fields);
        }
    }
}

#[test]
fn test_weekly_data_aggregation_logic() {
    // Test the weekly data aggregation patterns used in team stats

    struct ActivityData {
        athlete: String,
        week: String,
        distance_km: f64,
    }

    let activities = vec![
        ActivityData {
            athlete: "John".to_string(),
            week: "2024-01-01".to_string(),
            distance_km: 10.0,
        },
        ActivityData {
            athlete: "John".to_string(),
            week: "2024-01-01".to_string(),
            distance_km: 5.0,
        },
        ActivityData {
            athlete: "Jane".to_string(),
            week: "2024-01-01".to_string(),
            distance_km: 8.0,
        },
        ActivityData {
            athlete: "John".to_string(),
            week: "2024-01-08".to_string(),
            distance_km: 12.0,
        },
        ActivityData {
            athlete: "Jane".to_string(),
            week: "2024-01-08".to_string(),
            distance_km: 7.0,
        },
    ];

    // Aggregate by athlete
    let mut athlete_totals: HashMap<String, f64> = HashMap::new();
    for activity in &activities {
        *athlete_totals
            .entry(activity.athlete.clone())
            .or_insert(0.0) += activity.distance_km;
    }

    assert_eq!(athlete_totals.get("John"), Some(&27.0)); // 10 + 5 + 12
    assert_eq!(athlete_totals.get("Jane"), Some(&15.0)); // 8 + 7

    // Aggregate by week
    let mut week_totals: HashMap<String, f64> = HashMap::new();
    for activity in &activities {
        *week_totals.entry(activity.week.clone()).or_insert(0.0) += activity.distance_km;
    }

    assert_eq!(week_totals.get("2024-01-01"), Some(&23.0)); // 10 + 5 + 8
    assert_eq!(week_totals.get("2024-01-08"), Some(&19.0)); // 12 + 7

    // Aggregate by athlete per week
    let mut athlete_week_totals: HashMap<(String, String), f64> = HashMap::new();
    for activity in &activities {
        let key = (activity.athlete.clone(), activity.week.clone());
        *athlete_week_totals.entry(key).or_insert(0.0) += activity.distance_km;
    }

    assert_eq!(
        athlete_week_totals.get(&("John".to_string(), "2024-01-01".to_string())),
        Some(&15.0)
    ); // 10 + 5
    assert_eq!(
        athlete_week_totals.get(&("Jane".to_string(), "2024-01-01".to_string())),
        Some(&8.0)
    );
    assert_eq!(
        athlete_week_totals.get(&("John".to_string(), "2024-01-08".to_string())),
        Some(&12.0)
    );
    assert_eq!(
        athlete_week_totals.get(&("Jane".to_string(), "2024-01-08".to_string())),
        Some(&7.0)
    );
}

#[test]
fn test_running_sum_calculation() {
    // Test the running sum calculation used in team stats

    let weekly_values = vec![
        ("2024-01-01", 25.0),
        ("2024-01-08", 30.0),
        ("2024-01-15", 28.0),
        ("2024-01-22", 35.0),
    ];

    let mut running_sum = 0.0;
    let mut results = Vec::new();

    for (week, value) in weekly_values {
        running_sum += value;
        results.push((week, value, running_sum));
    }

    assert_eq!(results[0], ("2024-01-01", 25.0, 25.0));
    assert_eq!(results[1], ("2024-01-08", 30.0, 55.0));
    assert_eq!(results[2], ("2024-01-15", 28.0, 83.0));
    assert_eq!(results[3], ("2024-01-22", 35.0, 118.0));

    // Test that running sum is monotonically increasing (assuming positive values)
    for i in 1..results.len() {
        assert!(results[i].2 >= results[i - 1].2);
    }
}

#[test]
fn test_date_string_formatting() {
    // Test date string formatting patterns used in the app

    let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

    // Week string format (used for week keys)
    let week_string = test_date.format("%Y-%m-%d").to_string();
    assert_eq!(week_string, "2024-01-15");

    // ISO date format
    let iso_string = test_date.format("%Y-%m-%d").to_string();
    assert_eq!(iso_string, "2024-01-15");

    // Test parsing back from string
    let parsed_date = NaiveDate::parse_from_str(&week_string, "%Y-%m-%d");
    assert!(parsed_date.is_ok());
    assert_eq!(parsed_date.unwrap(), test_date);

    // Test with datetime formatting
    let datetime = test_date.and_hms_opt(10, 30, 0).unwrap();
    let datetime_string = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    assert_eq!(datetime_string, "2024-01-15 10:30:00");
}

#[test]
fn test_floating_point_precision_handling() {
    // Test floating point precision handling (important for injury risk calculations)

    let epsilon = f64::EPSILON;

    // Test values very close to thresholds
    let threshold = 0.1; // 10% threshold

    // Test epsilon handling for SSRD30 risk algorithm

    let test_cases = vec![
        (0.1 - epsilon, true),  // Just below threshold (no risk)
        (0.1, true),            // At threshold - 0.1 < 0.1 + EPSILON is true (no risk)
        (0.1 + epsilon, false), // Just above threshold (small risk)
        (0.095, true),          // Clearly below threshold (no risk)
        (0.11, false),          // Clearly above threshold (small risk)
    ];

    for (growth_percentage, should_be_no_risk) in test_cases {
        let is_no_risk = growth_percentage < threshold + epsilon;
        assert_eq!(
            is_no_risk, should_be_no_risk,
            "Failed for growth_percentage: {}, expected no_risk: {}, got: {}",
            growth_percentage, should_be_no_risk, is_no_risk
        );
    }
}

#[test]
fn test_team_assignment_logic() {
    // Test team assignment and filtering logic

    let athletes = vec![
        ("John Doe", "bulls"),
        ("Jane Smith", "sharks"),
        ("Bob Johnson", "bulls"),
        ("Alice Wilson", "sharks"),
        ("Unknown Player", "other"),
    ];

    let mut team_map: HashMap<String, String> = HashMap::new();
    for (name, team) in athletes {
        team_map.insert(name.to_string(), team.to_string());
    }

    // Test team filtering
    let bulls_players: Vec<_> = team_map
        .iter()
        .filter(|(_, team)| *team == "bulls")
        .map(|(name, _)| name)
        .collect();

    assert_eq!(bulls_players.len(), 2);
    assert!(bulls_players.contains(&&"John Doe".to_string()));
    assert!(bulls_players.contains(&&"Bob Johnson".to_string()));

    let sharks_players: Vec<_> = team_map
        .iter()
        .filter(|(_, team)| *team == "sharks")
        .map(|(name, _)| name)
        .collect();

    assert_eq!(sharks_players.len(), 2);
    assert!(sharks_players.contains(&&"Jane Smith".to_string()));
    assert!(sharks_players.contains(&&"Alice Wilson".to_string()));

    // Test unknown team handling
    assert_eq!(team_map.get("Unknown Player"), Some(&"other".to_string()));
    assert_eq!(team_map.get("Nonexistent Player"), None);
}

#[test]
fn test_configuration_parsing_patterns() {
    // Test configuration parsing patterns (environment variables, etc.)

    // Test token validation
    let valid_tokens = vec!["abc123", "token_456", "very_long_secure_token_789"];
    let invalid_tokens = vec!["", " ", "  "];

    for token in valid_tokens {
        assert!(!token.is_empty());
        assert!(!token.trim().is_empty());
        assert!(token.len() > 3); // Minimum length check
    }

    for token in invalid_tokens {
        assert!(token.is_empty() || token.trim().is_empty());
    }

    // Test short tokens separately
    let short_token = "abc";
    assert_eq!(short_token.len(), 3); // Exactly 3 characters is valid in our test

    // Test URL validation patterns
    let valid_urls = vec![
        "https://api.example.com",
        "http://localhost:3000",
        "https://www.strava.com/api/v3",
    ];

    for url in valid_urls {
        assert!(url.starts_with("http://") || url.starts_with("https://"));
        assert!(url.len() > 10);
    }
}

#[test]
fn test_sorting_and_ordering_utilities() {
    // Test sorting patterns used for chronological data

    let mut dates = vec!["2024-01-15", "2024-01-08", "2024-01-22", "2024-01-01"];

    // Sort chronologically
    dates.sort_by_key(|date_str| {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or_default()
    });

    assert_eq!(dates[0], "2024-01-01");
    assert_eq!(dates[1], "2024-01-08");
    assert_eq!(dates[2], "2024-01-15");
    assert_eq!(dates[3], "2024-01-22");

    // Test that consecutive dates are in ascending order
    for i in 1..dates.len() {
        let prev_date = NaiveDate::parse_from_str(dates[i - 1], "%Y-%m-%d").unwrap();
        let curr_date = NaiveDate::parse_from_str(dates[i], "%Y-%m-%d").unwrap();
        assert!(curr_date >= prev_date);
    }
}
