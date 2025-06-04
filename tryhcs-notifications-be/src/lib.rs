pub mod integrations;

pub use integrations::smtp_email::*;

use serde::{Deserialize, Serialize};
use tryhcs_commons_be::env::EnvConfig;

use crate::integrations::sendchamp::SendchampApi;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NotificationChannel {
    Email(String),
    Mobile(String),
}

pub async fn send_sms(env: &EnvConfig, mobile: &str, message: &str) -> eyre::Result<()> {
    let api = SendchampApi {
        base_url: &env.sendchamp_base_url,
        api_key: &env.sendchamp_api_key,
        sendchamp_sender_id: &env.sendchamp_sender_id,
    };
    if let Err(err) = api.send_message(mobile, message).await {
        tracing::error!(
            "Error sending SMS to mobile={}, message={}, err={:?}",
            mobile,
            message,
            err
        );
    }

    Ok(())
}
