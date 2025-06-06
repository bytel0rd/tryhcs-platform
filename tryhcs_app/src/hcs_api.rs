use either::Either;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, error, info};
use tryhcs_shared::institution_params::{
    AuthorizedUser, CreateDepartment, CreateInstitution, DepartmentAndStaffDto, DepartmentDto, DepartmentShadowId, InitiatedOtp, InstitutionDto, NewStaff, StaffDto, StaffId, StaffShadowId, VerifyOTP
};

use crate::core::AppHook;
use crate::storage::Storage;
use crate::{core::HcsAppConfig, hcs_endpoints::HcsEndpoints, utils::encrypt_payload};
use tryhcs_shared::api_params::{ApiResponseError, PaginatedResult};
use tryhcs_shared::{
    api_params::{ApiResponseData, ErrorMessage},
    encryption::Encryption,
    institution_params::{AuthenticatedUser, LoginResponse},
};

pub const REQUEST_FAILED_ERROR: &str = "REQUEST FAILED";
pub const AUTH_TOKEN_STORAGE_KEY: &str = "SYSTEM|AUTH_TOKEN";
pub const CURRENT_WORKSPACE_STORAGE_KEY: &str = "SYSTEM|WORKSPACE_ID";

#[derive(Clone)]
pub struct HcsApi {
    pub config: Arc<HcsAppConfig>,
    pub encryption: Arc<dyn Encryption>,
    pub storage: Arc<dyn Storage>,
    pub app_hooks: Arc<Option<Box<dyn AppHook>>>,
}

impl HcsApi {
    pub fn new(
        config: Arc<HcsAppConfig>,
        storage: Arc<dyn Storage>,
        encryption: Arc<dyn Encryption>,
        app_hooks: Arc<Option<Box<dyn AppHook>>>,
    ) -> Self {
        HcsApi {
            config,
            encryption,
            storage,
            app_hooks,
        }
    }

    pub async fn get(&self, url: &str) -> eyre::Result<Either<String, ErrorMessage>, ErrorMessage> {
        let client = reqwest::Client::new();
        debug!(method="GET", url=?url);

        let request = client.get(url);
        let request = self.add_request_headers(request).await;
        let request = request.send().await;

        match self.extract_response(url, request).await {
            Ok(value) => value,
            Err(value) => return value,
        }
    }

    async fn add_request_headers(
        &self,
        mut request: reqwest::RequestBuilder,
    ) -> reqwest::RequestBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        request = request.timeout(Duration::from_secs(
            self.config.request_timeout_in_sec as u64,
        ));

        match self.storage.get(AUTH_TOKEN_STORAGE_KEY).await {
            Err(error_message) => {
                error!(message="Storage error while getting the token header", err=?error_message);
            }
            Ok(Some(token)) => match HeaderValue::from_str(&format!("Bearer {}", token)) {
                Err(error_message) => {
                    error!(message="Failed to set authorization token header", err=?error_message);
                }
                Ok(header_value) => {
                    headers.insert(reqwest::header::AUTHORIZATION, header_value);
                }
            },
            _ => {}
        };

        match self.storage.get(CURRENT_WORKSPACE_STORAGE_KEY).await {
            Err(error_message) => {
                error!(message="Storage error while getting the workspace id header", err=?error_message);
            }
            Ok(Some(workspace_code)) => match HeaderValue::from_str(&workspace_code) {
                Err(error_message) => {
                    error!(message="Failed to set workspace id request header", err=?error_message);
                }
                Ok(header_value) => {
                    headers.insert("Workspace", header_value);
                }
            },
            _ => {}
        };

