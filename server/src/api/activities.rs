use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{error::ApiError, models::bullshark::BullSharkActivity, services::database::Database};

pub async fn read_activities(
    State(db): State<Arc<Database>>
) -> Result<Json<Vec<BullSharkActivity>>, ApiError> {
    let activities = db.get_all_activities().await?;
    Ok(Json(activities))
}
