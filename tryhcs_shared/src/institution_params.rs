use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct InstitutionDto {
    pub id: String,
    pub px: i64,
    pub institution_name: String,
    pub email: String,
    pub classification: String,
    pub workspace_code: String,
    pub setting: String,
    pub address: Option<String>,
    pub town: Option<String>,
    pub state: Option<String>,
    pub logo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct AuthenticatedUser {
    pub principal: AuthorizedUser,
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, TS)]
#[ts(export)]
pub struct AuthorizedUser {
    pub mobile: String,
    pub accounts: Vec<AuthorizedInstitutionUser>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, TS)]
#[ts(export)]
pub struct AuthorizedInstitutionUser {
    pub staff_id: String,
    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub profile_image_url: Option<String>,
    pub departments: Vec<DepartmentDto>,
    pub institution: InstitutionDto,
}

impl AuthorizedInstitutionUser {
    pub fn is_workspace_admin(&self) -> bool {
        false
        // self.departments.iter().any(|d| ADMIN_DEPT.eq_ignore_ascii_case(&d.name))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub struct InstitutionId(pub i64);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub struct StaffId(pub i64);

#[derive(Serialize, Deserialize, Debug, Builder, Clone, TS)]
#[ts(export)]
pub struct StaffDto {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub institution_id: Option<i64>,
    pub profile_image: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, TS)]
#[ts(export)]
pub struct DepartmentDto {
    pub id: String,
    pub name: String,
    pub institution_id: i64,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct VerifyOTP {
    pub otp_code: String,
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, TS)]
#[ts(export)]
pub struct InitiatedOtp {
    pub session_id: String,
    pub duration: u64,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, TS)]
#[ts(export)]
pub struct DepartmentAndStaffDto {
    pub department: DepartmentDto,
    pub staffs: Vec<StaffDto>,
    pub department_head: Option<StaffDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct LoginReq {
    pub phone_number: String,
    pub password: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, TS)]
#[ts(export)]
pub struct LoginResponse {
    pub otp: Option<InitiatedOtp>,
    pub auth: Option<AuthenticatedUser>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub struct DepartmentId(pub i64);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum BasePermission {
    Create,
    View,
    Edit,
    Delete,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum FinancialPermission {
    ViewReports,
    Withdraw,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum PermittedAction {
    MedicalHistory(BasePermission),
    Labouratory(BasePermission),
    Billing(BasePermission),
    InstitutionSetting(BasePermission),
    PersonnelManagement(BasePermission),
    DepartmentManagement(BasePermission),
    FinancialManagement(FinancialPermission),
}

#[derive(Serialize, Deserialize, Debug, Builder, Clone, TS)]
#[ts(export)]
pub struct DepartmentMember {
    pub staff_id: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, TS)]
#[ts(export)]
pub struct CreateDepartment {
    pub name: String,
    pub domain: String,
    pub head_staff_id: Option<String>,
    pub phone_no: Option<String>,
    pub staff_ids: Vec<DepartmentMember>,
}

#[derive(Serialize, Deserialize, Debug,TS)]
#[ts(export)]
pub struct NewStaff {
    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub profile_image: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Builder, TS)]
#[ts(export)]
pub struct CreateInstitution {
    pub institution_name: String,
    pub email: String,
    pub classification: String,
    pub setting: String,
    pub address: Option<String>,
    pub town: Option<String>,
    pub state: Option<String>,

    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub password: String,

    pub logo: Option<String>,
}
