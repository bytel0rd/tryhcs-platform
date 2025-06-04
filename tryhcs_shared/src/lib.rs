pub mod api_params;
pub mod compliance_params;
pub mod encryption;
pub mod finance_params;
pub mod institution_params;
pub mod records_param;

use ts_rs::TS;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct APIFileUpload {
    pub service: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub base64_data: String,
    pub link_expires_duration: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct APIFileUploadResponse {
    pub file_key: String,
    pub file_non_perment_link: Option<String>,
}
