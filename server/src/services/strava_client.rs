use crate::models::club::ClubActivity;
use crate::error::{
    ApiError,
};
use crate::services::auth_controller::{AuthController};

pub struct StravaClient {
    auth_controller: AuthController, 
}

impl StravaClient {
    pub fn new(auth_controller: AuthController) -> Self {
        StravaClient { auth_controller }
    }

    pub async fn read_last_100_activities(&self) -> Result<Vec<ClubActivity>, ApiError> {
        let fresh_token = self.auth_controller.get_valid_auth_token().await?;
        let club_id = self.auth_controller.get_club_id();
        let client = reqwest::Client::new();
        let response = client
        .get(format!("https://www.strava.com/api/v3/clubs/{}/activities", club_id))
        .query(&[
            ("page", "1"),
            ("per_page", "100"),
            ("access_token", &fresh_token),
        ])
        .send()
        .await
        .map_err(|e| {
            ApiError::ExternalAPIError(e.to_string())
        })?;

        // Check the status of the response, log details if needed
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            eprintln!("Strava API returned error {}: {}", status, error_text);
            return Err(ApiError::ExternalAPIError(error_text));
        }

        let club_activities: Vec<ClubActivity> = response
        .json()
        .await
        .map_err(|e| {
            eprintln!("Error deserializing response body: {}", e);
            ApiError::ExternalAPIError(e.to_string())
        })?; 

        Ok(club_activities)
    }

    pub async fn health_check(&self) -> Result<(), ApiError> {
        // Verify we can get a valid auth token from Strava
        self.auth_controller.get_valid_auth_token().await?;
        Ok(())
    }
}
