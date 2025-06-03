use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::response::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Couldn't query rpc")]
    RpcTimeout,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code: StatusCode = match self {
            ApiError::RpcTimeout => StatusCode::GATEWAY_TIMEOUT,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
