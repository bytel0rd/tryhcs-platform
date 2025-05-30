use either::Either;
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    boxed::Box,
    sync::{Arc, RwLock},
};

use crate::core::{
    ApplicationMode, GlobalApplication, GuestApplication, GENERAL_APP_INTERNAL_ERROR,
};
use tracing::*;
use tryhcs_shared::{
    api_params::{ErrorMessage, PaginatedQuery},
    institution_params::{LoginReq, VerifyOTP},
};
use ts_rs::TS;

use super::StateFeedBackTrait;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum UnAuthorizedStateAction {
    InitateLogin(LoginReq),
    VerifyLoginOTP(VerifyOTP),
}

#[derive(Serialize, Deserialize, Debug, Clone, TS, Default)]
#[ts(export)]
pub struct UnauthorizedState {
    pub login_state: LoginState,
}

pub type SessionId = String;

#[derive(Serialize, Deserialize, Debug, Clone, TS, Default)]
#[ts(export)]
pub struct LoginState {
    pub verify_otp_info: Option<(LoginReq, SessionId)>,
}

pub async fn unauthorized_state_machine<T: StateFeedBackTrait>(
    application: Arc<RwLock<GlobalApplication>>,
    action: &UnAuthorizedStateAction,
    feedback: T,
) -> eyre::Result<()> {
    match action {
        UnAuthorizedStateAction::InitateLogin(login_req) => {
            feedback.on_loading().await;
            match application.try_write() {
                Err(error_message) => {
                    error!("App lock failed: {:?}", error_message);
                    feedback.on_error(GENERAL_APP_INTERNAL_ERROR.into()).await;
                }
                Ok(mut app) => {
                    let current_app_mode = app.mode.clone();
                    if let ApplicationMode::Guest(op_mode) = current_app_mode {
                        let result = op_mode.login(&login_req).await;
                        match result {
                            Ok(result) => match result {
                                Either::Right(otp_session) => match app.state.write() {
                                    Err(error_message) => {
                                        error!("App state lock failed: {:?}", error_message);
                                        feedback.on_error(GENERAL_APP_INTERNAL_ERROR.into()).await;
                                    }

                                    Ok(mut app_state) => {
                                        app_state.guest_state.login_state.verify_otp_info =
                                            Some((login_req.clone(), otp_session.session_id));
                                        feedback.on_success(Box::new(login_req.clone())).await;
                                    }
                                },
                                Either::Left(authorized_app) => {
                                    app.mode = ApplicationMode::Authenticated(authorized_app);
                                }
                            },
                            Err(e) => {
                                feedback.on_error(e).await;
                            }
                        }
                    }
                }
            }
        }
        UnAuthorizedStateAction::VerifyLoginOTP(verify_otp) => {
            feedback.on_loading().await;
            match application.try_write() {
                Err(error_message) => {
                    error!("App lock failed: {:?}", error_message);
                    feedback.on_error(GENERAL_APP_INTERNAL_ERROR.into()).await;
                }
                Ok(mut app) => {
                    let current_app_mode = app.mode.clone();
                    if let ApplicationMode::Guest(op_mode) = current_app_mode {
                        let result = op_mode.verify_otp(&verify_otp).await;
                        match result {
                            Ok(authorized_app) => {
                                app.mode = ApplicationMode::Authenticated(authorized_app);
                                if let Ok(mut app_state) = app.state.write() {
                                    app_state.guest_state.login_state.verify_otp_info = None;
                                }
                            },
                            Err(e) => {
                                feedback.on_error(e).await;
                            }
                        }
                    }
                }
            }
        }
    };

    Ok(())
}
