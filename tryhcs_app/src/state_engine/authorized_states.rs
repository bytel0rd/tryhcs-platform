use either::Either;

use serde::{Deserialize, Serialize};
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tryhcs_shared::api_params::{ErrorMessage, PaginatedQuery};
use tryhcs_shared::institution_params::*;
use tryhcs_shared::records_param::*;
use ts_rs::TS;

use crate::core::GlobalApplication;

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
    pub user: AuthorizedUser,
}

pub async fn authorized_state_machine<T: StateFeedBackTrait>(
    action: AuthorizedStateAction,
    application: Arc<RwLock<GlobalApplication>>,
    feedback: T,
) {
    match action {
        AuthorizedStateAction::GetAuthProfile => {
            feedback.on_loading().await;
            // match feedback.get_auth_profile().await {
            //     Ok(profile) => {
            //         feedback.on_success(profile).await;
            //     }
            //     Err(error_message) => {
            //         feedback.on_error(error_message).await;
            //     }
            // }
        }
        _ => {}
    }
}
