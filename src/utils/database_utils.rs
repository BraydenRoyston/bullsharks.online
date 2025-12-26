use crate::models::oauth::StravaAuthToken;
use sqlx::Row;

/// Helper to map a database row to StravaAuthToken
pub fn map_row_to_token(row: sqlx::postgres::PgRow) -> StravaAuthToken {
      // Import Row trait for .get()

    StravaAuthToken {
        id: row.get("id"),
        token_type: row.get("token_type"),
        access_token: row.get("access_token"),
        expires_at: row.get("expires_at"),
        expires_in: row.get("expires_in"),
        refresh_token: row.get("refresh_token"),
    }
}