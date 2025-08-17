use axum::{extract::State, http::StatusCode, Json};
use shared::*;

use crate::AppState;

pub async fn list_languages(
    State(state): State<AppState>,
) -> Result<Json<Vec<Language>>, StatusCode> {
    let languages = state
        .db
        .list_languages()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(languages))
}