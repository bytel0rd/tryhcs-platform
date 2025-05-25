use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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