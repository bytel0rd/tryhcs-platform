use async_trait::async_trait;
use bon::Builder;
use chrono::{DateTime, Utc};
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tryhcs_commons_be::{
    auth::{InstitutionAdminUser, TypeAuthenticated},
    AUTH_ID_HEADER_FIELD, WORKSPACE_CODE_HEADER_FIELD,
};

use either::Either;
use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts, Json};
use reqwest::StatusCode;
use serde_json::json;

use tryhcs_shared::{
    api_params::ErrorMessage,
    institution_params::{AuthenticatedUser, AuthorizedInstitutionUser},
};

use crate::app::ComplianceApp;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate)  struct WorkspaceAdmin(pub InstitutionAdminUser);

impl FromRequestParts<Arc<ComplianceApp>> for WorkspaceAdmin {
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(
        req: &mut Parts,
        state: &Arc<ComplianceApp>,
    ) -> Result<Self, Self::Rejection> {
        let status = StatusCode::FORBIDDEN;
        let status_code: u16 = status.as_u16();

        // Manually extract session_id and workspace_code from headers
        let error_message = "Invalid session";
        let session_id = {
            let header_value = req
                .headers
                .get(AUTH_ID_HEADER_FIELD)
                .map(|v| v.to_str().ok())
                .flatten();

            match header_value {
                None => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(
                            json!({"message": error_message, "status_code": StatusCode::UNAUTHORIZED.as_u16()}),
                        ),
                    ));
                }
                Some(header_value) => {
                    let split = header_value.split(" ").into_iter().collect::<Vec<_>>();
                    if split.len() != 2 {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            Json(
                                json!({"message": error_message, "status_code": StatusCode::UNAUTHORIZED.as_u16()}),
                            ),
                        ));
                    }

                    if split
                        .get(0)
                        .map(|v| !v.eq_ignore_ascii_case("Bearer"))
                        .unwrap_or(true)
                    {
                        return Err((
                            StatusCode::UNAUTHORIZED,
                            Json(
                                json!({"message": error_message, "status_code": StatusCode::UNAUTHORIZED.as_u16()}),
                            ),
                        ));
                    }

                    split.get(1).unwrap_or(&"").to_owned()
                }
            }
        };

        let workspace_code = {
            let header_value = req
                .headers
                .get(WORKSPACE_CODE_HEADER_FIELD)
                .map(|v| v.to_str().ok())
                .flatten();

            match header_value {
                None => {
                    return Err((
                        StatusCode::FORBIDDEN,
                        Json(
                            json!({"message": "Workpace code is required!", "status_code": StatusCode::FORBIDDEN.as_u16()}),
                        ),
                    ));
                }
                Some(header_value) => header_value.to_owned(),
            }
        };

        let cached_session = state.redis.get_key(&session_id).await;
        let cached_session = cached_session.map(|c| {
            c.map(|v| serde_json::from_str::<AuthenticatedUser>(&v).ok())
                .flatten()
        });

        let user = match cached_session {
            Err(err) => {
                tracing::error!(message="Get session error", err=?err);
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(
                        json!({"message": error_message, "status_code": StatusCode::UNAUTHORIZED.as_u16()}),
                    ),
                ));
            }
            Ok(cached_session) => match cached_session {
                None => {
                    tracing::error!(message = "Cache session not found");
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(
                            json!({"message": error_message, "status_code": StatusCode::UNAUTHORIZED.as_u16()}),
                        ),
                    ));
                }
                Some(auth_user) => {
                    match AuthorizedInstitutionUser::from_authorized_user(
                        auth_user.principal,
                        &workspace_code,
                    ) {
                        Err(err) => {
                            tracing::error!(message="Error serializing session", err=?err);
                            return Err((
                                StatusCode::UNAUTHORIZED,
                                Json(
                                    json!({"message": error_message, "status_code": StatusCode::UNAUTHORIZED.as_u16()}),
                                ),
                            ));
                        }
                        Ok(institution_user) => match institution_user {
                            Either::Right((status, message)) => {
                                let status_code = status.as_u16();
                                return Err((
                                    status,
                                    Json(json!({"message": message, "status_code": status_code})),
                                ));
                            }
                            Either::Left(data) => data,
                        },
                    }
                }
            },
        };

        match InstitutionAdminUser::new(user).await {
            Err(ErrorMessage(message)) => {
                tracing::error!("Error extracting user into admin; {}", &message);
                return Err((
                    status,
                    Json(json!({"message": message, "status_code": status_code})),
                ));
            }
            Ok(user) => {
                return Ok(WorkspaceAdmin(user));
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate)  struct WorkspaceUser(pub AuthorizedInstitutionUser);

impl FromRequestParts<Arc<ComplianceApp>> for WorkspaceUser {
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(
        req: &mut Parts,
        state: &Arc<ComplianceApp>,
    ) -> Result<Self, Self::Rejection> {
        let wrap_error = |status: StatusCode, message: &str| {
            let status_code = status.as_u16();
            (
                status,
                Json(json!({"message": message,
                 "status_code": status_code})),
            )
        };

        let error_message = "Invalid session";
        let session_id = {
            let header_value = req
                .headers
                .get(AUTH_ID_HEADER_FIELD)
                .map(|v| v.to_str().ok())
                .flatten();

            match header_value {
                None => {
                    return Err(wrap_error(StatusCode::UNAUTHORIZED, error_message));
                }
                Some(header_value) => {
                    let split = header_value.split(" ").into_iter().collect::<Vec<_>>();
                    if split.len() != 2 {
                        return Err(wrap_error(StatusCode::UNAUTHORIZED, error_message));
                    }

                    if split
                        .get(0)
                        .map(|v| !v.eq_ignore_ascii_case("Bearer"))
                        .unwrap_or(true)
                    {
                        return Err(wrap_error(StatusCode::UNAUTHORIZED, error_message));
                    }

                    split.get(1).unwrap_or(&"").to_owned()
                }
            }
        };

        let workspace_code = {
            let header_value = req
                .headers
                .get(WORKSPACE_CODE_HEADER_FIELD)
                .map(|v| v.to_str().ok())
                .flatten();

            match header_value {
                None => {
                    return Err(wrap_error(
                        StatusCode::FORBIDDEN,
                        "Workpace code is required!",
                    ));
                }
                Some(header_value) => header_value.to_owned(),
            }
        };

        let cached_session = state.redis.get_key(&session_id).await;
        dbg!(session_id, &workspace_code, &cached_session);

        let cached_session = cached_session.map(|c| {
            c.map(|v| {
                dbg!(&serde_json::from_str::<AuthenticatedUser>(&v));
                serde_json::from_str::<AuthenticatedUser>(&v).ok()
            })
            .flatten()
        });

        match cached_session {
            Err(err) => {
                tracing::error!(message="Get session error", err=?err);
                return Err(wrap_error(StatusCode::UNAUTHORIZED, error_message));
            }

            Ok(cached_session) => match cached_session {
                None => {
                    tracing::error!(message = "Cache session not found");
                    return Err(wrap_error(StatusCode::UNAUTHORIZED, error_message));
                }
                Some(auth_user) => {
                    match AuthorizedInstitutionUser::from_authorized_user(
                        auth_user.principal,
                        &workspace_code,
                    ) {
                        Err(err) => {
                            tracing::error!(message="Error serializing session", err=?err);
                            return Err(wrap_error(StatusCode::UNAUTHORIZED, error_message));
                        }
                        Ok(institution_user) => match institution_user {
                            Either::Right((status, message)) => {
                                let status_code = status.as_u16();
                                return Err((
                                    status,
                                    Json(json!({"message": message,
                                             "status_code": status_code})),
                                ));
                            }
                            Either::Left(data) => {
                                return Ok(WorkspaceUser(data));
                            }
                        },
                    }
                }
            },
        }
    }
}
