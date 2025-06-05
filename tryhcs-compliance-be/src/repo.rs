use async_trait::async_trait;
use derive_more::{Display, FromStr};
use eyre::Context;
use sqlx::{query_as, PgPool};
use tryhcs_commons_be::data_encryption::Encryptor;
use tryhcs_shared::{compliance_params::{ComplianceStatus, CorporateComplianceEdit, FinancialComplianceEdit, HealthcareComplianceEdit, NewComplainceEdit, NewHealthcareComplainceEdit, NewinancialComplainceEdit}, institution_params::InstitutionId};
use super::models::*;

use serde::{Deserialize, Serialize};
#[cfg(test)]
use mockall::{automock, mock, predicate::*};



#[cfg_attr(test, automock)]
#[async_trait]
pub trait ComplianceRepo: Send + Sync {
    async fn create_corporate_compliance(&self, institution_id: &InstitutionId, data: &NewComplainceEdit) -> eyre::Result<CorporateCompliance>;
    async fn edit_corporate_compliance(&self, id: &CorporateComplianceId, data: &CorporateComplianceEdit) -> eyre::Result<CorporateCompliance>;
    async fn update_corporate_compliance_status(&self, institution_id: &InstitutionId, status: &ComplianceStatus) -> eyre::Result<CorporateCompliance>;
    async fn get_corporate_compliance(&self, institution_id: &InstitutionId) -> eyre::Result<Option<CorporateCompliance>>;

    async fn create_healthcare_compliance(&self, institution_id: &InstitutionId, data: &NewHealthcareComplainceEdit) -> eyre::Result<HealthcareCompliance>;
    async fn edit_healthcare_compliance(&self, id: &HealthcareComplianceId, data: &HealthcareComplianceEdit) -> eyre::Result<HealthcareCompliance>;
    async fn update_healthcare_compliance_status(&self, institution_id: &InstitutionId, status: &ComplianceStatus) -> eyre::Result<HealthcareCompliance>;
    async fn get_healthcare_compliance(&self, institution_id: &InstitutionId) -> eyre::Result<Option<HealthcareCompliance>>;

    async fn create_financial_compliance(&self, institution_id: &InstitutionId, data: &NewinancialComplainceEdit) -> eyre::Result<FinancialCompliance>;
    async fn edit_financial_compliance(&self, id: &FinancialComplianceId, data: &FinancialComplianceEdit) -> eyre::Result<FinancialCompliance>;
    async fn get_financial_compliance(&self, institution_id: &InstitutionId) -> eyre::Result<Option<FinancialCompliance>>;
    async fn update_financial_compliance_status(&self, institution_id: &InstitutionId, status: &ComplianceStatus) -> eyre::Result<FinancialCompliance>;
    
}

#[derive(Clone)]
pub struct ComplianceDB {
    pub customer_db: PgPool,
    // global_encryptor: Encryptor,
}

