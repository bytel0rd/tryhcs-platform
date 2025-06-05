use std::sync::Arc;

use axum::response;
use either::Either;
use eyre::{eyre, Ok};
use params::*;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, error, info};
use tryhcs_commons_be::{api_response::ErrorMessage, env::EnvConfig};
use tryhcs_shared::finance_params::BankAccountInfo;

use crate::api::{BVNData, BusinessData, ComplianceVerification, DriverLicenseData, NinData, TINData};




#[derive(Clone, Debug)]
pub struct YouverifyApi {
    pub env: Arc<EnvConfig>,
}

pub mod params {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub(super) struct NINReq {
        pub id: String,

        #[serde(rename = "isSubjectConsent")]
        pub is_subject_consent: bool,

        #[serde(rename = "premiumNin")]
        pub premium_nin: bool,
    }


    #[derive(Deserialize, Serialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub(super) struct RcNoReq {
        pub registration_number: String,
        pub is_consent: bool,
        pub country_code: String,
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub(super) struct TaxIdReq {
        pub tin: String,
        pub is_consent: bool,
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub(super) struct BVNReq {
        pub id: String,
        pub is_subject_consent: bool,
        // pub premium_bvn: bool,
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub(super) struct DriverLicenseReq {
        pub id: String,
        pub is_subject_consent: bool,
    }


    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct YouverifyResponse<T> {
        pub success: bool,
        pub status_code: u16,
        pub message: Option<String>,
        pub data: Option<T>,
        pub links: Option<Vec<Option<serde_json::Value>>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NiNResponse {
        pub id: String,
        pub address: Address,
        pub status: String,
        pub first_name: String,
        pub middle_name: Option<String>,
        pub last_name: String,
        pub image: Option<String>,
        pub mobile: String,
        pub email: Option<String>,
        pub birth_state: String,
        pub religion: Option<String>,
        #[serde(rename = "birthLGA")]
        pub birth_lga: String,
        pub birth_country: String,
        pub date_of_birth: String,
        pub is_consent: bool,
        pub id_number: String,
        #[serde(rename = "type")]
        pub id_type: String,
        pub gender: String,
        pub requested_at: String,
        pub requested_by_id: String,
        pub country: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RcNoResponse {
        pub name: String,
        pub type_of_entity: String,
        pub address: String,
        pub email: Option<String>,
        pub phone: Option<String>,
        pub lga: Option<String>,
        pub state: Option<String>,
        pub activity: Option<String>,
        pub status: String,
        pub is_consent: bool,
        pub country: String,
        pub key_personnel: Vec<CompanyPersonnel>,
        pub business_id: String,
        pub r#type: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TaxIdResponse {
        pub name: String,
        pub tax_office: String,
        pub status: String,
        pub is_consent: bool,
        pub business_id: String,
        pub r#type: String,
    }


    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BVNResponse {
        pub first_name: String,
        pub middle_name: Option<String>,
        pub last_name: String,
        pub mobile: String,
        pub image: String,
        pub date_of_birth: String,
        pub status: String,
        pub is_consent: bool,
        pub business_id: String,
        pub r#type: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DriverLicenseResponse {
        pub first_name: String,
        pub middle_name: Option<String>,
        pub last_name: String,
        pub image: String,
        pub date_of_birth: String,
        pub status: String,
        pub is_consent: bool,
        pub business_id: String,
        pub r#type: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AccountLookupData {
        pub account_name: String,
        pub account_number: String,
        pub bank_name: String,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AccountLookupResponse {
        pub bank_details: AccountLookupData,
    }
    

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CompanyPersonnel {
        pub name: String,
        pub designation: String,
        pub address: String,
        pub occupation: Option<String>,
        pub nationality: Option<String>,
        pub country_of_residence: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(super) struct Address {
        pub town: String,
        pub lga: String,
        pub state: String,
        pub address_line: String,
    }
}

const YOUVERIFY_NOT_FOUND_STATUS: &str = "not_found";
const YOUVERIFY_FOUND_STATUS: &str = "found";
const YOUVERIFY_STATUS_FIELD: &str = "status";

impl YouverifyApi {
    fn create_client(&self) -> eyre::Result<Client> {
        use reqwest::header;

        let mut headers = header::HeaderMap::new();

        let content_type = mime::APPLICATION_JSON.to_string();

        headers.insert(
            "token",
            header::HeaderValue::from_str(&self.env.youverify_api_key)?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_str(content_type.as_str())?,
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            ?;

        Ok(client)
    }

    fn extract_response_or_error<T: DeserializeOwned>(&self, raw_response: &str, error_message: &str) -> eyre::Result<Either<T, ErrorMessage>> {
        let response = serde_json::from_str::<YouverifyResponse<serde_json::Value>>(raw_response)
            .map(|v| v.data)
            .ok()
            .flatten();

        if let Some(response) = response {
            if let Some(validation_error) = response.get("message").map(|v| v.as_str()).flatten() {
                return Ok(Either::Right(ErrorMessage(validation_error.to_string())));
            }
            let status = response.get(YOUVERIFY_STATUS_FIELD)
            .map(|s| s.as_str())
            .flatten()
            .unwrap_or(YOUVERIFY_NOT_FOUND_STATUS);
            if status.eq_ignore_ascii_case(YOUVERIFY_NOT_FOUND_STATUS) {
                return Ok(Either::Right(ErrorMessage(error_message.to_string())));
            }
            let data: T = serde_json::from_value::<T>(response)?;
            return Ok(Either::Left(data));
        }

        return Err(eyre!("Failed to extract response details"));
    }

}

#[async_trait::async_trait]
impl ComplianceVerification for YouverifyApi {
     async fn lookup_nin(&self, nin: &str) -> eyre::Result<Either<NinData, ErrorMessage>> {
        let client = self.create_client()?;

        let url = format!("{}/v2/api/identity/ng/nin", self.env.youverify_base_url);

        let body = NINReq {
            id: nin.to_owned(),
            is_subject_consent: true,
            premium_nin: false,
        };
        let body = serde_json::to_string(&body)?;

        info!(message ="send lookup NiN req", url=&url, body= ?body);

        let response = client
            .post(url.as_str())
            .body(body)
            .send()
            .await
            ?;

        let is_success = response.status();
        let response = response.text().await?;

        info!(message="Lookup Nin response", status=?is_success, response=?response);
        let response = self.extract_response_or_error::<NiNResponse>(response.as_str(), "NIN not found")?;
        let nin_info = response.map_left(|response| NinData {
            nin: nin.to_owned(),
            first_name: response.first_name,
            middle_name: response.middle_name,
            last_name: response.last_name,
            phone_number: response.mobile,
            dob: response.date_of_birth,
            email: response.email,
            profile_image: response.image,
        });
        Ok(nin_info)

    }
    
    async fn lookup_rc_no(&self, rc_no: &str) -> eyre::Result<Either<BusinessData, ErrorMessage>> {
let client = self.create_client()?;

        let url = format!("{}/v2/api/verifications/global/company-advance-check", self.env.youverify_base_url);

        let body = RcNoReq {
            registration_number: rc_no.to_owned(),
            is_consent: true,
            country_code: "NG".to_owned(),
        };
        let body = serde_json::to_string(&body)?;

        info!(message ="send lookup business registration number req", url=&url, body= ?body);

        let response = client
            .post(url.as_str())
            .body(body)
            .send()
            .await
            ?;

        let is_success = response.status();
        let response = response.text().await?;

        info!(message="lookup business registration number response", status=?is_success, response=?response);
        let business_info = self.extract_response_or_error::<RcNoResponse>(response.as_str(), "RC number not found")?;
        let business_info = business_info.map_left(|response| BusinessData {
            name: response.name,
            type_of_entity: response.type_of_entity,
            activity: response.activity,
            address: response.address,
            phone: response.phone,
            lga: response.lga,
            state: response.state,
            email: response.email,
        });
        Ok(business_info)
        }
    
    async fn lookup_tin(&self, tin: &str) -> eyre::Result<Either<TINData, ErrorMessage>> {
        let client = self.create_client()?;

        let url = format!("{}/v2/api/verifications/ng/tin", self.env.youverify_base_url);

        let body: TaxIdReq = TaxIdReq {
            tin: tin.to_owned(),
            is_consent: true,
        };
        let body = serde_json::to_string(&body)?;

        info!(message ="send lookup TIN req", url=&url, body= ?body);

        let response = client
            .post(url.as_str())
            .body(body)
            .send()
            .await
            ?;

        let is_success = response.status();
        let response = response.text().await?;

        info!(message="Lookup TIN response", status=?is_success, response=?response);
        let tax_info = self.extract_response_or_error::<TaxIdResponse>(response.as_str(), "TIN not found")?;
        let tax_info = tax_info.map_left(|response| TINData {
            name: response.name,
            tax_office: Some(response.tax_office),                   
        });
        Ok(tax_info)

    }
    
    async fn lookup_bvn(&self, bvn: &str) -> eyre::Result<Either<BVNData, ErrorMessage>> {
        let client = self.create_client()?;

        let url = format!("{}/v2/api/identity/ng/bvn", self.env.youverify_base_url);
        let body = BVNReq {
            id: bvn.to_owned(),
            is_subject_consent: true,
            // premium_bvn: false,
        };
        let body = serde_json::to_string(&body)?;

        info!(message ="send lookup BVN req", url=&url, body= ?body);

        let response = client
            .post(url.as_str())
            .body(body)
            .send()
            .await
            ?;

        let is_success = response.status();
        let response = response.text().await?;

        info!(message="Lookup business response", status=?is_success);
// This is meant to be a debug log        
info!(message="lookup BVN response", response=?response);
        let bvn_info = self.extract_response_or_error::<BVNResponse>(response.as_str(), "BVN not found")?;
        let bvn_info = bvn_info.map_left(|response|BVNData {
            first_name: response.first_name,
            middle_name: response.middle_name,
            last_name: response.last_name,
            mobile: response.mobile,
            image: response.image,
            date_of_birth: response.date_of_birth,
        });
        Ok(bvn_info)
    }
    
    async fn lookup_driver_license(&self, license_no: &str) -> eyre::Result<Either<DriverLicenseData, ErrorMessage>>  {
        let client = self.create_client()?;

        let url = format!("{}/v2/api/identity/ng/drivers-license", self.env.youverify_base_url);

        let body = DriverLicenseReq {
            id: license_no.to_owned(),
            is_subject_consent: true,
        };
        let body = serde_json::to_string(&body)?;

        info!(message ="send lookup driver license req", url=&url, body= ?body);

        let response = client
            .post(url.as_str())
            .body(body)
            .send()
            .await
            ?;

        let is_success = response.status();
        let response = response.text().await?;

        info!(message="Lookup business response", status=?is_success, response=?response);
        let license_info = self.extract_response_or_error::<DriverLicenseResponse>(response.as_str(), "License information not found")?;
        let license_info = license_info.map_left(|response| DriverLicenseData {
            first_name: response.first_name,
            middle_name: response.middle_name,
            last_name: response.last_name,
            image: response.image,
            date_of_birth: response.date_of_birth,
        });
        Ok(license_info)
    }

    async fn name_lookup(&self, bank_code: &str, account_number: &str) -> eyre::Result<Either<BankAccountInfo, ErrorMessage>> {
        let client = self.create_client()?;

        let url = format!("{}/v2/api/identity/ng/bank-account-number/resolve", self.env.youverify_base_url);

        let body = json!({
            "accountNumber": account_number,
            "bankCode": bank_code,
            "isSubjectConsent": true
        });
        let body = serde_json::to_string(&body)?;

        info!(message ="send lookup  bank account information req", url=&url, body= ?body);
        let response = client
            .post(url.as_str())
            .body(body)
            .send()
            .await
            ?;

        let is_success = response.status();
        let response = response.text().await?;

        info!(message="Bank account information response", status=?is_success, response=?response);
        let account_info = self.extract_response_or_error::<AccountLookupResponse>(response.as_str(), "Account information not found")?;
        let account_info = account_info.map_left(|v| BankAccountInfo{
            name: v.bank_details.account_name,
            account_nmuber: v.bank_details.account_number,
            bank_code: bank_code.to_string(),
        });
        Ok(account_info)
    }
    
}

// #[async_trait::async_trait]
// impl BankAccountOperations for YouverifyApi {
//     async fn get_banks(&self) -> eyre::Result<Vec<BankInfo>> {
//         let client = self.create_client()?;

//         let url = format!("{}/v2/api/identity/ng/bank-account-number/bank-list", self.env.youverify_base_url);
//         info!(message ="Get banks list", url=&url);

//         let response = client
//             .get(url.as_str())
//             .send()
//             .await
//             ?;

//         let is_success = response.status();
//         let response = response.text().await?;
//         info!(message="Get banks response", status=?is_success);
//         if !is_success.is_success() {
//             error!(message="Get banks failed", response=?response);
//             return Err(eyre!("Get banks list failed"));
//         }

//         info!(message="Get banks response", response=?response);
//         match serde_json::from_str::<YouverifyResponse<Vec<BankInfo>>>(response.as_str())
//             .map(|v| v.data)
//             .ok()
//             .flatten()
//         {
//             Some(response) => {
//                 return Ok(response);
//             }
//             None => {
//                 error!(message="failed to get banks list", response=?response);
//                 return Err(eyre!("Get banks list failed"));
//             }
//         }
//     }

//     async fn name_lookup(&self, bank_code: &str, account_number: &str) -> eyre::Result<Either<BankAccountInfo, ErrorMessage>> {
//         let client = self.create_client()?;

//         let url = format!("{}/v2/api/identity/ng/bank-account-number/resolve", self.env.youverify_base_url);

//         let body = json!({
//             "accountNumber": account_number,
//             "bankCode": bank_code,
//             "isSubjectConsent": true
//         });
//         let body = serde_json::to_string(&body)?;

//         info!(message ="send lookup  bank account information req", url=&url, body= ?body);
//         let response = client
//             .post(url.as_str())
//             .body(body)
//             .send()
//             .await
//             ?;

//         let is_success = response.status();
//         let response = response.text().await?;

//         info!(message="Bank account information response", status=?is_success, response=?response);
//         let account_info = self.extract_response_or_error::<AccountLookupResponse>(response.as_str(), "Account information not found")?;
//         let account_info = account_info.map_left(|v| BankAccountInfo{
//             name: v.bank_details.account_name,
//             account_nmuber: v.bank_details.account_number,
//             bank_code: bank_code.to_string(),
//         });
//         Ok(account_info)
//     }
// }

// #[cfg(test)]
// mod tests {

//     use eyre::Context;
//     use phonenumber::{country::Id::NG, Mode};
//     use dotenv::dotenv;


//     use crate::env::EnvConfig;

//     use super::*;

//     #[tokio::test]
//     async fn lookup_nin() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let api = get_api(Arc::new(env));
//         let response = api.lookup_nin("11111111111").await?;

//         println!(
//             "NiN data {}",
//             serde_json::to_string_pretty(&response)?
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn lookup_rc_no() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let api = get_api(Arc::new(env));
//         let response = api.lookup_rc_no("RC00000000").await?;

//         println!(
//             "RcNo data {}",
//             serde_json::to_string_pretty(&response)?
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn lookup_tin() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let api = get_api(Arc::new(env));
//         let response = api.lookup_tin("00000000-0000").await?;

//         println!(
//             "TIN data {}",
//             serde_json::to_string_pretty(&response)?
//         );

//         Ok(())
//     }


//     #[tokio::test]
//     async fn lookup_bvn() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let api = get_api(Arc::new(env));
//         let response = api.lookup_bvn("11111111111").await?;

//         println!(
//             "BVN data {}",
//             serde_json::to_string_pretty(&response)?
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn lookup_driver_license() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let api = get_api(Arc::new(env));
//         let response = api.lookup_driver_license("AAA00000AA00").await?;

//         println!(
//             "Driver license data {}",
//             serde_json::to_string_pretty(&response)?
//         );

//         Ok(())
//     }


//     #[tokio::test]
//     async fn get_banks_list() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let api = get_api(Arc::new(env));
//         let response = api.get_banks().await?;

//         println!(
//             "Bank list data {}",
//             serde_json::to_string_pretty(&response)?
//         );

//         Ok(())
//     }


//     #[tokio::test]
//     async fn lookup_bank_account_details() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let api = get_api(Arc::new(env));
//         let response = api.name_lookup("058", "1000000000").await?;

//         println!(
//             "Account details data {}",
//             serde_json::to_string_pretty(&response)?
//         );

//         Ok(())
//     }

//     fn get_api(env: Arc<EnvConfig>) -> YouverifyApi {
//         let api = YouverifyApi { env };

//         api
//     }

//     #[test]
//     fn it_works() -> eyre::Result<()> {
//         dotenv().ok();
//         tracing_subscriber::fmt().init();
//         let env = envy::from_env::<EnvConfig>().wrap_err("loaded config files")?;

//         let number = "+2348149464288";
//         let number = phonenumber::parse(Some(NG), number).unwrap();
//         let valid = phonenumber::is_valid(&number);

//         if valid {
//             println!("\x1b[32m{:#?}\x1b[0m", number);
//             println!();
//             println!(
//                 "International: {}",
//                 number.format().mode(Mode::International)
//             );
//             println!("     National: {}", number.format().mode(Mode::National));
//             println!("      RFC3966: {}", number.format().mode(Mode::Rfc3966));
//             println!("        E.164: {}", number.format().mode(Mode::E164));
//         }

//         Ok(())
//     }
// }