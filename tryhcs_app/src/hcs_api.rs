use tracing::debug;
use std::sync::Arc;

use crate::{app::HcsAppConfig, hcs_endpoints::HcsEndpoints, utils::{encrypt_payload, extract_hcs_response}};
use tryhcs_shared::{api_params::{ApiResponseData, ErrorMessage}, encryption::Encryption, institution_params::{AuthenticatedUser, LoginResponse}};

#[derive(Clone)]
pub struct HcsApi {
    client: reqwest::Client,
    config: HcsAppConfig,
        encryption: Arc<dyn Encryption>

}

#[async_trait::async_trait]
impl HcsEndpoints for HcsApi  {

    async fn login(&self, login_req: &tryhcs_shared::institution_params::LoginReq) -> eyre::Result<LoginResponse, ErrorMessage> {
        let url = format!("{}/api/login", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), login_req)?;
        
        debug!(url=?url, body=?body);

        let request = self.client.post(url)
        .body(body)
        .send()
        .await;

        let response: ApiResponseData<LoginResponse> = extract_hcs_response(self.encryption.as_ref(), request).await?;
        Ok(response.data)
    }

    async fn verify_otp(&self, verify_otp: &tryhcs_shared::institution_params::VerifyOTP) -> eyre::Result<AuthenticatedUser, ErrorMessage> {
        let url = format!("{}/api/login/complete", self.config.base_api_url);
        let body = encrypt_payload(self.encryption.as_ref(), verify_otp)?;
        
        debug!(url=?url, body=?body);

        let request = self.client.post(url)
        .body(body)
        .send()
        .await;

        let response: ApiResponseData<AuthenticatedUser> = extract_hcs_response(self.encryption.as_ref(), request).await?;
        Ok(response.data)
    }

    async fn get_auth_profile(&self) -> eyre::Result<tryhcs_shared::institution_params::StaffDto, ErrorMessage> {
        todo!()
    }

    async fn get_staff_details(&self, staff_id: &tryhcs_shared::institution_params::StaffId) -> eyre::Result<Option<tryhcs_shared::institution_params::StaffDto>, ErrorMessage> {
        todo!()
    }

    async fn search_staffs_directory(&self, query: Option<&str>) -> eyre::Result<Vec<tryhcs_shared::institution_params::StaffDto>, ErrorMessage> {
        todo!()
    }

    async fn search_departments(&self, query: Option<&str>) -> eyre::Result<Vec<tryhcs_shared::institution_params::DepartmentDto>, ErrorMessage> {
        todo!()
    }

    async fn get_department(&self, department_id: &tryhcs_shared::institution_params::DepartmentId) -> eyre::Result<Option<tryhcs_shared::institution_params::StaffDto>, ErrorMessage> {
        todo!()
    }
}