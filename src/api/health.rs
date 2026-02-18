use axum::{Json, extract::State};
use serde::Serialize;

use crate::utils::startup_utils::AppState;

/// Health status response containing the status of all system components.
///
/// This structure provides detailed health information for monitoring and
/// alerting systems. Each component is checked independently and an overall
/// status is calculated based on all components.
///
/// # Fields
///
/// * `database` - PostgreSQL database connection status
/// * `strava` - Strava API connectivity and authentication status
/// * `overall` - Combined system health status
///
/// # Status Values
///
/// - `"healthy"` - Component is functioning normally
/// - `"unhealthy: <reason>"` - Component has issues, with error details
#[derive(Serialize)]
pub struct HealthStatus {
    pub database: String,
    pub strava: String,
    pub overall: String,
}

/// Comprehensive system health check endpoint.
///
/// This endpoint performs active health checks on all critical system components
/// and returns detailed status information. It's designed for use by monitoring
/// systems, load balancers, and operational dashboards.
///
/// # Health Checks Performed
///
/// ## Database Health
/// - Tests PostgreSQL connection pool connectivity
/// - Executes a simple query to verify database responsiveness
/// - Reports connection pool status and query performance
///
/// ## Strava API Health
/// - Verifies OAuth token validity and refresh capability
/// - Tests connectivity to Strava Club API endpoints
/// - Checks rate limiting status and authentication
///
/// ## Overall Health Logic
/// - `healthy`: All components are healthy
/// - `unhealthy`: One or more components are unhealthy
///
/// # Returns
///
/// * `Json<HealthStatus>` - Detailed health status for all components
///
/// # Response Structure
///
/// ```json
/// {
///   "database": "healthy",
///   "strava": "healthy",
///   "overall": "healthy"
/// }
/// ```
///
/// Or when issues are detected:
///
/// ```json
/// {
///   "database": "unhealthy: connection timeout",
///   "strava": "healthy",
///   "overall": "unhealthy"
/// }
/// ```
///
/// # Usage
///
/// - **Monitoring**: Called by external monitoring systems (Prometheus, etc.)
/// - **Load Balancers**: Used for routing decisions and failover
/// - **CI/CD**: Verify deployment health before promoting to production
/// - **Debugging**: Quick way to diagnose system issues
///
/// # Performance
///
/// This endpoint is optimized for frequent calls:
/// - Lightweight database query (SELECT 1)
/// - Cached Strava token validation where possible
/// - Minimal resource usage for monitoring scenarios
pub async fn health_check(State(state): State<AppState>) -> Json<HealthStatus> {
    let db_status = match state.db.health_check().await {
        Ok(_) => "healthy".to_string(),
        Err(e) => format!("unhealthy: {:?}", e),
    };

    let strava_status = match state.activity_controller.health_check_strava().await {
        Ok(_) => "healthy".to_string(),
        Err(e) => format!("unhealthy: {:?}", e),
    };

    let overall = if db_status == "healthy" && strava_status == "healthy" {
        "healthy".to_string()
    } else {
        "unhealthy".to_string()
    };

    Json(HealthStatus {
        database: db_status,
        strava: strava_status,
        overall,
    })
}
