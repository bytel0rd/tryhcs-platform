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
    institution_params::{CreateInstitution, LoginReq, VerifyOTP},
};
use ts_rs::TS;

use super::StateFeedBackTrait;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum UnAuthorizedStateAction {
    InitateLogin(LoginReq),
    VerifyLoginOTP(VerifyOTP),
    InitateRegistration(CreateInstitution),
    CompleteRegistration(VerifyOTP),
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

pub async fn unauthorized_state_machine(
    app: &mut GlobalApplication,
    action: &UnAuthorizedStateAction,
    feedback: &Box<dyn StateFeedBackTrait>,
) -> eyre::Result<(), ErrorMessage> {
    match action {
        UnAuthorizedStateAction::InitateLogin(login_req) => {
            feedback.on_loading().await;
            let current_app_mode = app.mode.clone();
            if let ApplicationMode::Guest(op_mode) = current_app_mode {
                let result = op_mode.login_initate(&login_req).await?;
                match result {
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
                }
            }
        }
        UnAuthorizedStateAction::VerifyLoginOTP(verify_otp) => {
            feedback.on_loading().await;
            let current_app_mode = app.mode.clone();
            if let ApplicationMode::Guest(op_mode) = current_app_mode {
                let authorized_app = op_mode.login_complete(&verify_otp).await?;
                app.mode = ApplicationMode::Authenticated(authorized_app);
                if let Ok(mut app_state) = app.state.write() {
                    app_state.guest_state.login_state.verify_otp_info = None;
                }
            }
        }
        UnAuthorizedStateAction::InitateRegistration(req) => {
            feedback.on_loading().await;
            if let ApplicationMode::Guest(op_mode) = &app.mode {
                let response = op_mode.register_initate(&req).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }

        UnAuthorizedStateAction::CompleteRegistration(verify_otp) => {
            feedback.on_loading().await;
            if let ApplicationMode::Guest(op_mode) = &app.mode {
                let response = op_mode.register_complete(&verify_otp).await?;
                feedback.on_success(Box::new(response)).await;
            }
        }
    };

    Ok(())
}
