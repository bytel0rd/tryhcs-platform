use either::Either;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info};

use crate::{core::HcsAppConfig, hcs_endpoints::HcsEndpoints, utils::encrypt_payload};
use tryhcs_shared::api_params::ApiResponseError;
use tryhcs_shared::{
    api_params::{ApiResponseData, ErrorMessage},
    encryption::Encryption,
    institution_params::{AuthenticatedUser, LoginResponse},
};

const REQUEST_FAILED_ERROR: &str = "REQUEST FAILED";

#[derive(Clone)]
pub struct HcsApi {
    pub config: Arc<HcsAppConfig>,
    pub encryption: Arc<dyn Encryption>,
    pub token: Arc<RwLock<Option<String>>>,
}

impl HcsApi {
    pub fn new(config: Arc<HcsAppConfig>, encryption: Arc<dyn Encryption>) -> Self {
        HcsApi {
            config,
            encryption,
            token: Arc::new(RwLock::new(None)),
        }
    }

    // #[cfg(all(not(target_arch = "wasm32"), any(unix, windows)))]
    pub async fn post(
        &self,
        url: &str,
        body: String,
    ) -> eyre::Result<Either<String, ErrorMessage>, ErrorMessage> {
        let client = reqwest::Client::new();

        debug!(url=?url, body=?body);

        let request = client
            .post(url)
            .header(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )
            .body(body)
            .send()
            .await;

        match request {
            Err(error) => {
                error!(message="Request failure: ", url=?error.url(), error=?error);
                return Err(REQUEST_FAILED_ERROR.into());
            }
            Ok(response) => {
                let status = response.status();
                let response_str = {
                    match response.text().await {
                        Err(err) => {
                            error!(message="Failed to read response body ", error=?err);
                            return Err(REQUEST_FAILED_ERROR.into());
                        }
                        Ok(raw_str) => raw_str,
                    }
                };

                debug!(url=?url, status=?status, response=?response_str);

                if !status.is_success() {
                    return Ok(Either::Right(ErrorMessage(response_str)));
                }
                return Ok(Either::Left(response_str));
            }
        }
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
    async fn login(
        &self,
        login_req: &tryhcs_shared::institution_params::LoginReq,
    ) -> eyre::Result<LoginResponse, ErrorMessage> {
        let url = format!("{}/login", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), login_req)?;

        debug!(url=?url, body=?body);

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
        let url = format!("{}/api/login/complete", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), verify_otp)?;

        debug!(url=?url, body=?body);

        let request = self.post(&url, body).await?;
        let response = self
            .decrypt_response::<ApiResponseData<AuthenticatedUser>, ApiResponseError>(&request)
            .await?;
        Ok(response.data)
    }

    async fn get_auth_profile(
        &self,
    ) -> eyre::Result<tryhcs_shared::institution_params::StaffDto, ErrorMessage> {
        todo!()
    }

    async fn get_staff_details(
        &self,
        staff_id: &tryhcs_shared::institution_params::StaffId,
    ) -> eyre::Result<Option<tryhcs_shared::institution_params::StaffDto>, ErrorMessage> {
        todo!()
    }

    async fn search_staffs_directory(
        &self,
        query: Option<&str>,
    ) -> eyre::Result<Vec<tryhcs_shared::institution_params::StaffDto>, ErrorMessage> {
        todo!()
    }

    async fn search_departments(
        &self,
        query: Option<&str>,
    ) -> eyre::Result<Vec<tryhcs_shared::institution_params::DepartmentDto>, ErrorMessage> {
        todo!()
    }

    async fn get_department(
        &self,
        department_id: &tryhcs_shared::institution_params::DepartmentId,
    ) -> eyre::Result<Option<tryhcs_shared::institution_params::StaffDto>, ErrorMessage> {
        todo!()
    }
}
