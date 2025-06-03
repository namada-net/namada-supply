use crate::{error::ApiError, state::CommonState};
use axum::{extract::State, http::HeaderMap};
use axum_macros::debug_handler;

#[debug_handler]
pub async fn get_total_supply(
    _headers: HeaderMap,
    State(mut state): State<CommonState>,
) -> Result<String, ApiError> {
    let supply = state.client.get_native_total_supply().await.map_err(|e| {
        tracing::error!("Failed to get total supply: {}", e);
        ApiError::RpcTimeout
    })?;

    Ok(supply)
}

#[debug_handler]
pub async fn get_effective_supply(
    _headers: HeaderMap,
    State(mut state): State<CommonState>,
) -> Result<String, ApiError> {
    let supply = state
        .client
        .get_effective_total_supply()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get total supply: {}", e);
            ApiError::RpcTimeout
        })?;

    Ok(supply)
}
