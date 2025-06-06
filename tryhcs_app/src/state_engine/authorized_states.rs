use either::Either;

use serde::{Deserialize, Serialize};
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tryhcs_shared::api_params::{ErrorMessage, PaginatedQuery};
use tryhcs_shared::institution_params::*;
use tryhcs_shared::records_param::*;
use ts_rs::TS;

use crate::core::{ApplicationMode, GlobalApplication};

use super::StateFeedBackTrait;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum AuthorizedStateAction {
    GetAuthProfile,
    GetWorkspaceProfile,
    FindStaffs(Option<String>, PaginatedQuery),
    GetStaffById(StaffShadowId),
    FindDepartments(Option<String>, PaginatedQuery),
    CreateDepartment(CreateDepartment),
    EditDepartment(DepartmentShadowId, CreateDepartment),
    DeleteDepartment(DepartmentShadowId),
    AddStaff(NewStaff),
    EditStaff(StaffShadowId, NewStaff),
    DeleteStaff(StaffShadowId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthorizedState {
    pub user: AuthorizedUser,
}

pub async fn authorized_state_machine(
    app: &mut GlobalApplication,
    action: AuthorizedStateAction,
    feedback: &Box<dyn StateFeedBackTrait>,
) -> eyre::Result<(), ErrorMessage> {
    match action {
        AuthorizedStateAction::GetAuthProfile => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.get_auth_profile().await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::GetWorkspaceProfile => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.get_user_workpace_profile().await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::FindStaffs(search, _) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode
                    .search_staffs_directory(search.as_ref().map(|x| x.as_str()))
                    .await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::GetStaffById(staff_shadow_id) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.get_staff_details(&staff_shadow_id).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::FindDepartments(search, _) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode
                    .search_departments(search.as_ref().map(|x| x.as_str()))
                    .await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::CreateDepartment(create_department) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.create_department(&create_department).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::EditDepartment(department_shadow_id, create_department) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode
                    .edit_department(&department_shadow_id, &create_department)
                    .await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::DeleteDepartment(department_shadow_id) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.delete_department(&department_shadow_id).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::AddStaff(new_staff) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.add_staff(&new_staff).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::EditStaff(staff_shadow_id, new_staff) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.edit_staff(&staff_shadow_id, &new_staff).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
        AuthorizedStateAction::DeleteStaff(staff_shadow_id) => {
            feedback.on_loading().await;
            if let ApplicationMode::Authenticated(op_mode) = &app.mode {
                let response = op_mode.delete_staff(&staff_shadow_id).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
    };

    Ok(())
}
