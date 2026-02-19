use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    error::ApiError,
    models::{athlete::Athlete, athlete_training_data::AllAthletesTrainingData},
    services::{activity_controller::ActivityController, database::Database},
};

pub async fn get_athletes(State(db): State<Arc<Database>>) -> Result<Json<Vec<Athlete>>, ApiError> {
    let result = db.read_all_athletes().await?;
    Ok(Json(result))
}

pub async fn get_athletes_training_data(
    State(activity_controller): State<Arc<ActivityController>>,
) -> Result<Json<AllAthletesTrainingData>, ApiError> {
    let training_data = activity_controller.get_all_athletes_training_data().await?;
    Ok(Json(training_data))
}
