use reqwest::StatusCode;
use sqlx::Either;
use serde::{Deserialize, Serialize};
use tryhcs_commons_be::{api_response::{ApiResponse, ErrorMessage}, auth::InstitutionAdminUser};
use std::str::FromStr;

use tryhcs_shared::{compliance_params::{ComplianceStatus, CorporateComplianceDto, CorporateComplianceEdit, FinancialComplianceDto, FinancialComplianceEdit, HealthcareComplianceDto, HealthcareComplianceEdit, NewComplainceEdit, NewHealthcareComplainceEdit, NewinancialComplainceEdit}, institution_params::{InstitutionId, StaffId, StaffShadowId}};

use crate::app::ComplianceApp;

use super::repo::*;
use super::models::*;

#[cfg(test)]
use mockall::{automock, mock, predicate::*};

#[derive(Debug, Clone)]
pub struct ComplianceEvaluator<'a> (
    Option<&'a CorporateCompliance>,
    Option<&'a FinancialCompliance>,
    Option<&'a HealthcareCompliance>);

#[derive(Debug, Clone, Serialize, Deserialize,)]
pub struct ComplianceEvaluation {
    pub compliance_status: ComplianceStatus,
    pub compliance_message: String,
    pub rejected: Vec<String>,
}


impl <'a> ComplianceEvaluator<'a> {
    
    pub fn evaluate(&self) -> eyre::Result<ComplianceEvaluation> {

        let ComplianceEvaluator(corporate, financial, healthcare) = self;

            let mut corporate_compliance = ComplianceStatus::PENDING;
            let mut finance_compliance = ComplianceStatus::PENDING;
            let mut healthcare_compliance = ComplianceStatus::PENDING;

            match corporate.map(|v| ComplianceStatus::from_str(&v.stage)) {
                Some(Err(err)) => {
                    return Err(err.into());
                },
                Some(Ok(status)) => {
                    corporate_compliance = status;
                },
                _ => {},
            }

            match financial.map(|v| ComplianceStatus::from_str(&v.stage)) {
                Some(Err(err)) => {
                    return Err(err.into());
                },
                Some(Ok(status)) => {
                    finance_compliance = status;
                },
                _ => {},
            }

            match healthcare.map(|v| ComplianceStatus::from_str(&v.stage)) {
                Some(Err(err)) => {
                    return Err(err.into());
                },
                Some(Ok(status)) => {
                    healthcare_compliance = status;
                },
                _ => {},
            }

            Ok(ComplianceEvaluator::evaluate_compliance(corporate_compliance, finance_compliance, healthcare_compliance))
    }


    // write the test!!!
    fn evaluate_compliance(corporate_compliance: ComplianceStatus, finance_compliance: ComplianceStatus, healthcare_compliance: ComplianceStatus) -> ComplianceEvaluation {
        let statuses = &[(corporate_compliance, "corporate"), (finance_compliance, "financial"), (healthcare_compliance, "healthcare")];
        if statuses.iter().any(|(status, _ )| *status == ComplianceStatus::PENDING) {
                return ComplianceEvaluation {
                    compliance_status: ComplianceStatus::PENDING,
                    compliance_message: "Registration pending".into(),
                    rejected: vec![],
                };
        }
        let rejected = statuses.iter()
        .filter_map(|(status, field)| {
            if *status != ComplianceStatus::REJECTED {
                return None;
            }
            Some(field.to_string())
        }).collect::<Vec<_>>();
        if !rejected.is_empty() {
            return ComplianceEvaluation {
                compliance_status: ComplianceStatus::REJECTED,
                compliance_message: "Compliance data rejected".into(),
                rejected,
            };
        }
        if statuses.iter().any(|(status, _ )| *status == ComplianceStatus::SUBMITTED) {
            return ComplianceEvaluation {
                compliance_status: ComplianceStatus::SUBMITTED,
                compliance_message: "Compliance verification pending".into(),
                rejected: vec![],
            };
            }


        return ComplianceEvaluation {
                    compliance_status: ComplianceStatus::VERIFIED,
                    compliance_message: "Compliance verified".into(),
                    rejected: vec![],
        };
    }


}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResponse {
    pub corporate: Option<CorporateComplianceDto>,
    pub financial: Option<FinancialComplianceDto>,
    pub healthcare: Option<HealthcareComplianceDto>,
    pub evaluation: ComplianceEvaluation,
}

#[cfg(test)]
mod compliance_evaluator_tests {

    #[test]
    fn test_complaince_evaluator(
    
    ) {
        // let app = App::faux();
        // faux::when!(app)
    }
}

