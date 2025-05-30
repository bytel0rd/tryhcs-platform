use either::Either;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tryhcs_shared::{
    api_params::ErrorMessage,
    encryption::{Encryption, NoEncryption},
    institution_params::{AuthenticatedUser, InitiatedOtp, LoginReq, VerifyOTP},
};

use crate::{
    hcs_api::HcsApi,
    hcs_endpoints::HcsEndpoints,
    state_engine::global_state::GlobalState,
    storage::{AppStorage, Storage},
};

pub const GENERAL_APP_INTERNAL_ERROR: &str = "INTERNAL APP ERROR";

pub struct GlobalApplication {
    pub(crate) mode: ApplicationMode,
    pub(crate) state: Arc<RwLock<GlobalState>>,
}

#[derive(Clone)]
pub enum ApplicationMode {
    Guest(GuestApplication),
    Authenticated(AuthenticatedApplication),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppEncryption {
    NoEncryption,
    // Not implemented yet defaults to NoEncryption
    InstitutionEncryption,
    // Not implemented yet defaults to NoEncryption
    UserEncryption,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HcsAppConfig {
    pub base_api_url: String,
    pub debug_enabled: bool,
    pub encryption_mode: AppEncryption,
    pub storage: AppStorage,
}

fn create_app_core(config: HcsAppConfig) -> eyre::Result<CoreApplication> {
    let config = Arc::new(config);
    let encryption = Arc::new(NoEncryption);

    let storage: Arc<dyn Storage> = {
        match &config.storage {
            #[cfg(target_arch = "wasm32")]
            AppStorage::Browser => Arc::new(crate::storage::browser::BrowserStorage::open(
                encryption.clone(),
            )?),
            AppStorage::InMemory => Arc::new(crate::storage::memory::InMemoryStorage::open()),

            #[cfg(all(not(target_arch = "wasm32"), any(unix, windows)))]
            AppStorage::Native(path) => Arc::new(crate::storage::native::NativeStorage::open(
                &path,
                encryption.clone(),
            )?),
        }
    };

    let hcs_api = Arc::new(HcsApi::new(config.clone(), encryption.clone()));
    Ok(CoreApplication {
        hcs_api,
        storage,
        config,
        encryption: encryption.clone(),
    })
}

#[derive(Clone)]
pub struct CoreApplication {
    hcs_api: Arc<dyn HcsEndpoints>,
    config: Arc<HcsAppConfig>,
    storage: Arc<dyn Storage>,
    encryption: Arc<dyn Encryption>,
}

#[derive(Clone)]
pub struct GuestApplication {
    core: CoreApplication,
}

impl GuestApplication {
    pub fn new(config: &HcsAppConfig) -> eyre::Result<Self> {
        Ok(Self {
            core: create_app_core(config.clone())?,
        })
    }
}

impl GuestApplication {
    pub async fn login(
        &self,
        login_req: &LoginReq,
    ) -> eyre::Result<Either<AuthenticatedApplication, InitiatedOtp>, ErrorMessage> {
        let login_res = self.core.hcs_api.login(login_req).await?;
        if let Some(otp) = login_res.otp {
            return Ok(Either::Right(otp));
        }

        if let Some(authenticated) = login_res.auth {
            return Ok(Either::Left(AuthenticatedApplication::new(
                authenticated,
                self.core.clone(),
            )));
        }

        Err(ErrorMessage("#Authentication failure".into()))
    }

    pub async fn verify_otp(
        &self,
        verify_otp: &VerifyOTP,
    ) -> eyre::Result<AuthenticatedApplication, ErrorMessage> {
        let authenticated = self.core.hcs_api.verify_otp(verify_otp).await?;
        return Ok(AuthenticatedApplication::new(
            authenticated,
            self.core.clone(),
        ));
    }
}

#[derive(Clone)]
pub struct AuthenticatedApplication {
    user: AuthenticatedUser,
    core: CoreApplication,
}

impl AuthenticatedApplication {
    pub fn new(user: AuthenticatedUser, core: CoreApplication) -> Self {
        Self { core, user }
    }
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
// impl AuthenticatedApplication {

//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
//     pub async fn get_auth_profile(&self) -> eyre::Result<StaffDto, ErrorMessage> {
//         todo!()
//     }

//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
//     pub async fn get_staff_details(
//         &self,
//         staff_id: &StaffId,
//     ) -> eyre::Result<Option<StaffDto>, ErrorMessage> {
//         todo!()
//     }

//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
//     pub async fn search_staffs_directory(
//         &self,
//         query: Option<&str>,
//     ) -> eyre::Result<Vec<StaffDto>, ErrorMessage> {
//         todo!()
//     }

//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
//     pub async fn search_departments(
//         &self,
//         query: Option<&str>,
//     ) -> eyre::Result<Vec<DepartmentDto>, ErrorMessage> {
//         todo!()
//     }

//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
//     pub async fn get_department(
//         &self,
//         department_id: &DepartmentId,
//     ) -> eyre::Result<Option<StaffDto>, ErrorMessage> {
//         todo!()
//     }
// }
