use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Path, Query, State}, http::{request::Parts, StatusCode}, routing::{delete, get, post, put}, Json, RequestPartsExt, Router
};
use chrono::Duration;
use either::Either;
use serde::Serialize;
use serde_json::{json, Value};
use tryhcs_commons_be::api_response::convert_result_to_json_response;
use tryhcs_shared::compliance_params::{CorporateComplianceEdit, FinancialComplianceEdit, HealthcareComplianceEdit};

use crate::{api, app::ComplianceApp, params::WorkspaceAdmin};


pub  fn compliance_router(app: Arc<ComplianceApp>) -> Router {
    let router = Router::new()
        .route("/v1/onboard/overview", get(get_compliance_status))
        .route("/v1/onboard/corporate", post(update_corporate_compliance))
        .route("/v1/onboard/finance", post(update_financial_compliance))
        .route("/v1/onboard/healthcare", post(update_healthcare_compliance))
        .route("/v1/onboard/submit", post(submit_compliance))
        .with_state(app)
        ;
    router
}

#[axum::debug_handler]
pub async fn get_compliance_status(
    State(app): State<Arc<ComplianceApp>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::get_compliance_data(app.as_ref(), &user).await)
}

#[axum::debug_handler]
pub async fn update_corporate_compliance(
    State(app): State<Arc<ComplianceApp>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Json(req): Json<CorporateComplianceEdit>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::update_corporate_compliance(app.as_ref(), &user, &req).await)
}

#[axum::debug_handler]
pub async fn update_financial_compliance(
    State(app): State<Arc<ComplianceApp>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Json(req): Json<FinancialComplianceEdit>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::update_financial_compliance(app.as_ref(), &user, &req).await)
}

#[axum::debug_handler]
pub async fn update_healthcare_compliance(
    State(app): State<Arc<ComplianceApp>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Json(req): Json<HealthcareComplianceEdit>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::update_healthcare_compliance(app.as_ref(), &user, &req).await)
}

#[axum::debug_handler]
pub async fn submit_compliance(
    State(app): State<Arc<ComplianceApp>>,
    WorkspaceAdmin(user): WorkspaceAdmin,
    Json(req): Json<Value>,
) -> (StatusCode, Json<Value>) {
    convert_result_to_json_response(api::submit_compliance(app.as_ref(), &user).await)
}