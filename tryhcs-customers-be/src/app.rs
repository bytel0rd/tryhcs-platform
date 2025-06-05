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

use crate::db_repo::EhrDataRepo as CustomerDbRepo;

pub struct CustomersApp {
    pub db_pool: Arc<dyn CustomerDbRepo>,
    pub s3_client: aws_sdk_s3::Client,
    pub env: EnvConfig,
    pub redis: Arc<dyn Cache>,
}
