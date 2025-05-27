use std::sync::Arc;
use std::sync::Once;

use either::Either;
use tracing::*;
use tracing_subscriber::util::SubscriberInitExt;

use tryhcs_app::core::AppEncryption;
use tryhcs_app::core::GuestApplication;
use tryhcs_app::core::HcsAppConfig;
use tryhcs_app::storage::AppStorage;
use tryhcs_shared::{
    encryption::NoEncryption,
    institution_params::{LoginReq, VerifyOTP},
};

static INIT: Once = Once::new();

fn create_app_config() -> HcsAppConfig {
    pub fn init_tracing() {
        INIT.call_once(|| {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
                .init();
        });
    }

    HcsAppConfig {
        base_api_url: "https://hcs-demo-api.blueandgreen.ng".into(),
        debug_enabled: true,
        encryption_mode: AppEncryption::NoEncryption,
        storage: AppStorage::InMemory,
    }
}

fn create_test_app() -> GuestApplication {
    GuestApplication::new(&create_app_config()).expect("Failed to create guest application")
}

#[tokio::test]
async fn initate_login_invalid_credentials() {
    let app = create_test_app();

    let login_req = LoginReq {
        phone_number: "+2348149464288".into(),
        password: "Password!".into(),
        device_id: "device1234".into(),
    };
    let response = app.login(&login_req).await;
    if let Err(err) = response {
        error!(message="Login failed response", err=?err);
        return;
    }

    panic!("Login successful")
}

#[tokio::test]
async fn initate_login_successfully() {
    let app = create_test_app();

    let login_req = LoginReq {
        phone_number: "+2348149464288".into(),
        password: "Password1!".into(),
        device_id: "device_12345".into(),
    };
    let response = app.login(&login_req).await.expect("login failed");

    if let Either::Right(verify_otp) = response {
        app.verify_otp(&VerifyOTP {
            otp_code: "12345".into(),
            session_id: verify_otp.session_id,
        })
        .await
        .expect("Otp verification failed");
    }
}
