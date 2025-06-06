use std::sync::Arc;

use either::Either;
use tryhcs_shared::{
    api_params::ErrorMessage,
    institution_params::{
        AuthenticatedUser, AuthorizedUser, CreateDepartment, CreateInstitution, DepartmentAndStaffDto, DepartmentDto, DepartmentId, DepartmentShadowId, InitiatedOtp, InstitutionDto, LoginReq, LoginResponse, NewStaff, StaffDto, StaffId, StaffShadowId, VerifyOTP
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

    async fn get_auth_profile(&self) -> eyre::Result<AuthorizedUser, ErrorMessage>;

    async fn get_staff_details(
        &self,
        staff_id: &StaffShadowId,
    ) -> eyre::Result<StaffDto, ErrorMessage>;
    async fn search_staffs_directory(&self) -> eyre::Result<Vec<StaffDto>, ErrorMessage>;

    async fn search_departments(&self) -> eyre::Result<Vec<DepartmentDto>, ErrorMessage>;

    async fn initiate_registration(&self, req: &CreateInstitution) -> eyre::Result<InitiatedOtp, ErrorMessage>;
    async fn complete_registration(
        &self,
        verify_otp: &VerifyOTP,
    ) -> eyre::Result<InstitutionDto, ErrorMessage>;

    async fn get_departments(&self, id: &DepartmentShadowId) -> eyre::Result<DepartmentAndStaffDto, ErrorMessage>;
    async fn create_departments(&self, req: &CreateDepartment) -> eyre::Result<DepartmentDto, ErrorMessage>;
    async fn edit_departments(&self, id: &DepartmentShadowId, req: &CreateDepartment) -> eyre::Result<DepartmentDto, ErrorMessage>;
    async fn delete_departments(&self, id: &DepartmentShadowId) -> eyre::Result<(), ErrorMessage>;


    async fn add_staff(
        &self,
        req: &NewStaff,
    ) -> eyre::Result<StaffDto, ErrorMessage>;

    async fn edit_staff(
        &self,
        staff_id: &StaffShadowId,
        req: &NewStaff,
    ) -> eyre::Result<StaffDto, ErrorMessage>;


    async fn delete_staff(
        &self,
        staff_id: &StaffShadowId,
    ) -> eyre::Result<(), ErrorMessage>;

    // async fn get_department(
    //     &self,
    //     department_id: &DepartmentId,
    // ) -> eyre::Result<DepartmentDto, ErrorMessage>;
}
