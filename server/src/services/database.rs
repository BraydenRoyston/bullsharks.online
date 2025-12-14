use sqlx::PgPool;
use crate::{error::ApiError, models::{bullshark::BullSharkActivity, oauth::StravaAuthToken}, utils::database_utils};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Database { pool }
    }

    // MARK: Auth Begins
    pub async fn upsert_auth_token(&self, token: &StravaAuthToken) -> Result<(),ApiError> {
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

        Ok(())
    }

    pub async fn get_auth_token(&self, id: &str) -> Result<Option<StravaAuthToken>, ApiError> {
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
                Err(e) => {
                    failed_insertions += 1;
                    eprintln!("Failed to insert activity {:?}: {:?}", activity, e); // too noisy
                }
            }
        }

        println!("Batch insert complete. Attempted to insert {}/{} failed.", failed_insertions, activities.len());

        Ok(())
    }

    pub async fn get_all_activities(&self) -> Result<Vec<BullSharkActivity>, ApiError> {
        use sqlx::Row;

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

        let activities = rows.into_iter().map(|row| {
            BullSharkActivity {
                id: row.get("id"),
                date: row.get("date"),
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

        Ok(activities)
    }
    // MARK: Activities End
}