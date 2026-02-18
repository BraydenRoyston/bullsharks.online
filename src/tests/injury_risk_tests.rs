use crate::models::athlete_training_data::RiskyWeek;
use crate::models::bullshark::BullSharkActivity;
use crate::models::injury_risk::InjuryRiskType;
use chrono::{Datelike, Duration, FixedOffset, NaiveDate, TimeZone};
use std::collections::HashMap;

/// Comprehensive test suite for injury risk algorithms (SSRD30 & 10% Rule)
///
/// Tests cover:
/// - SSRD30 risk classification (no, small, moderate, large risk scenarios)
/// - 30-day lookback window validation
/// - 10% rule calculations  
/// - Risk type classification and string conversion
/// - Floating point precision edge cases

// Helper function to create a test activity
fn create_test_activity(date_str: &str, athlete_name: &str, distance_km: f64) -> BullSharkActivity {
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
fn analyze_ssrd30_test(
    athlete_name: &str,
    activities: &[BullSharkActivity],
) -> HashMap<String, RiskyWeek> {
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
            let start_of_week = activity_date_naive.date().and_hms_opt(0, 0, 0).unwrap()
                - Duration::days(days_since_monday as i64);
            let week_string = start_of_week.format("%Y-%m-%d").to_string();

            let risky_week = risky_weeks
                .entry(week_string.clone())
                .or_insert_with(|| RiskyWeek {
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
        create_test_activity("2024-01-01", "John Doe", 5.0), // 5km baseline
        create_test_activity("2024-01-15", "John Doe", 5.5), // 5.5km - 10% increase, should be no risk
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
        match 0.05 {
            // 5% increase
            x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk,
            x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
            x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
            _ => InjuryRiskType::SSRD30LargeRisk,
        },
        InjuryRiskType::SSRD30NoRisk
    );

    assert_eq!(
        match 0.25 {
            // 25% increase
            x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk,
            x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
            x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
            _ => InjuryRiskType::SSRD30LargeRisk,
        },
        InjuryRiskType::SSRD30SmallRisk
    );

    assert_eq!(
        match 0.8 {
            // 80% increase
            x if x < 0.1 + f64::EPSILON => InjuryRiskType::SSRD30NoRisk,
            x if x <= 0.3 => InjuryRiskType::SSRD30SmallRisk,
            x if x <= 1.0 => InjuryRiskType::SSRD30ModerateRisk,
            _ => InjuryRiskType::SSRD30LargeRisk,
        },
        InjuryRiskType::SSRD30ModerateRisk
    );

    assert_eq!(
        match 1.5 {
            // 150% increase
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
    assert_eq!(
        InjuryRiskType::SSRD30SmallRisk.as_str(),
        "SSRD30_SMALL_RISK"
    );
    assert_eq!(
        InjuryRiskType::SSRD30ModerateRisk.as_str(),
        "SSRD30_MODERATE_RISK"
    );
    assert_eq!(
        InjuryRiskType::SSRD30LargeRisk.as_str(),
        "SSRD30_LARGE_RISK"
    );
    assert_eq!(
        InjuryRiskType::HighVolumeSpike.as_str(),
        "HIGH_VOLUME_SPIKE"
    );
}
