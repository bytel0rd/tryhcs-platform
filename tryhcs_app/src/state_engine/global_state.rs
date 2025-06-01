use either::Either;
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    boxed::Box,
    sync::{Arc, RwLock},
};
use tryhcs_shared::{
    api_params::{ErrorMessage, PaginatedQuery},
    institution_params::{LoginReq, VerifyOTP},
};
use ts_rs::TS;

use super::unauthorized_states::UnauthorizedState;

#[derive(Serialize, Deserialize, Debug, Clone, TS, Default)]
#[ts(export)]
pub struct GlobalState {
    pub guest_state: UnauthorizedState,
}
