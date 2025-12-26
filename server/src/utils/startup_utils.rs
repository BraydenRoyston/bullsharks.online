use std::sync::Arc;

use axum::{Router, routing::{get, post}, extract::FromRef};
use sqlx::{PgPool};

use crate::{api::{activities::{read_activities, populate_activities, get_activities_from_this_week, get_activities_from_this_month, get_team_stats}, health::health_check}, services::{activity_controller::ActivityController, auth_controller::{AuthController, StravaConfig}, database::Database, strava_client::StravaClient}};

pub fn get_strava_config() -> StravaConfig {
    return StravaConfig::from_env()
        .expect("Failed to find environment variables.");
}

pub fn get_auth_controller(strava_config: StravaConfig, db: Arc<Database>) -> AuthController {
    return AuthController::new(strava_config, db)
}

pub fn get_strava_client(auth_controller: AuthController) -> StravaClient {
    return StravaClient::new(auth_controller)
}

pub fn get_activity_controller(db: Arc<Database>, strava_client: StravaClient) -> ActivityController {
    ActivityController::new(db, strava_client)
}

pub async fn get_db() -> Arc<Database> {
    let pool = get_pg_pool().await
        .expect("Error: could not create the database connection pool");

    let db = Arc::new(Database::new(pool));

    return db;
}

async fn get_pg_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    println!("Database connected!");
    
    Ok(pool)
}

// AppState holds both Database and ActivityController for routing
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub activity_controller: Arc<ActivityController>,
}

// Allow extracting Database from AppState
impl FromRef<AppState> for Arc<Database> {
    fn from_ref(state: &AppState) -> Arc<Database> {
        state.db.clone()
    }
}

// Allow extracting ActivityController from AppState
impl FromRef<AppState> for Arc<ActivityController> {
    fn from_ref(state: &AppState) -> Arc<ActivityController> {
        state.activity_controller.clone()
    }
}

fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/read", get(read_activities))
        .route("/populate", post(populate_activities))
        .route("/activities/week", get(get_activities_from_this_week))
        .route("/activities/month", get(get_activities_from_this_month))
        .route("/team_stats", get(get_team_stats))
        .with_state(state)
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutdown signal received, starting graceful shutdown");
}

pub async fn create_server(db: Arc<Database>, activity_controller: Arc<ActivityController>) {
    let state = AppState {
        db,
        activity_controller,
    };

    let app = create_app(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind TCP listener.");

    println!("Server running on 8080");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to start server.");
}