        request = request.headers(headers);
        request
    }

    pub async fn post(
        &self,
        url: &str,
        body: String,
    ) -> eyre::Result<Either<String, ErrorMessage>, ErrorMessage> {
        let client = reqwest::Client::new();
        debug!(url=?url, method="POST", body=?body);
        let request = client.post(url);
        let request = self.add_request_headers(request).await;
        let request = request.body(body).send().await;
        match self.extract_response(url, request).await {
            Ok(value) => value,
            Err(value) => return value,
        }
    }

    pub async fn put(
        &self,
        url: &str,
        body: String,
    ) -> eyre::Result<Either<String, ErrorMessage>, ErrorMessage> {
        let client = reqwest::Client::new();
        debug!(method="POST", url=?url,  body=?body);
        let request = client.put(url);
        let request = self.add_request_headers(request).await;
        let request = request.body(body).send().await;
        match self.extract_response(url, request).await {
            Ok(value) => value,
            Err(value) => return value,
        }
    }

    pub async fn delete(
        &self,
        url: &str,
        body: String,
    ) -> eyre::Result<Either<String, ErrorMessage>, ErrorMessage> {
        let client = reqwest::Client::new();
        debug!(method="DELETE", url=?url,  body=?body);
        let request = client.delete(url);
        let request = self.add_request_headers(request).await;
        let request = request.body(body).send().await;
        match self.extract_response(url, request).await {
            Ok(value) => value,
            Err(value) => return value,
        }
    }

    async fn extract_response(
        &self,
        url: &str,
        request: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<
        Result<Either<String, ErrorMessage>, ErrorMessage>,
        Result<Either<String, ErrorMessage>, ErrorMessage>,
    > {
        match request {
            Err(error) => {
                error!(message="Request failure: ", url=?error.url(), error=?error);
                return Err(Err(REQUEST_FAILED_ERROR.into()));
            }
            Ok(response) => {
                let status = response.status();
                dbg!(&response);
                let response_str = {
                    match response.text().await {
                        Err(err) => {
                            error!(message="Failed to read response body ", error=?err);
                            return Err(Err(REQUEST_FAILED_ERROR.into()));
                        }
                        Ok(raw_str) => raw_str,
                    }
                };

                dbg!(&response_str);
                debug!(url=?url, status=?status, response=?response_str);

                if !status.is_success() {
                    return Err(Ok(Either::Right(ErrorMessage(response_str))));
                }
                return Ok(Ok(Either::Left(response_str)));
            }
        };
    }

    async fn decrypt_response<T: DeserializeOwned, E: DeserializeOwned + Into<ErrorMessage>>(
        &self,
        response: &Either<String, ErrorMessage>,
    ) -> eyre::Result<T, ErrorMessage> {
        match response {
            Either::Left(success) => {
                let result = self.encryption.decrypt(&success)?;
                match serde_json::from_str::<T>(&result) {
                    Err(e) => {
                        error!(message="Success response deserialization failed. ", err=?e);
                        return Err(REQUEST_FAILED_ERROR.into());
                    }
                    Ok(v) => {
                        return Ok(v);
                    }
                };
            }
            Either::Right(ErrorMessage(error)) => {
                let result = self.encryption.decrypt(&error)?;
                match serde_json::from_str::<E>(&result) {
                    Err(e) => {
                        error!(message="Failure deserialization failed. ", err=?e);
                        return Err(REQUEST_FAILED_ERROR.into());
                    }
                    Ok(v) => {
                        return Err(v.into());
                    }
                };
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl HcsEndpoints for HcsApi {
    async fn is_online(&self) -> bool {
        if let Some(app_hooks) = &*self.app_hooks {
            return app_hooks.is_online_callback();
        }
        return true;
    }

    async fn login(
        &self,
        login_req: &tryhcs_shared::institution_params::LoginReq,
    ) -> eyre::Result<LoginResponse, ErrorMessage> {
        let url = format!("{}/workspace/v1/login", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), login_req)?;
        let request = self.post(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<LoginResponse>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn verify_otp(
        &self,
        verify_otp: &tryhcs_shared::institution_params::VerifyOTP,
    ) -> eyre::Result<AuthenticatedUser, ErrorMessage> {
        let url = format!("{}/workspace/v1/login/complete", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), verify_otp)?;

        let request = self.post(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<AuthenticatedUser>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn get_auth_profile(
        &self,
    ) -> eyre::Result<tryhcs_shared::institution_params::AuthorizedUser, ErrorMessage> {
        let url = format!("{}/workspace/v1/user/profile", self.config.base_api_url);
        let request = self.get(&url).await?;
        let response = self
            .decrypt_response::<ApiResponseData<AuthorizedUser>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn get_staff_details(
        &self,
        StaffShadowId(staff_id): &tryhcs_shared::institution_params::StaffShadowId,
    ) -> eyre::Result<tryhcs_shared::institution_params::StaffDto, ErrorMessage> {
        let url = format!(
            "{}/workspace/v1/staffs/{}",
            self.config.base_api_url, staff_id
        );
        let request = self.get(&url).await?;
        let response = self
            .decrypt_response::<ApiResponseData<StaffDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn search_staffs_directory(
        &self,
    ) -> eyre::Result<Vec<tryhcs_shared::institution_params::StaffDto>, ErrorMessage> {
        let url = format!("{}/workspace/v1/staffs", self.config.base_api_url);
        let request = self.get(&url).await?;
        let response = self
            .decrypt_response::<PaginatedResult<StaffDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn search_departments(
        &self,
    ) -> eyre::Result<Vec<tryhcs_shared::institution_params::DepartmentDto>, ErrorMessage> {
        let url = format!("{}/workspace/v1/departments", self.config.base_api_url);
        let request = self.get(&url).await?;
        let response = self
            .decrypt_response::<PaginatedResult<DepartmentDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

     async fn initiate_registration(&self, req: &CreateInstitution) -> eyre::Result<InitiatedOtp, ErrorMessage> {
                let url = format!("{}/workspace/v1/register/initate", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), req)?;
        let request = self.post(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<InitiatedOtp>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
     }
     
    async fn complete_registration(
        &self,
        verify_otp: &VerifyOTP,
    ) -> eyre::Result<InstitutionDto, ErrorMessage> {
                let url = format!("{}/workspace/v1/register/complete", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), verify_otp)?;
        let request = self.post(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<InstitutionDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn get_departments(&self, DepartmentShadowId(id): &DepartmentShadowId) -> eyre::Result<DepartmentAndStaffDto, ErrorMessage>{
        let url = format!(
            "{}/workspace/v1/departments/{}",
            self.config.base_api_url, id
        );
        let request = self.get(&url).await?;
        let response = self
            .decrypt_response::<ApiResponseData<DepartmentAndStaffDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn create_departments(&self, req: &CreateDepartment) -> eyre::Result<DepartmentDto, ErrorMessage>{
                let url = format!("{}/workspace/v1/departments", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), req)?;
        let request = self.post(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<DepartmentDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }
    async fn edit_departments(&self, DepartmentShadowId(id): &DepartmentShadowId, req: &CreateDepartment) -> eyre::Result<DepartmentDto, ErrorMessage>{
                let url = format!("{}/workspace/v1/departments/{}", self.config.base_api_url, id);
        let body = encrypt_payload(self.encryption.as_ref(), req)?;
        let request = self.put(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<DepartmentDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }
    async fn delete_departments(&self, DepartmentShadowId(id): &DepartmentShadowId) -> eyre::Result<(), ErrorMessage>{
                let url = format!("{}/workspace/v1/departments/{}", self.config.base_api_url, id);
        let body = encrypt_payload(self.encryption.as_ref(), &json!({}))?;
        let request = self.delete(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<()>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }


    async fn add_staff(
        &self,
        req: &NewStaff,
    ) -> eyre::Result<StaffDto, ErrorMessage>{
                let url = format!("{}/workspace/v1/staffs", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), req)?;
        let request = self.post(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<StaffDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn edit_staff(
        &self,
        StaffShadowId(staff_id): &StaffShadowId,
        req: &NewStaff,
    ) -> eyre::Result<StaffDto, ErrorMessage>{
                let url = format!("{}/workspace/v1/staffs/{}", self.config.base_api_url, staff_id);
        let body = encrypt_payload(self.encryption.as_ref(), req)?;
        let request = self.put(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<StaffDto>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }


    async fn delete_staff(
        &self,
        StaffShadowId(staff_id): &StaffShadowId,
    ) -> eyre::Result<(), ErrorMessage>{
                let url = format!("{}/workspace/v1/staffs/{}", self.config.base_api_url, staff_id);
        let body = encrypt_payload(self.encryption.as_ref(), &json!({}))?;
        let request = self.delete(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<()>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }
    
}
