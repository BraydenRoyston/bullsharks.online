use sqlx::PgPool;
use crate::{error::ApiError, models::{bullshark::BullSharkActivity, oauth::StravaAuthToken}, utils::database_utils};
use chrono::{DateTime, Utc, TimeZone, Offset};
use chrono_tz::America::Los_Angeles;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Database { pool }
    }

    // MARK: Auth Begins
    pub async fn upsert_auth_token(&self, token: &StravaAuthToken) -> Result<(),ApiError> {
        println!("[DB] upsert_auth_token: Starting upsert for user '{}'", token.id);
        sqlx::query(
            r#"
            INSERT INTO strava_auth_tokens
            (id, token_type, access_token, expires_at, expires_in, refresh_token)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET
                token_type = EXCLUDED.token_type,
                access_token = EXCLUDED.access_token,
                expires_at = EXCLUDED.expires_at,
                expires_in = EXCLUDED.expires_in,
                refresh_token = EXCLUDED.refresh_token,
                updated_at = NOW()
            "#
        )
        .bind(&token.id)
        .bind(&token.token_type)
        .bind(&token.access_token)
        .bind(&token.expires_at)
        .bind(&token.expires_in)
        .bind(&token.refresh_token)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to upsert auth token: {}", e)))?;

        println!("[DB] upsert_auth_token: Successfully completed upsert for user '{}'", token.id);
        Ok(())
    }

    pub async fn get_auth_token(&self, id: &str) -> Result<Option<StravaAuthToken>, ApiError> {
        println!("[DB] get_auth_token: Querying database for user '{}'", id);
        let result = sqlx::query(
            r#"
            SELECT id, token_type, access_token, expires_at, expires_in, refresh_token
            FROM strava_auth_tokens
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to get auth token: {}", e)))?;

        println!("[DB] get_auth_token: Query completed for user '{}', found: {}", id, result.is_some());
        Ok(result.map(database_utils::map_row_to_token))
    }
    // MARK: Auth Tokens End





    // MARK: Activities Begin
    pub async fn insert_activity(&self, activity: &BullSharkActivity) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            INSERT INTO bullshark_activities 
            (id, date, resource_state, name, distance, moving_time, elapsed_time, 
            total_elevation_gain, sport_type, workout_type, device_name, athlete_name)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(&activity.id)
        .bind(&activity.date)
        .bind(&activity.resource_state)
        .bind(&activity.name)
        .bind(&activity.distance)
        .bind(&activity.moving_time)
        .bind(&activity.elapsed_time)
        .bind(&activity.total_elevation_gain)
        .bind(&activity.sport_type)
        .bind(&activity.workout_type)
        .bind(&activity.device_name)
        .bind(&activity.athlete_name)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_activities(&self, activities: &[BullSharkActivity]) -> Result<(), ApiError> {
        if activities.len() == 0 {
            println!("insert_activities | received an activities slice with 0 length, skipping batch operation");
            return Ok(())
        }

        let mut failed_insertions = 0;

        for activity in activities {
            match self.insert_activity(activity).await {
                Ok(_) => {}
                Err(_e) => {
                    failed_insertions += 1;
                }
            }
        }

        println!("Batch insert complete. Inserted {:?} new activities.", activities.len() - failed_insertions);

        Ok(())
    }

    pub async fn get_all_activities(&self) -> Result<Vec<BullSharkActivity>, ApiError> {
        use sqlx::Row;

        println!("[DB] get_all_activities: Starting query for all activities");
        let rows = sqlx::query(
            r#"
            SELECT id, date, resource_state, name, distance, moving_time,
                    elapsed_time, total_elevation_gain, sport_type, workout_type, device_name, athlete_name
            FROM bullshark_activities
            ORDER BY date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch activities: {}", e)))?;

        let activities: Vec<BullSharkActivity> = rows.into_iter().map(|row| {
            // Get timestamp from DB (stored as UTC)
            let date_utc: DateTime<Utc> = row.get("date");

            // Convert to Pacific timezone
            let date_pacific_tz = Los_Angeles.from_utc_datetime(&date_utc.naive_utc());
            // Convert to FixedOffset for serialization support
            let date_pacific = date_pacific_tz.with_timezone(&date_pacific_tz.offset().fix());

            BullSharkActivity {
                id: row.get("id"),
                date: date_pacific,
                athlete_name: row.get("athlete_name"),
                resource_state: row.get("resource_state"),
                name: row.get("name"),
                distance: row.get("distance"),
                moving_time: row.get("moving_time"),
                elapsed_time: row.get("elapsed_time"),
                total_elevation_gain: row.get("total_elevation_gain"),
                sport_type: row.get("sport_type"),
                workout_type: row.get("workout_type"),
                device_name: row.get("device_name"),
            }
        }).collect();

        println!("[DB] get_all_activities: Query completed, returned {} activities", activities.len());
        Ok(activities)
    }
    // MARK: Activities End





    // MARK: Health Check
    pub async fn health_check(&self) -> Result<(), ApiError> {
        println!("[DB] health_check: Starting database health check");
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(format!("Health check failed: {}", e)))?;

        println!("[DB] health_check: Health check completed successfully");
        Ok(())
    }
    // MARK: Health Check End
}