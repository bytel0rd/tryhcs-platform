use either::Either;

use serde::{Deserialize, Serialize};
use std::boxed::Box;
use std::collections::HashMap;
use tryhcs_shared::api_params::{ErrorMessage, PaginatedQuery};
use tryhcs_shared::institution_params::*;
use tryhcs_shared::records_param::*;
use ts_rs::TS;

use super::StateFeedBackTrait;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum AuthorizedStateAction {
    GetAuthProfile,
    GetStaffs(PaginatedQuery),
    GetDepartments(PaginatedQuery),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorizedState {
    pub institution: InstitutionDto,
    pub departments: Option<Vec<DepartmentDto>>,
    pub staffs: Option<Vec<StaffDto>>,
    pub user: AuthorizedUser,
    pub recent_patients: HashMap<i64, (StaffPatientData, Vec<PatientHistoryDto>)>,
}

pub async fn authorized_state_machine<T: StateFeedBackTrait>(
    action: AuthorizedStateAction,
    feedback: T,
) {
}
