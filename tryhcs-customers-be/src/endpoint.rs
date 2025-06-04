use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::Duration;
use either::Either;
use serde_json::{json, Value};
use tracing::error;
use tryhcs_commons_be::{
    api_response::{convert_result_to_json_response, ErrorMessage},
    AUTH_ID_HEADER_FIELD,
};
use tryhcs_shared::{
    api_params::PaginatedQuery,
    institution_params::{
        AuthenticatedUser, AuthorizedInstitutionUser, CreateDepartment, CreateInstitution,
        LoginReq, NewStaff, StaffDto, StaffId, VerifyOTP,
    },
    APIFileUpload,
};

use crate::{
    api::{self, upload_base64_file_api},
    app::App,
    params::{WorkspaceAdmin, WorkspaceUser},
};

pub async fn app_router(app: Arc<App>) -> eyre::Result<Router> {
    let router = Router::new()
        .route("/register/initate", post(create_institution_init_endpoint))
        .route(
            "/register/complete",
            post(create_institution_complete_endpoint),
        )
        .route("/login", post(login_init_endpoint))
        .route("/login/complete", post(login_complete_endpoint))
        .route("/user/profile", get(get_user_profile_endpoint))
        .route("/staffs", get(find_staffs_endpoint))
        .route("/staffs", post(add_staff))
        .route("/staffs/{staff_id}", get(get_staff_profile_endpoint))
        .route("/staffs/{staff_id}", put(edit_staff))
        .route("/staffs/{staff_id}", delete(delete_staff))
        .route("/departments", get(find_departments_endpoint))
        .route(
            "/departments/{department_id}",
            get(get_department_and_staff_endpoint),
        )
        .route("/departments/{department_id}", put(edit_department))
        .route("/departments/{department_id}", delete(delete_department))
        .route("/departments", post(create_department))
        // File uploads module
        .route("/file-uploads/v1/upload", post(generic_upload_endpoint))
        // Finance module
        // .route("/finance/v1/banks", get(get_banks_endpoint))
        .with_state(app);

    return Ok(router);
}

#[axum::debug_handler]
pub async fn generic_upload_endpoint(
    State(app): State<Arc<App>>,
    WorkspaceUser(user): WorkspaceUser,
    Json(req): Json<APIFileUpload>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(upload_base64_file_api(app.as_ref(), &user, &req).await)
}

// #[axum::debug_handler]
// pub async fn get_banks_endpoint(
//     State(app): State<Arc<App>>,
//     WorkspaceUser(user): WorkspaceUser,
// ) -> (StatusCode, Json<Value>) {
//     convert_result_to_json_response(finance_api::get_banks_api(app.as_ref()).await)
// }

//

#[axum::debug_handler]
pub async fn create_institution_init_endpoint(
    State(app): State<Arc<App>>,
    Json(req): Json<CreateInstitution>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::create_institution_init(app.as_ref(), &req).await)
}

#[axum::debug_handler]
pub async fn create_institution_complete_endpoint(
    State(app): State<Arc<App>>,
    Json(req): Json<VerifyOTP>,
) -> (StatusCode, Json<Value>) {
    let result = api::create_institution_complete(app.as_ref(), &req).await;
    convert_result_to_json_response(result)
}

async fn add_user_session(app: Arc<App>, authenticated: &AuthenticatedUser) {
    dbg!((&authenticated.token, &authenticated));

    let mut duplicate = authenticated.clone();
    duplicate.token = None;

    match (serde_json::to_string(&duplicate), &authenticated.token) {
        (Err(err), _) => {
            tracing::error!(message="Failed to serialized authenticated user for session", err=?err);
        }
        (Ok(str_value), Some(token)) => {
            if let Err(err) = app
                .redis
                .set_key(
                    token,
                    &str_value,
                    Some(
                        Duration::minutes(app.env.session_expires_in_min as i64).num_seconds()
                            as u64,
                    ),
                )
                .await
            {
                tracing::error!(message="Error occurred during session insertion", err=?err);
            };
        }
        _ => {
            tracing::error!(message = "Failed to setup authenticated user for session");
        }
    }
}

#[axum::debug_handler]
pub async fn login_init_endpoint(
    State(app): State<Arc<App>>,

    Json(req): Json<LoginReq>,
) -> (StatusCode, Json<Value>) {
    let result = api::login_init(app.as_ref(), &req).await;
    if let Ok((_, Either::Left(Some(authenticated)))) = &result {
        if let Some(authenticated) = &authenticated.auth {
            add_user_session(app, authenticated).await;
        }
    }
    convert_result_to_json_response(result)
}

