use sqlx::PgPool;
use crate::{error::ApiError, models::{athlete::Athlete, bullshark::BullSharkActivity, oauth::StravaAuthToken}, utils::database_utils};
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
        if activities.is_empty() {
            println!("insert_activities | received an activities slice with 0 length, skipping batch operation");
            return Ok(())
        }

        // Build arrays for each column
        let ids: Vec<i64> = activities.iter().map(|a| a.id).collect();
        let dates: Vec<DateTime<Utc>> = activities.iter().map(|a| a.date.with_timezone(&Utc)).collect();
        let resource_states: Vec<i32> = activities.iter().map(|a| a.resource_state).collect();
        let names: Vec<String> = activities.iter().map(|a| a.name.clone()).collect();
        let distances: Vec<f64> = activities.iter().map(|a| a.distance).collect();
        let moving_times: Vec<i32> = activities.iter().map(|a| a.moving_time).collect();
        let elapsed_times: Vec<i32> = activities.iter().map(|a| a.elapsed_time).collect();
        let total_elevation_gains: Vec<f64> = activities.iter().map(|a| a.total_elevation_gain).collect();
        let sport_types: Vec<String> = activities.iter().map(|a| a.sport_type.clone()).collect();
        let workout_types: Vec<Option<i32>> = activities.iter().map(|a| a.workout_type).collect();
        let device_names: Vec<Option<String>> = activities.iter().map(|a| a.device_name.clone()).collect();
        let athlete_names: Vec<String> = activities.iter().map(|a| a.athlete_name.clone()).collect();

        // Use PostgreSQL UNNEST to insert all rows in a single query
        let result = sqlx::query(
            r#"
            INSERT INTO bullshark_activities
            (id, date, resource_state, name, distance, moving_time, elapsed_time,
            total_elevation_gain, sport_type, workout_type, device_name, athlete_name)
            SELECT * FROM UNNEST($1::bigint[], $2::timestamptz[], $3::int[], $4::text[], $5::float8[],
                                 $6::int[], $7::int[], $8::float8[], $9::text[], $10::int[],
                                 $11::text[], $12::text[])
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(&ids)
        .bind(&dates)
        .bind(&resource_states)
        .bind(&names)
        .bind(&distances)
        .bind(&moving_times)
        .bind(&elapsed_times)
        .bind(&total_elevation_gains)
        .bind(&sport_types)
        .bind(&workout_types)
        .bind(&device_names)
        .bind(&athlete_names)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to batch insert activities: {}", e)))?;

        println!("Batch insert complete. Inserted {} new activities.", result.rows_affected());

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

    pub async fn get_activities_from_window(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<BullSharkActivity>, ApiError> {
        use sqlx::Row;

        println!("[DB] get_activities_from_window: Starting query for activities between {:?} and {:?}", start, end);
        let rows = sqlx::query(
            r#"
            SELECT id, date, resource_state, name, distance, moving_time,
                    elapsed_time, total_elevation_gain, sport_type, workout_type, device_name, athlete_name
            FROM bullshark_activities
            WHERE date >= $1 AND date <= $2
            ORDER BY date DESC
            "#
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch activities from window: {}", e)))?;

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

        println!("[DB] get_activities_from_window: Query completed, returned {} activities", activities.len());
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





    // MARK: Athletes
    pub async fn insert_athlete(&self, athlete: &Athlete) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            INSERT INTO athletes
            (id, name, team, event)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(&athlete.id)
        .bind(&athlete.name)
        .bind(&athlete.team)
        .bind(&athlete.event)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_athletes(&self, athletes: &[Athlete]) -> Result<(), ApiError> {
        if athletes.is_empty() {
            println!("insert_athletes | received an athletes slice with 0 length, skipping batch operation");
            return Ok(())
        }

        // Build arrays for each column
        let ids: Vec<String> = athletes.iter().map(|a| a.id.clone()).collect();
        let names: Vec<String> = athletes.iter().map(|a| a.name.clone()).collect();
        let teams: Vec<String> = athletes.iter().map(|a| a.team.clone()).collect();
        let events: Vec<String> = athletes.iter().map(|a| a.event.clone()).collect();

        // Use PostgreSQL UNNEST to insert all rows in a single query
        let result = sqlx::query(
            r#"
            INSERT INTO athletes
            (id, name, team, event)
            SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(&ids)
        .bind(&names)
        .bind(&teams)
        .bind(&events)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to batch insert athletes: {}", e)))?;

        println!("Batch insert complete. Inserted {} new athletes.", result.rows_affected());

        Ok(())
    }

    pub async fn read_all_athletes(&self) -> Result<Vec<Athlete>, ApiError> {
        use sqlx::Row;

        println!("[DB] read_all_athletes: Starting query for all athletes");
        let rows = sqlx::query(
            r#"
            SELECT id, name, team, event
            FROM athletes
            ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ApiError::DatabaseError(format!("Failed to fetch athletes: {}", e)))?;

        let athletes: Vec<Athlete> = rows.into_iter().map(|row| {
            Athlete {
                id: row.get("id"),
                name: row.get("name"),
                team: row.get("team"),
                event: row.get("event"),
            }
        }).collect();

        println!("[DB] read_all_athletes: Query completed, returned {} athletes", athletes.len());
        Ok(athletes)
    }
    // MARK: Athletes End
}