use axum::{extract::State, http::StatusCode, Json};
use shared::*;

use crate::KernelState;

pub async fn list_languages(
    State(state): State<KernelState>,
) -> Result<Json<Vec<Language>>, StatusCode> {
    let languages = state
        .db
        .list_languages()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(languages))
}