#[axum::debug_handler]
pub async fn login_complete_endpoint(
    State(app): State<Arc<App>>,
    Json(req): Json<VerifyOTP>,
) -> (StatusCode, Json<Value>) {
    let result = api::login_complete(app.as_ref(), &req).await;
    if let Ok((_, Either::Left(Some(authenticated)))) = &result {
        add_user_session(app, authenticated).await;
    }
    convert_result_to_json_response(result)
}

#[axum::debug_handler]
pub async fn get_user_profile_endpoint(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    WorkspaceUser(user_): WorkspaceUser,
) -> (StatusCode, Json<Value>) {
    let response = {
        match headers.get(AUTH_ID_HEADER_FIELD) {
            None => (
                StatusCode::UNAUTHORIZED,
                Either::Right(ErrorMessage("Unauthorized".into())),
            ),
            Some(session_key) => {
                if let Ok(session_key) = session_key.to_str() {
                    match app.redis.get_key(session_key).await {
                        Err(err) => {
                            error!(message="Failed to get session from cache", err=?err);
                            (
                                StatusCode::UNAUTHORIZED,
                                Either::Right(ErrorMessage("Unable to get profile".into())),
                            )
                        }
                        Ok(session) => {
                            let session = session
                                .and_then(|s| serde_json::from_str::<AuthenticatedUser>(&s).ok());
                            match session {
                                None => {
                                    error!(message = "Invalid session");
                                    (
                                        StatusCode::UNAUTHORIZED,
                                        Either::Right(ErrorMessage("Unable to get profile".into())),
                                    )
                                }
                                Some(data) => (StatusCode::OK, Either::Left(Some(data))),
                            }
                        }
                    }
                } else {
                    (
                        StatusCode::UNAUTHORIZED,
                        Either::Right(ErrorMessage("Unable to get profile".into())),
                    )
                }
            }
        }
    };

    convert_result_to_json_response(Ok(response))
}

#[axum::debug_handler]
pub async fn get_staff_profile_endpoint(
    State(app): State<Arc<App>>,
    WorkspaceUser(user): WorkspaceUser,
    Path(staff_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(
        api::get_staff_profile(app.as_ref(), &user, &staff_id).await,
    )
}

#[axum::debug_handler]
pub async fn find_staffs_endpoint(
    State(app): State<Arc<App>>,
    WorkspaceUser(user): WorkspaceUser,
    Query(req_query): Query<PaginatedQuery>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::find_staffs(app.as_ref(), &user, &req_query).await)
}

#[axum::debug_handler]
pub async fn find_departments_endpoint(
    State(app): State<Arc<App>>,
    WorkspaceUser(user): WorkspaceUser,
    Query(req_query): Query<PaginatedQuery>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::find_departments(app.as_ref(), &user, &req_query).await)
}

#[axum::debug_handler]
pub async fn get_department_and_staff_endpoint(
    State(app): State<Arc<App>>,
    WorkspaceUser(user): WorkspaceUser,
    Path(department_id): Path<i64>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(
        api::get_department_and_staffs(app.as_ref(), &user, department_id).await,
    )
}

#[axum::debug_handler]
pub async fn create_department(
    State(app): State<Arc<App>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Json(req): Json<CreateDepartment>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::create_department(app.as_ref(), &user, req).await)
}

#[axum::debug_handler]
pub async fn edit_department(
    State(app): State<Arc<App>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Path(department_id): Path<String>,
    Json(req): Json<CreateDepartment>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(
        api::edit_department(app.as_ref(), &user, &department_id, req).await,
    )
}

#[axum::debug_handler]
pub async fn delete_department(
    State(app): State<Arc<App>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Path(department_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(
        api::delete_department(app.as_ref(), &user, &department_id).await,
    )
}

#[axum::debug_handler]
pub async fn add_staff(
    State(app): State<Arc<App>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Json(req): Json<NewStaff>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::add_staff(app.as_ref(), &user, req).await)
}

#[axum::debug_handler]
pub async fn edit_staff(
    State(app): State<Arc<App>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Path(staff_id): Path<String>,
    Json(req): Json<NewStaff>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::edit_staff(app.as_ref(), &user, &staff_id, req).await)
}

#[axum::debug_handler]
pub async fn delete_staff(
    State(app): State<Arc<App>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Path(staff_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::delete_staff(app.as_ref(), &user, &staff_id).await)
}
