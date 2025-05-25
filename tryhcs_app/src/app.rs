use std::sync::Arc;

use either::Either;
use serde::{Deserialize, Serialize};
use tracing::debug;
use tryhcs_shared::{api_params::ErrorMessage, encryption::Encryption, institution_params::{AuthenticatedUser, DepartmentDto, DepartmentId, InitiatedOtp, LoginReq, StaffDto, StaffId, VerifyOTP}};

use crate::{hcs_endpoints::HcsEndpoints, storage::Storage, utils::{encrypt_payload, extract_response}};

#[async_trait::async_trait]
pub trait GuestOperations: Send + Sync {
    async fn login(&self, login_req: &LoginReq) -> eyre::Result<Either<Arc<impl UserOperations>, InitiatedOtp>, ErrorMessage>;
    async fn verify_otp(&self, verify_otp: &VerifyOTP) -> eyre::Result<Arc<impl UserOperations>, ErrorMessage>;
}

#[async_trait::async_trait]
pub trait UserOperations {
    async fn get_auth_profile(&self) -> eyre::Result<StaffDto, ErrorMessage>;

    async fn get_staff_details(&self, staff_id: &StaffId) -> eyre::Result<Option<StaffDto>, ErrorMessage>;
    async fn search_staffs_directory(&self, query: Option<&str>) -> eyre::Result<Vec<StaffDto>, ErrorMessage>;

    async fn search_departments(&self, query: Option<&str>) -> eyre::Result<Vec<DepartmentDto>, ErrorMessage>;
    async fn get_department(&self, department_id: &DepartmentId) -> eyre::Result<Option<StaffDto>, ErrorMessage>;
    
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HcsAppConfig {
    pub base_api_url: String,
    pub debug_enabled: bool,
}

#[derive(Clone)]
pub struct CoreApplication {
    hcs_api: Arc<dyn HcsEndpoints>,
    config: HcsAppConfig,
    storage: Arc<dyn Storage>,
    encryption: Arc<dyn Encryption>
}

#[derive(Clone)]
pub struct GuestApplication {
    core: CoreApplication
}

#[async_trait::async_trait]
impl GuestOperations for GuestApplication {
    async fn login(&self, login_req: &LoginReq) -> eyre::Result<Either<Arc<impl UserOperations>, InitiatedOtp>, ErrorMessage> {
            let login_res = self.core.hcs_api.login(login_req).await?;
            if let Some(otp) = login_res.otp {
                return Ok(Either::Right(otp));
            }

            if let Some(authenticated) = login_res.auth {
                return Ok(Either::Left(Arc::new(AuthenticatedApplication::new(authenticated, self.core.clone()))));
            } 

        Err(ErrorMessage("#Authentication failure".into()))
    }

    async fn verify_otp(&self, verify_otp: &VerifyOTP) -> eyre::Result<Arc<impl UserOperations>, ErrorMessage> {
             let authenticated = self.core.hcs_api.verify_otp(verify_otp).await?;
            return Ok(Arc::new(AuthenticatedApplication::new(authenticated, self.core.clone())));
    }
}


#[derive(Clone)]
pub struct AuthenticatedApplication {
    user: AuthenticatedUser,
    core: CoreApplication

}

impl AuthenticatedApplication  {
    pub fn new(user: AuthenticatedUser, core: CoreApplication) -> Self {
        Self {core, user}
    }
}

#[async_trait::async_trait]
impl UserOperations for AuthenticatedApplication {
    async fn get_auth_profile(&self) -> eyre::Result<StaffDto, ErrorMessage> {
        todo!()
    }

    async fn get_staff_details(&self, staff_id: &StaffId) -> eyre::Result<Option<StaffDto>, ErrorMessage> {
        todo!()
    }

    async fn search_staffs_directory(&self, query: Option<&str>) -> eyre::Result<Vec<StaffDto>, ErrorMessage> {
        todo!()
    }

    async fn search_departments(&self, query: Option<&str>) -> eyre::Result<Vec<DepartmentDto>, ErrorMessage> {
        todo!()
    }

    async fn get_department(&self, department_id: &DepartmentId) -> eyre::Result<Option<StaffDto>, ErrorMessage> {
        todo!()
    }
}