pub async fn get_compliance_data(app: &ComplianceApp, InstitutionAdminUser(user): &InstitutionAdminUser) -> eyre::Result<ApiResponse<ComplianceResponse>> {
    let institution_id = InstitutionId(user.institution.px);
    let corporate = app.compliance_repo.get_corporate_compliance(&institution_id).await?;
    let financial: Option<FinancialCompliance> = app.compliance_repo.get_financial_compliance(&institution_id).await?;
    let healthcare: Option<HealthcareCompliance> = app.compliance_repo.get_healthcare_compliance(&institution_id).await?;

    let evaluation = ComplianceEvaluator(corporate.as_ref(), financial.as_ref(), healthcare.as_ref()).evaluate()?;
    let overview = ComplianceResponse {
        corporate: corporate.map(|v| v.into()),
        financial: financial.map(|v| v.into()),
        healthcare: healthcare.map(|v| v.into()),
        evaluation,
    };

    Ok((StatusCode::OK, Either::Left(Some(overview))))
}


// #[cfg(test)]
// mod compliance_tests {
//     use tryhcs_shared::institution_params::{AuthorizedInstitutionUser, InstitutionDto};



//     #[tokio::test]
//     async fn get_compliance() {
//         let (app, db_pool, compliance_repo, redis,  compliance, finance)  = get_test_app().await;

//         let institution = InstitutionDto { id: 1, 
//             institution_name: "Test Institution".into(), 
//             email: "mail@institution.com".into(),
//              classification: "PRIMARY".into(), 
//              workspace_code: "TCHS-R123".into(),
//               setting: "PRIVATE".into(), address: Some("Iwo road".into()), 
//               town: Some("Ibadan".into()),
//                state: Some("Oyo".into()), 
//                logo: None 

//         };
        
//         let user = AuthorizedInstitutionUser::builder()
//         .departments(vec![])
//         .first_name("Test".into())
//         .last_name("User".into())
//         .mobile("+2348149464289".into())
//         .staff_id(1)
//         .title("Doctor".into())
//         .institution(institution.clone())
//         .build();

//         let user = InstitutionAdminUser(user.clone());


//         compliance_repo.expect_get_corporate_compliance().returning(|a| Ok(None));
//         let (status, response) = get_compliance_data(&app, &user).await.expect("failed to get compliance data");
//         let response_value = response.unwrap_left();
//     }
// }

pub async fn update_corporate_compliance(
    app: &ComplianceApp, 
    InstitutionAdminUser(user): &InstitutionAdminUser,
    data: &CorporateComplianceEdit
) -> eyre::Result<ApiResponse<CorporateComplianceDto>> {
    let institution_id = InstitutionId(user.institution.px);

    // intentionally not parallezing the request
    if let Either::Right(error_message) = app.compliance.lookup_rc_no(&data.rc_no).await? {
        return Ok((StatusCode::BAD_REQUEST, Either::Right(error_message)));
    }
    if let Either::Right(error_message) = app.compliance.lookup_tin(&data.tin).await? {
        return Ok((StatusCode::BAD_REQUEST, Either::Right(error_message)));
    }
    if let Either::Right(error_message) = app.compliance.name_lookup(&data.corporate_bank_code, &data.corporate_account_number).await? {
        return Ok((StatusCode::BAD_REQUEST, Either::Right(error_message)));
    }

    let saved_corporate: Option<CorporateCompliance> = app.compliance_repo.get_corporate_compliance(&institution_id).await?;
    let corporate_data = match saved_corporate {
        None => {
            app.compliance_repo.create_corporate_compliance(&institution_id, &NewComplainceEdit(StaffShadowId(user.staff_id.clone()),  data.to_owned())).await?
        },
        Some(corporate) => {
            if !can_user_update_document(&corporate.stage) {
                return Ok((StatusCode::FORBIDDEN, Either::Right(ErrorMessage("Forbidden".into()))))
            }
            app.compliance_repo.edit_corporate_compliance(&CorporateComplianceId(corporate.id), &data).await?
        },
    };

    Ok((StatusCode::OK, Either::Left(Some(corporate_data.into()))))
}

fn can_user_update_document(current_status: &str) -> bool {
    !ComplianceStatus::VERIFIED.to_string().eq_ignore_ascii_case(current_status) 
}

pub async fn update_healthcare_compliance(    app: &ComplianceApp, 
    InstitutionAdminUser(user): &InstitutionAdminUser, data: &HealthcareComplianceEdit) -> eyre::Result<ApiResponse<HealthcareComplianceDto>> {
    let institution_id = InstitutionId(user.institution.px);
    let saved_compliance = app.compliance_repo.get_healthcare_compliance(&institution_id).await?;
    let compliance = match saved_compliance {
        None => {
            app.compliance_repo.create_healthcare_compliance(&institution_id, &NewHealthcareComplainceEdit(StaffShadowId(user.staff_id.clone()),  data.to_owned())).await?
        },
        Some(corporate) => {
            if !can_user_update_document(&corporate.stage) {
                return Ok((StatusCode::FORBIDDEN, Either::Right(ErrorMessage("Forbidden".into()))))
            }
            app.compliance_repo.edit_healthcare_compliance(&HealthcareComplianceId(corporate.id), &data).await?
        },
    };

    Ok((StatusCode::OK, Either::Left(Some(compliance.into()))))
}

