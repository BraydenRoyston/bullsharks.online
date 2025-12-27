use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{error::ApiError, models::athlete::Athlete, services::{database::Database}};

pub async fn get_athletes(
    State(db): State<Arc<Database>>
) -> Result<Json<Vec<Athlete>>, ApiError> {
    let result = db.read_all_athletes().await?;
    Ok(Json(result))
}
