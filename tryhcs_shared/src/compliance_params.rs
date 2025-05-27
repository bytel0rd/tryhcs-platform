use bon::Builder;
use derive_more::{Display, FromStr};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComplianceEvaluation {
    pub compliance_status: ComplianceStatus,
    pub compliance_message: String,
    pub rejected: Vec<String>,
}

#[derive(Debug, Clone, Display, FromStr, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ComplianceStatus {
    VERIFIED,
    PENDING,
    REJECTED,
    SUBMITTED,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComplianceResponse {
    pub corporate: Option<CorporateComplianceDto>,
    pub financial: Option<FinancialComplianceDto>,
    pub healthcare: Option<HealthcareComplianceDto>,
    pub evaluation: ComplianceEvaluation,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct CorporateComplianceDto {
    pub rc_no: String,
    pub tin: String,
    pub corporate_account_number: String,
    pub corporate_bank_code: String,
    pub private_healthcare_certificate_url: Option<String>,
    pub stage: String,
}

#[derive(Serialize, Deserialize, Debug, Builder, Clone, TS)]
#[ts(export)]
pub struct HealthcareComplianceDto {
    pub licensed_medical_doctor_name: String,
    pub licensed_medical_doctor_mdcn_no: String,
    pub licensed_medical_doctor_mdcn_speciality: String,
    pub licensed_medical_doctor_mdcn_image_url: String,
    pub licensed_medical_doctor_email: String,
    pub licensed_medical_doctor_phone_no: String,
    pub stage: String,
}

#[derive(Serialize, Deserialize, Debug, Builder, Clone, TS)]
#[ts(export)]
pub struct FinancialComplianceDto {
    pub director_legal_name: String,
    pub director_legal_bvn: String,
    pub director_legal_dob: String,
    pub director_legal_gov_id_type: String,
    pub director_legal_gov_id_url: String,
    pub stage: String,
}
