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
    let strava_config= startup_utils::get_strava_config();
    let db = startup_utils::get_db().await;
    let auth_controller = startup_utils::get_auth_controller(strava_config.clone(), db.clone());
    let strava_client = startup_utils::get_strava_client(auth_controller);

    startup_utils::schedule_tasks(
        Arc::clone(&db),
        strava_client
    )
        .await
        .expect("Error: scheduler failed.");

    startup_utils::create_server(db)
        .await;
}
