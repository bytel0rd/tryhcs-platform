use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tryhcs_shared::compliance_params::{CorporateComplianceDto, FinancialComplianceDto, HealthcareComplianceDto};
use uuid::Uuid;
// use tryhcs_derive::{declare_db_columns, query_many, query_one};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CorporateComplianceId(pub i64);

// #[declare_db_columns]
#[derive(Serialize, Deserialize, Debug, Builder, sqlx::FromRow, Clone)]
pub struct CorporateCompliance {
    pub id: i64,
    pub institution_id: i64,
    pub rc_no: String,
    pub tin: String,
    pub corporate_account_number: String,
    pub corporate_bank_code: String,
    pub private_healthcare_certificate_url: Option<String>,
    
    pub stage: String,
    pub created_by: String,

    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<CorporateCompliance> for CorporateComplianceDto  {
    fn from(value: CorporateCompliance) -> Self {
        CorporateComplianceDto {
            rc_no: value.rc_no,
            tin: value.tin,
            corporate_account_number: value.corporate_account_number,
            corporate_bank_code: value.corporate_bank_code,
            private_healthcare_certificate_url: value.private_healthcare_certificate_url,
            stage: value.stage,
        }
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthcareComplianceId(pub i64);

#[derive(Serialize, Deserialize, Debug, Builder, sqlx::FromRow, Clone)]
pub struct HealthcareCompliance {
    pub id: i64,
    pub institution_id: i64,
    pub licensed_medical_doctor_name: String,
    pub licensed_medical_doctor_mdcn_no: String,
    pub licensed_medical_doctor_mdcn_speciality: String,
    pub licensed_medical_doctor_mdcn_image_url: String,
    pub licensed_medical_doctor_email: String,
    pub licensed_medical_doctor_phone_no: String,

    pub stage: String,
    pub created_by: String,

    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<HealthcareCompliance> for HealthcareComplianceDto  {
    fn from(value: HealthcareCompliance) -> Self {
        HealthcareComplianceDto {
            licensed_medical_doctor_name: value.licensed_medical_doctor_name,
            licensed_medical_doctor_mdcn_no: value.licensed_medical_doctor_mdcn_no,
            licensed_medical_doctor_mdcn_speciality: value.licensed_medical_doctor_mdcn_speciality,
            licensed_medical_doctor_mdcn_image_url: value.licensed_medical_doctor_mdcn_image_url,
            licensed_medical_doctor_email: value.licensed_medical_doctor_email,
            licensed_medical_doctor_phone_no: value.licensed_medical_doctor_phone_no,
            stage: value.stage,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinancialComplianceId(pub i64);

// #[declare_db_columns]
#[derive(Serialize, Deserialize, Debug, Builder, sqlx::FromRow, Clone)]
pub struct FinancialCompliance {
    pub id: i64,
    pub institution_id: i64,

    pub director_legal_name: String,
    pub director_legal_bvn: String,
    pub director_legal_dob: String,
    pub director_legal_gov_id_type: String,
    pub director_legal_gov_id_url: String,

    pub stage: String,
    pub created_by: String,

    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<FinancialCompliance> for FinancialComplianceDto  {
    fn from(value: FinancialCompliance) -> Self {
        FinancialComplianceDto {
            director_legal_name: value.director_legal_name,
            director_legal_bvn: value.director_legal_bvn,
            director_legal_dob: value.director_legal_dob,
            director_legal_gov_id_type: value.director_legal_gov_id_type,
            director_legal_gov_id_url: value.director_legal_gov_id_url,
            stage: value.stage,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaffComplianceId(pub i64);

// #[declare_db_columns]
#[derive(Serialize, Deserialize, Debug, Builder, sqlx::FromRow, Clone)]
pub struct StaffCompliance {
    pub id: i64,
    pub staff_id: i64,

    pub license_type: String,
    pub license_no: Option<String>,
    pub license_certificate_url: Option<String>,

    pub stage: String,
    pub created_by: String,

    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}