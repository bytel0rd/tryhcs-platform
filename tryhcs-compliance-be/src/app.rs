use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts, Json};
use reqwest::StatusCode;
use serde_json::{json, Value};
use tryhcs_commons_be::{
    auth::{InstitutionAdminUser, TypeAuthenticated},
    env::EnvConfig,
    redis::Cache,
    AUTH_ID_HEADER_FIELD, WORKSPACE_CODE_HEADER_FIELD,
};
use tryhcs_shared::{
    api_params::ErrorMessage,
    institution_params::{AuthenticatedUser, AuthorizedInstitutionUser},
};

use crate::{api::ComplianceVerification, repo::ComplianceRepo};


pub struct App {
    pub compliance: Arc<dyn ComplianceVerification>,
    pub env: EnvConfig,
    pub redis: Arc<dyn Cache>,
    pub compliance_repo: Arc<dyn ComplianceRepo>,
}
