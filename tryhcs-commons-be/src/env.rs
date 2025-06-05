use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EnvConfig {
    pub database_url: String,
    pub redis_url: String,

    pub session_expires_in_min: u32,

    pub otp_length: u8,
    pub otp_expires_in_sec: u8,
    pub max_failed_attempts: i32,
    pub min_password_length: u32,

    pub presigned_url_expires_in_sec: u64,

    pub gemini_api_key: String,

    pub cloudflare_r2_url: String,
    pub cloudflare_r2_api_token: String,
    pub cloudflare_r2_access_key_id: String,
    pub cloudflare_r2_secret_access_key: String,
    pub cloudflare_r2_bucket: String,

    pub sendchamp_base_url: String,
    pub sendchamp_api_key: String,
    pub sendchamp_sender_id: String,

    pub smtp_server: String,
    pub smtp_port: String,
    pub no_reply_email_address: String,
    pub no_reply_email_password: String,
    pub app_url: String,

    pub youverify_base_url: String,
    pub youverify_api_key: String,

    pub banks_cache_expires_in_hr: u64,
}
