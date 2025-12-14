use std::sync::Arc;

use axum::{Router, routing::{get}};
use chrono::Utc;
use sqlx::{PgPool};
use tokio_cron_scheduler::{JobScheduler, Job};

use crate::{api::activities::read_activities, error::ApiError, services::{activity_controller::ActivityController, auth_controller::{AuthController, StravaConfig}, database::Database, strava_client::StravaClient}};

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

pub async fn get_db() -> Arc<Database> {
    let pool = get_pg_pool().await
        .expect("Error: could not create the database connection pool");

    let db = Arc::new(Database::new(pool));

    return db;
}

async fn get_pg_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    println!("{}", database_url.to_string());

    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    println!("Database connected!");
    
    Ok(pool)
}

pub async fn schedule_tasks(db: Arc<Database>, strava_client: StravaClient) -> Result<(), ApiError> {
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
            Job::new_async("0 0 * * * *", move|_uuid, _lock| { // top of every hour cron
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


fn create_app(db: Arc<Database>) -> Router {
    Router::new()
        .route("/read", get(read_activities))
        .with_state(db)
}

pub async fn create_server(db: Arc<Database>) {
    let app = create_app(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener.");

    println!("Server running on 3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server.");
}
