use crate::{error::ApiError, models::oauth::StravaTokenResponse, services::database::Database};
use dashmap::DashMap;
use std::sync::Arc;
use crate::models::oauth::StravaAuthToken;

#[derive(Clone)]
pub struct StravaConfig {
    pub client_id: String,
    pub client_secret: String,
    pub club_id: String,
    pub admin_id: String,
}

impl StravaConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            client_id: std::env::var("STRAVA_CLIENT_ID")?,
            client_secret: std::env::var("STRAVA_CLIENT_SECRET")?,
            club_id: std::env::var("STRAVA_CLUB_ID")?,
            admin_id: "admin".to_string(),
        })
    }
}

pub struct AuthController {
    strava_config: StravaConfig,
    db: Arc<Database>,
    token_cache: Arc<DashMap<String, StravaAuthToken>>,
}

impl AuthController {
    pub fn new(config: StravaConfig, db: Arc<Database>) -> Self {
        AuthController { 
            strava_config: config,
            db: db,
            token_cache: Arc::new(DashMap::new()),
        }
    }

    pub fn get_club_id(&self) -> String {
        return self.strava_config.club_id.to_string();
    }

    pub async fn get_valid_auth_token(&self) -> Result<String, ApiError> {
        self.get_valid_auth_token_for_user(&self.strava_config.admin_id).await
    }

    pub async fn get_valid_auth_token_for_user(&self, user_id: &str) -> Result<String, ApiError> {
        if let Some(cached_token) = self.token_cache.get(user_id) {
            if !cached_token.is_expired() {
                println!("Using cached token for user {}", user_id.to_string());
                return Ok(cached_token.access_token.to_string());
            } else {
                println!("Found expired token in cache. Evicting.");
                self.token_cache.remove(user_id);
            }
        }

        println!("Cache miss. Checking database for fresh token");
        let db_token = self.db.get_auth_token(user_id).await?
              .ok_or_else(|| ApiError::AuthTokenError(
                  format!("No token found for user: {}. Please insert initial token.", user_id)
              ))?;

        println!("Database token retrieved. Checking expiration status...");
        if db_token.is_expired() || db_token.expires_soon() {
            println!("Token is expired or expiring soon. Refreshing via the Strava API...");
            let new_token = self.refresh_token(&db_token).await?;
            println!("Token refresh from Strava completed. Now storing to database...");
            self.store_token(new_token.clone()).await?;
            println!("Token refresh successful and stored.");
            return Ok(new_token.access_token);
        }

        println!("Database token is still valid. Using it.");
        
        self.token_cache.insert(user_id.to_string(), db_token.clone());
        Ok(db_token.access_token)
    }

    async fn refresh_token(&self, old_token: &StravaAuthToken) -> Result<StravaAuthToken, ApiError> {
        println!("[AUTH] refresh_token: Starting token refresh request to Strava API");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| ApiError::ExternalAPIError(format!("Failed to build HTTP client: {}", e)))?;
        println!("[AUTH] refresh_token: Sending POST request to Strava OAuth endpoint");
        let response = client
            .post("https://www.strava.com/oauth/token")
            .form(&[
                ("client_id", self.strava_config.client_id.as_str()),
                ("client_secret", self.strava_config.client_secret.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", old_token.refresh_token.as_str()),
            ])
            .send()
            .await
            .map_err(|e| ApiError::ExternalAPIError(format!("Strava API request failed: {}", e)))?;

        println!("[AUTH] refresh_token: Received response from Strava, status: {}", response.status());
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!("[AUTH] refresh_token: ERROR - Strava returned non-success status {}: {}", status, error_text);
            return Err(ApiError::ExternalAPIError(
                format!("Strava token refresh failed ({}): {}", status, error_text)
            ));
        }

        println!("[AUTH] refresh_token: Parsing Strava response JSON");
        let token_response: StravaTokenResponse = response
            .json()
            .await
            .map_err(|e| ApiError::ExternalAPIError(format!("Failed to parse Strava response: {}", e)))?;

        println!("[AUTH] refresh_token: Successfully parsed new token from Strava");
        Ok(StravaAuthToken::new(old_token.id.clone(), token_response))
    }

    async fn store_token(&self, token: StravaAuthToken) -> Result<(), ApiError> {
        println!("[AUTH] store_token: Inserting token into cache for user '{}'", token.id);
        self.token_cache.insert(token.id.clone(), token.clone());
        println!("[AUTH] store_token: Token cached. Now upserting to database...");
        self.db.upsert_auth_token(&token).await?;
        println!("[AUTH] store_token: Token successfully stored in both cache and database");
        Ok(())
    }
}
