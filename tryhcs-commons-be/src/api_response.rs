use axum::Json;
use either::Either;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tryhcs_shared::api_params::PaginatedResult;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage(pub String);

pub type ApiResponse<T> = (StatusCode, Either<Option<T>, ErrorMessage>);

pub fn convert_result_to_json_response<T: Serialize>(
    res: eyre::Result<ApiResponse<T>>,
) -> (StatusCode, Json<Value>) {
    match res {
        Ok(api_response) => {
            return convert_to_json_response(api_response);
        }
        Err(err) => {
            tracing::error!(err=?err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error_message": "Request processing failed",
             "status_code": StatusCode::INTERNAL_SERVER_ERROR.as_u16()})),
            );
        }
    }
}

pub fn convert_paginated_result_to_json_response<T: Serialize>(
    res: eyre::Result<(StatusCode, PaginatedResult<T>)>,
) -> (StatusCode, Json<Value>) {
    match res {
        Ok((status, api_response)) => {
            return (status, Json(api_response.into()));
        }
        Err(err) => {
            tracing::error!(err=?err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error_message": "Request processing failed",
                 "status_code": StatusCode::INTERNAL_SERVER_ERROR.as_u16()})),
            );
        }
    }
}

pub fn convert_to_json_response<T: Serialize>(
    (status, result): ApiResponse<T>,
) -> (StatusCode, Json<Value>) {
    let status_code = status.as_u16();

    match result {
        Either::Right(ErrorMessage(message)) => {
            return (
                status,
                Json(json!({"message": message,
                 "status_code": status_code})),
            );
        }
        Either::Left(data) => {
            return (
                status,
                Json(json!({"message": "Successful!",
                 "data": data,
                 "status_code": status_code})),
            );
        }
    }
}
