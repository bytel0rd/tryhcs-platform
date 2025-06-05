use std::sync::Arc;
use std::sync::Once;

use either::Either;
use tracing::*;
use tracing_subscriber::util::SubscriberInitExt;

use tryhcs_app::core::create_app_core;
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
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_test_writer() // optional, routes to test framework output
            .try_init();
    });

    HcsAppConfig::new("https://hcs-demo-api.blueandgreen.ng".into())
        .set_storage(AppStorage::InMemory)
        .set_encryption(AppEncryption::NoEncryption)
        .set_req_timeout_in_sec(10)
}

fn create_test_app() -> GuestApplication {
    let config = create_app_config();
    let core: tryhcs_app::core::CoreApplication =
        create_app_core(config, Arc::new(None)).expect("Failed to create core application");
    GuestApplication::new(core)
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

#[tokio::test]
async fn get_user_auth_profile_successfully() {
    let app = create_test_app();

    let login_req = LoginReq {
        phone_number: "+2348149464288".into(),
        password: "Password1!".into(),
        device_id: "device_12345".into(),
    };
    let response = app.login(&login_req).await.expect("login failed");

    if let Either::Right(verify_otp) = &response {
        app.verify_otp(&VerifyOTP {
            otp_code: "12345".into(),
            session_id: verify_otp.session_id.clone(),
        })
        .await
        .expect("Otp verification failed");
    }

    if let Either::Left(app) = &response {
        let user = app
            .get_auth_profile()
            .await
            .expect("failed to get auth profile");
        info!("user profile: {}", serde_json::to_string(&user).unwrap());
    }
}

#[tokio::test]
async fn test_logging_basics() {
    println!("🚧 println works?");
    tracing::info!("🚧 tracing works?");
}