#[async_trait]
impl ComplianceRepo for ComplianceDB {
    async fn create_corporate_compliance(&self, institution_id: &InstitutionId, NewComplainceEdit(staff_id,  data): &NewComplainceEdit) -> eyre::Result<CorporateCompliance> {
        query_as!(
            CorporateCompliance,
            "insert into corporate_compliance
            (institution_id, rc_no, tin, private_healthcare_certificate_url, corporate_account_number,corporate_bank_code, created_by  )
        values
            ($1, $2, $3, $4, $5, $6, $7)
        returning id, institution_id, rc_no, tin, private_healthcare_certificate_url, corporate_account_number,corporate_bank_code, created_by, stage, created_at, modified_at, deleted_at ",
        institution_id.0,
            data.rc_no,
            data.tin,
            data.private_healthcare_certificate_url,
            data.corporate_account_number,
            data.corporate_bank_code,
            staff_id.0,
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error creating corporate compliance")
    }

    async fn edit_corporate_compliance(&self, id: &CorporateComplianceId, data: &CorporateComplianceEdit) -> eyre::Result<CorporateCompliance> {
        query_as!(
            CorporateCompliance,
            "update corporate_compliance set
                rc_no = $2, tin=$3, private_healthcare_certificate_url=$4,
                corporate_account_number=$5, corporate_bank_code=$6,
                stage = 'PENDING' 
            where id = $1
            returning  id, institution_id, rc_no, tin, private_healthcare_certificate_url, corporate_account_number,corporate_bank_code, created_by, stage, created_at, modified_at, deleted_at 
        ",
        id.0,
            data.rc_no,
            data.tin,
            data.private_healthcare_certificate_url,
            data.corporate_account_number,
            data.corporate_bank_code
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error updating corporate compliance")
    }

    async fn update_corporate_compliance_status(&self, institution_id: &InstitutionId, status: &ComplianceStatus) -> eyre::Result<CorporateCompliance> {
        query_as!(
            CorporateCompliance,
            "update corporate_compliance set 
                    stage = $2 
                where institution_id = $1
            returning id, institution_id, rc_no, tin, private_healthcare_certificate_url, corporate_account_number,corporate_bank_code, created_by, stage, created_at, modified_at, deleted_at 
        ",
        institution_id.0,
            &status.to_string()
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error updating corporate compliance status")
    }


    async fn get_corporate_compliance(&self, institution_id: &InstitutionId) -> eyre::Result<Option<CorporateCompliance>> {
        query_as!(
            CorporateCompliance,
            "select id, institution_id, rc_no, tin, private_healthcare_certificate_url, corporate_account_number,corporate_bank_code, created_by, stage, created_at, modified_at, deleted_at from corporate_compliance where institution_id = $1",
        institution_id.0
        )
        .fetch_optional(&self.customer_db)
        .await
        .wrap_err("Error fetching corporate compliance")
    }

    async fn create_healthcare_compliance(&self, institution_id: &InstitutionId, NewHealthcareComplainceEdit(staff_id, data): &NewHealthcareComplainceEdit) -> eyre::Result<HealthcareCompliance> {
        query_as!(
            HealthcareCompliance,
            "insert into healthcare_compliance
            (institution_id, licensed_medical_doctor_name, licensed_medical_doctor_mdcn_no,
             licensed_medical_doctor_mdcn_speciality, licensed_medical_doctor_mdcn_image_url,
               licensed_medical_doctor_email, licensed_medical_doctor_phone_no,  created_by)
        values
            ($1, $2, $3, $4, $5, $6, $7, $8)
        returning id, institution_id, licensed_medical_doctor_name, licensed_medical_doctor_mdcn_no,
             licensed_medical_doctor_mdcn_speciality, licensed_medical_doctor_mdcn_image_url,
               licensed_medical_doctor_email, licensed_medical_doctor_phone_no,  created_by,
               stage, created_at, modified_at, deleted_at 
               ",
        institution_id.0,
            data.licensed_medical_doctor_name,
            data.licensed_medical_doctor_mdcn_no,
            data.licensed_medical_doctor_mdcn_speciality,
            data.licensed_medical_doctor_mdcn_image_url,
            data.licensed_medical_doctor_email,
            data.licensed_medical_doctor_phone_no,
            staff_id.0
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error creating healthcare compliance")
    }

    async fn edit_healthcare_compliance(&self, id: &HealthcareComplianceId, data: &HealthcareComplianceEdit) -> eyre::Result<HealthcareCompliance> {
        query_as!(
            HealthcareCompliance,
            "update healthcare_compliance set
                licensed_medical_doctor_name=$2, licensed_medical_doctor_mdcn_no=$3,
                licensed_medical_doctor_mdcn_speciality=$4, licensed_medical_doctor_mdcn_image_url=$5,
                licensed_medical_doctor_email=$6, licensed_medical_doctor_phone_no=$7,
                stage = 'PENDING'
            where id = $1
            returning id, institution_id, licensed_medical_doctor_name, licensed_medical_doctor_mdcn_no,
             licensed_medical_doctor_mdcn_speciality, licensed_medical_doctor_mdcn_image_url,
               licensed_medical_doctor_email, licensed_medical_doctor_phone_no,  created_by,
               stage, created_at, modified_at, deleted_at",
        id.0,
            data.licensed_medical_doctor_name,
            data.licensed_medical_doctor_mdcn_no,
            data.licensed_medical_doctor_mdcn_speciality,
            data.licensed_medical_doctor_mdcn_image_url,
            data.licensed_medical_doctor_email,
            data.licensed_medical_doctor_phone_no,
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error updating healthcare compliance")
    }


    async fn update_healthcare_compliance_status(&self, institution_id: &InstitutionId, status: &ComplianceStatus) -> eyre::Result<HealthcareCompliance> {
        query_as!(
            HealthcareCompliance,
            "update healthcare_compliance set 
                    stage = $2 
                where institution_id = $1
            returning id, institution_id, licensed_medical_doctor_name, licensed_medical_doctor_mdcn_no,
             licensed_medical_doctor_mdcn_speciality, licensed_medical_doctor_mdcn_image_url,
               licensed_medical_doctor_email, licensed_medical_doctor_phone_no,  created_by,
               stage, created_at, modified_at, deleted_at 
        ",
        institution_id.0,
            &status.to_string()
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error updating healthcare compliance status")
    }

    async fn get_healthcare_compliance(&self, institution_id: &InstitutionId) -> eyre::Result<Option<HealthcareCompliance>> {
        query_as!(
            HealthcareCompliance,
            "select id, institution_id, licensed_medical_doctor_name, licensed_medical_doctor_mdcn_no,
             licensed_medical_doctor_mdcn_speciality, licensed_medical_doctor_mdcn_image_url,
               licensed_medical_doctor_email, licensed_medical_doctor_phone_no,  created_by,
               stage, created_at, modified_at, deleted_at from healthcare_compliance where institution_id = $1",
        institution_id.0
        )
        .fetch_optional(&self.customer_db)
        .await
        .wrap_err("Error fetching corporate compliance")
    }

    async fn create_financial_compliance(&self, institution_id: &InstitutionId, NewinancialComplainceEdit(staff_id, data): &NewinancialComplainceEdit) -> eyre::Result<FinancialCompliance> {
        query_as!(
            FinancialCompliance,
            "insert into financial_compliance
            (institution_id, director_legal_name, director_legal_bvn, 
            director_legal_dob, director_legal_gov_id_type,  director_legal_gov_id_url,
             created_by)
        values
            ($1, $2, $3, $4, $5, $6, $7)
        returning id, institution_id, director_legal_name, director_legal_bvn, 
            director_legal_dob, director_legal_gov_id_type,  director_legal_gov_id_url,
             created_by, stage, created_at, modified_at, deleted_at ",
        institution_id.0,
        data.director_legal_name,
        data.director_legal_bvn,
        data.director_legal_dob,
        data.director_legal_gov_id_type,
        data.director_legal_gov_id_url,
        staff_id.0,
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error creating financial compliance")
    }

    async fn edit_financial_compliance(&self, id: &FinancialComplianceId, data: &FinancialComplianceEdit) -> eyre::Result<FinancialCompliance> {
        query_as!(
            FinancialCompliance,
            "update financial_compliance set 
            director_legal_name=$2, director_legal_bvn=$3, 
            director_legal_dob=$4, director_legal_gov_id_type=$5,  director_legal_gov_id_url=$6,
            stage = 'PENDING' 
              where id = $1
              returning  id, institution_id, director_legal_name, director_legal_bvn, 
            director_legal_dob, director_legal_gov_id_type,  director_legal_gov_id_url,
             created_by, stage, created_at, modified_at, deleted_at ",
        id.0,
        data.director_legal_name,
        data.director_legal_bvn,
        data.director_legal_dob,
        data.director_legal_gov_id_type,
        data.director_legal_gov_id_url,
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error creating financial compliance")
    }


    async fn update_financial_compliance_status(&self, institution_id: &InstitutionId, status: &ComplianceStatus) -> eyre::Result<FinancialCompliance> {
        query_as!(
            FinancialCompliance,
            "update financial_compliance set 
                    stage = $2 
                where institution_id = $1
            returning  id, institution_id, director_legal_name, director_legal_bvn, 
            director_legal_dob, director_legal_gov_id_type,  director_legal_gov_id_url,
             created_by, stage, created_at, modified_at, deleted_at 
        ",
        institution_id.0,
            &status.to_string()
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error updating financial compliance status")
    }

    async fn get_financial_compliance(&self, institution_id: &InstitutionId) -> eyre::Result<Option<FinancialCompliance>> {
        query_as!(
            FinancialCompliance,
            "select  id, institution_id, director_legal_name, director_legal_bvn, 
            director_legal_dob, director_legal_gov_id_type,  director_legal_gov_id_url,
             created_by, stage, created_at, modified_at, deleted_at  from financial_compliance where institution_id = $1",
        institution_id.0
        )
        .fetch_optional(&self.customer_db)
        .await
        .wrap_err("Error fetching financial compliance")
    }
}