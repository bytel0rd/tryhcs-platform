pub mod authorized_states;
pub mod global_state;
pub mod unauthorized_states;

use authorized_states::AuthorizedStateAction;
use unauthorized_states::UnAuthorizedStateAction;

use either::Either;
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::boxed::Box;
use tryhcs_shared::{
    api_params::{ErrorMessage, PaginatedQuery},
    institution_params::{LoginReq, VerifyOTP},
};
use ts_rs::TS;

#[async_trait::async_trait(?Send)]
pub trait StateFeedBackTrait {
    async fn on_loading(&self);
    async fn on_success(&self, data: Box<dyn erased_serde::Serialize>);
    async fn on_error(&self, error: ErrorMessage);
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum StateAction {
    UnAuthorized(UnAuthorizedStateAction),
    Authorized(AuthorizedStateAction),
}

pub async fn state_machine(action: StateAction, feedback: Box<dyn StateFeedBackTrait>) {}
