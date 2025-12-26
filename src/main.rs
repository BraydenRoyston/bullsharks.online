use std::sync::Arc;

use crate::utils::startup_utils;

mod error;
mod api;
mod services;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let strava_config = startup_utils::get_strava_config();
    let db = startup_utils::get_db().await;
    let auth_controller = startup_utils::get_auth_controller(strava_config.clone(), db.clone());
    let strava_client = startup_utils::get_strava_client(auth_controller);

    // Create ActivityController instead of starting scheduler
    let activity_controller = Arc::new(startup_utils::get_activity_controller(
        Arc::clone(&db),
        strava_client
    ));

    // Pass both db and activity_controller to the server
    startup_utils::create_server(db, activity_controller).await;
}
