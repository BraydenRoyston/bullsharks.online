use std::sync::Arc;

use axum::{Router, routing::{get, post}, extract::FromRef};
use chrono::Utc;
use sqlx::{PgPool};
use tokio_cron_scheduler::{JobScheduler, Job};

use crate::{api::activities::{read_activities, populate_activities}, error::ApiError, services::{activity_controller::ActivityController, auth_controller::{AuthController, StravaConfig}, database::Database, strava_client::StravaClient}};

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

pub async fn schedule_tasks(db: Arc<Database>, strava_client: StravaClient) -> Result<(), ApiError> {
    let cron_top_of_every_hour =  "0 0 * * * *";
    let fast_cron =  "*/30 * * * * *";

    println!("Creating scheduler and adding tasks...");

    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| ApiError::StartupError(format!("Failed to create scheduler: {}", e)))?;

    // Create the ActivityController instance
    let activity_controller = Arc::new(ActivityController::new(
        db,
        strava_client
    ));

    // Clone the Arc to move into the closure
    let controller: Arc<ActivityController> = Arc::clone(&activity_controller);

    scheduler
        .add(
            Job::new_async(fast_cron, move|_uuid, _lock| { // top of every hour cron
                let controller = Arc::clone(&controller);
                Box::pin(async move {
                    println!("Running job to populate new activities. Current time is {}", Utc::now());
                    if let Err(e) = controller.populate_new_activities().await {
                        eprintln!("Failed to populate new activities: {:?}", e);
                    }
                })
            })
            .map_err(|e| ApiError::StartupError(format!("Failed to create job: {}", e)))?
        )
        .await
        .map_err(|e| ApiError::StartupError(format!("Failed to add job: {}", e)))?;

    scheduler.start()
        .await
        .map_err(|e| ApiError::StartupError(format!("Failed to start scheduler:{}", e)))?;

    println!("Scheduler started successfully.");
    Ok(())
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

async fn health_check() -> &'static str {
    "OK"
}

fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/read", get(read_activities))
        .route("/populate", post(populate_activities))
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
