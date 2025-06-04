use bon::Builder;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum RecordTypes {
    PatientComplain(String),
    Prescription {
        name: String,
        dosage: Option<String>,
        time: Option<String>,
    },
    Examination {
        method: Option<String>,
        observation: String,
    },
    LabTestRequest {
        name: String,
    },
    LabTestResult {
        name: String,
        results: String,
        code: Option<String>,
    },
    WardAdmission {
        ward_name: String,
        bed_number: String,
    },
    WardDischarge {
        reason: String,
    },
    WardTransfer {
        ward_name: String,
        bed_number: String,
        reason: Option<String>,
    },
    PatientAppointment {
        schedule: String,
        reason: Option<String>,
    },
    EmergencyReferrer {
        reason: Option<String>,
        destination: Option<String>,
        date_time: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default, TS)]
#[ts(export)]
pub struct StaffPatientData {
    pub fullname: String,
    pub age: Option<String>,
    pub card_no: Option<String>,
    pub last_entry_date: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, TS)]
#[ts(export)]
pub struct PatientHistoryDto {
    pub id: i64,
    pub card_no: Option<String>,
    pub raw_text: Option<String>,
    pub files: Vec<Value>,
    pub version: i32,
    pub is_extracted: bool,
    pub records: Vec<Value>,
    pub metadata: Value,
    pub staff_name: Option<String>,
    pub staff_profile_image_url: Option<String>,
    pub institution_name: String,
    pub title: String,
    pub institution_profile_image_url: Option<String>,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, TS)]
#[ts(export)]
pub struct UploadedFile {
    pub base64: String,
    pub content_type: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder, TS)]
#[ts(export)]
pub struct AddPatientRecordReq {
    pub files: Vec<UploadedFile>,
    pub text: Option<String>,
}
