use std::sync::Arc;

use either::Either;
use tryhcs_shared::{
    api_params::ErrorMessage,
    institution_params::{
        AuthenticatedUser, AuthorizedUser, DepartmentDto, DepartmentId, LoginReq, LoginResponse,
        StaffDto, StaffId, VerifyOTP,
    },
};

#[async_trait::async_trait(?Send)]
pub trait HcsEndpoints: Send + Sync {
    async fn is_online(&self) -> bool;

    async fn login(&self, login_req: &LoginReq) -> eyre::Result<LoginResponse, ErrorMessage>;
    async fn verify_otp(
        &self,
        verify_otp: &VerifyOTP,
    ) -> eyre::Result<AuthenticatedUser, ErrorMessage>;

    async fn get_auth_profile(&self) -> eyre::Result<StaffDto, ErrorMessage>;

    async fn get_staff_details(&self, staff_id: &StaffId) -> eyre::Result<StaffDto, ErrorMessage>;
    async fn search_staffs_directory(&self) -> eyre::Result<Vec<StaffDto>, ErrorMessage>;

    async fn search_departments(&self) -> eyre::Result<Vec<DepartmentDto>, ErrorMessage>;
    // async fn get_department(
    //     &self,
    //     department_id: &DepartmentId,
    // ) -> eyre::Result<DepartmentDto, ErrorMessage>;
}