pub async fn update_financial_compliance(    app: &ComplianceApp, 
    InstitutionAdminUser(user): &InstitutionAdminUser, data: &FinancialComplianceEdit) -> eyre::Result<ApiResponse<FinancialComplianceDto>> {
    let institution_id = InstitutionId(user.institution.px);
   
   let bvn_search = app.compliance.lookup_bvn(&data.director_legal_bvn).await?;
   let bvn_error_message: String = "Invalid BVN credentials".into();
   match bvn_search    {
       Either::Right(_) => {
            return Ok((StatusCode::BAD_REQUEST, Either::Right(ErrorMessage(bvn_error_message))));     
        },
        Either::Left(bvn_info) => {
            if !bvn_info.date_of_birth.eq_ignore_ascii_case(&data.director_legal_dob) {
            return Ok((StatusCode::BAD_REQUEST, Either::Right(ErrorMessage(bvn_error_message))));
            }
        },
    };

    let saved_compliance = app.compliance_repo.get_financial_compliance(&institution_id).await?;
    let corporate_data = match saved_compliance {
        None => {
            app.compliance_repo.create_financial_compliance(&institution_id, &NewinancialComplainceEdit(StaffShadowId(user.staff_id.clone()),  data.to_owned())).await?
        },
        Some(compliance) => {
            if !can_user_update_document(&compliance.stage) {
                return Ok((StatusCode::FORBIDDEN, Either::Right(ErrorMessage("Forbidden".into()))))
            }
            app.compliance_repo.edit_financial_compliance(&FinancialComplianceId(compliance.id), &data).await?
        },
    };

    Ok((StatusCode::OK, Either::Left(Some(corporate_data.into()))))
}

pub async fn submit_compliance(    app: &ComplianceApp, 
    InstitutionAdminUser(user): &InstitutionAdminUser) -> eyre::Result<ApiResponse<()>> {
    let institution_id = InstitutionId(user.institution.px);
    let financial_compliance = app.compliance_repo.get_financial_compliance(&institution_id).await?;
    let corporate_compliance = app.compliance_repo.get_corporate_compliance(&institution_id).await?;
    let healthcare_compliance = app.compliance_repo.get_healthcare_compliance(&institution_id).await?;

    let financial_compliance = {
        match financial_compliance {
            None => {
                return Ok((StatusCode::BAD_REQUEST, Either::Right(ErrorMessage("Financial compliance details not provided".into()))))
            },
            Some(compliance) => compliance,
        }
    };
    let corporate_compliance = {
        match corporate_compliance {
            None => {
                return Ok((StatusCode::BAD_REQUEST, Either::Right(ErrorMessage("Corporate compliance details not provided".into()))))
            },
            Some(compliance) => compliance,
        }
    };
    let healthcare_compliance = {
        match healthcare_compliance {
            None => {
                return Ok((StatusCode::BAD_REQUEST, Either::Right(ErrorMessage("Healthcare compliance details not provided".into()))))
            },
            Some(compliance) => compliance,
        }
    };

    if !financial_compliance.stage.eq_ignore_ascii_case(&ComplianceStatus::VERIFIED.to_string()) {
        app.compliance_repo.update_financial_compliance_status(&institution_id, &ComplianceStatus::SUBMITTED).await?;
    }
    if !corporate_compliance.stage.eq_ignore_ascii_case(&ComplianceStatus::VERIFIED.to_string()) {
        app.compliance_repo.update_corporate_compliance_status(&institution_id, &ComplianceStatus::SUBMITTED).await?;
    }
    if !healthcare_compliance.stage.eq_ignore_ascii_case(&ComplianceStatus::VERIFIED.to_string()) {
        app.compliance_repo.update_healthcare_compliance_status(&institution_id, &ComplianceStatus::SUBMITTED).await?;
    }

    Ok((StatusCode::OK, Either::Left(Some(()))))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NinData {
    pub nin: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub phone_number: String,
    pub dob: String,
    pub email: Option<String>,
    pub profile_image: Option<String>,
}

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait ComplianceVerification: Send + Sync {
    async fn lookup_rc_no(&self, rc_no: &str) -> eyre::Result<Either<BusinessData, ErrorMessage>>;
    async fn lookup_tin(&self, tin: &str) -> eyre::Result<Either<TINData, ErrorMessage>>;
    async fn lookup_bvn(&self, bvn: &str) -> eyre::Result<Either<BVNData, ErrorMessage>>;
    async fn lookup_nin(&self, nin: &str) -> eyre::Result<Either<NinData, ErrorMessage>>;
    async fn lookup_driver_license(&self, license_no: &str) -> eyre::Result<Either<DriverLicenseData, ErrorMessage>> ;
    async fn name_lookup(&self, bank_code: &str, account_number: &str) -> eyre::Result<Either<tryhcs_shared::finance_params::BankAccountInfo, ErrorMessage>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessData {
    pub name: String,
    pub type_of_entity: String,
    pub address: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub lga: Option<String>,
    pub state: Option<String>,
    pub activity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TINData {
    pub name: String,
    pub tax_office: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BVNData {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub mobile: String,
    pub image: String,
    pub date_of_birth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverLicenseData {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub image: String,
    pub date_of_birth: String,
}