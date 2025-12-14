
/*
These are external models defined by Strava.
*/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StravaAuthToken {
    pub id: String,
    pub token_type: String, 
    pub access_token: String,
    pub expires_at: i64,  
    pub expires_in: i32, 
    pub refresh_token: String,
}

impl StravaAuthToken {
    pub fn new(id: String, response: StravaTokenResponse) -> Self {          
        StravaAuthToken {
            id,
            token_type: response.token_type,
            access_token: response.access_token,
            expires_at: response.expires_at,
            expires_in: response.expires_in,
            refresh_token: response.refresh_token,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= self.expires_at
    }

    /// Check if token expires soon (within 5 minutes)
    pub fn expires_soon(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at - now < 300  // 5 minutes
    }
}

/// Matches Strava's exact OAuth response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StravaTokenResponse {
    pub token_type: String,
    pub access_token: String,
    pub expires_at: i64,
    pub expires_in: i32,
    pub refresh_token: String,
}