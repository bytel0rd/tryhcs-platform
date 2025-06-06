pub mod authorized_states;
pub mod global_state;
pub mod unauthorized_states;

use authorized_states::AuthorizedStateAction;
use unauthorized_states::UnAuthorizedStateAction;

use either::Either;
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{boxed::Box, sync::{Arc, RwLock}};
use tryhcs_shared::{
    api_params::{ErrorMessage, PaginatedQuery},
    institution_params::{LoginReq, VerifyOTP},
};
use ts_rs::TS;

use crate::{core::GlobalApplication, state_engine::unauthorized_states::unauthorized_state_machine};

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


pub async fn state_machine(app: &mut GlobalApplication, action: StateAction, feedback: Box<dyn StateFeedBackTrait>) {
    match action {
        StateAction::UnAuthorized(un_authorized_state_action) => {
            if let Err(err) = unauthorized_state_machine(app, &un_authorized_state_action, &feedback).await  {
                feedback.on_error(err).await;
            }
        },
        StateAction::Authorized(authorized_state_action) => todo!(),
    }
}
