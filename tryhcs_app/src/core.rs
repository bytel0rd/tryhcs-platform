use either::Either;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error,
    sync::{Arc, RwLock},
};
use tracing::{error, info};
use tryhcs_shared::{
    api_params::ErrorMessage,
    encryption::{self, Encryption, NoEncryption},
    institution_params::{
        AuthenticatedUser, AuthorizedUser, DepartmentDto, DepartmentId, DepartmentShadowId,
        InitiatedOtp, LoginReq, StaffDto, StaffId, StaffShadowId, VerifyOTP,
    },
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{
    hcs_api::{HcsApi, AUTH_TOKEN_STORAGE_KEY, CURRENT_WORKSPACE_STORAGE_KEY},
    hcs_endpoints::HcsEndpoints,
    state_engine::global_state::GlobalState,
    storage::{AppStorage, Storage},
};

pub const GENERAL_APP_INTERNAL_ERROR: &str = "INTERNAL APP ERROR";

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct GlobalApplication {
    pub(crate) mode: ApplicationMode,
    pub(crate) state: Arc<RwLock<GlobalState>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct JsAppHooks {}
impl AppHook for JsAppHooks {
    fn on_log_out(&self) {
        todo!()
    }

    fn is_online_callback(&self) -> bool {
        todo!()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl GlobalApplication {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub async fn new(config: HcsAppConfig, app_hooks: Option<JsAppHooks>) -> Result<Self, String> {
        let app_hooks: Option<Box<dyn AppHook>> =
            app_hooks.map(|c| Box::new(c) as Box<dyn AppHook>);
        GlobalApplication::internal_new(config, app_hooks).await
    }

    async fn internal_new(
        config: HcsAppConfig,
        app_hooks: Option<Box<dyn AppHook>>,
    ) -> Result<Self, String> {
        match create_app_core(config, Arc::new(app_hooks)) {
            Err(error_message) => {
                error!(message = "Failed to create application core", error=?error_message);
                return Err("FAILED TO CREATE APPLICATION CORE".into());
            }
            Ok(core_app) => {
                let api = core_app.hcs_api.clone();
                let app_mode = {
                    match api.get_auth_profile().await {
                        Err(err) => {
                            error!(message="Initate boot get auth profile", error_message=? err);
                            ApplicationMode::Guest(GuestApplication::new(core_app))
                        }
                        Ok(user_profile) => {
                            info!("Initate boot get app profile successful");
                            ApplicationMode::Guest(GuestApplication { core: core_app })
                            // allow compiling
                            // ApplicationMode::Authenticated(AuthenticatedApplication::new(
                            //     user_profile,
                            //     core_app,
                            // ))
                        }
                    }
                };
                Ok(GlobalApplication {
                    mode: app_mode,
                    state: Arc::new(RwLock::new(GlobalState::default())),
                })
            }
        }
    }
}

#[derive(Clone)]
pub enum ApplicationMode {
    Guest(GuestApplication),
    Authenticated(AuthenticatedApplication),
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppEncryption {
    NoEncryption,
    // Not implemented yet defaults to NoEncryption
    InstitutionEncryption,
    // Not implemented yet defaults to NoEncryption
    UserEncryption,
}

pub trait AppHook: Send + Sync {
    fn on_log_out(&self);
    fn is_online_callback(&self) -> bool;
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HcsAppConfig {
    pub(crate) base_api_url: String,
    pub(crate) debug_enabled: bool,
    pub(crate) encryption_mode: AppEncryption,
    pub(crate) storage: AppStorage,
    pub(crate) request_timeout_in_sec: u32,
}

impl Default for HcsAppConfig {
    fn default() -> Self {
        Self {
            base_api_url: Default::default(),
            debug_enabled: true,
            encryption_mode: AppEncryption::NoEncryption,
            storage: AppStorage::InMemory,
            request_timeout_in_sec: 10,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl HcsAppConfig {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(base_url: String) -> Self {
        Self {
            base_api_url: base_url,
            ..HcsAppConfig::default()
        }
    }

    pub fn set_storage(self, storage: AppStorage) -> Self {
        Self { storage, ..self }
    }

    pub fn set_encryption(self, encryption_mode: AppEncryption) -> Self {
        Self {
            encryption_mode,
            ..self
        }
    }

    pub fn set_req_timeout_in_sec(self, request_timeout_in_sec: u32) -> Self {
        Self {
            request_timeout_in_sec,
            ..self
        }
    }
}

pub fn create_app_core(
    config: HcsAppConfig,
    app_hooks: Arc<Option<Box<dyn AppHook>>>,
) -> eyre::Result<CoreApplication> {
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

    let hcs_api = Arc::new(HcsApi::new(
        config.clone(),
        storage.clone(),
        encryption.clone(),
        app_hooks,
    ));
    Ok(CoreApplication {
        hcs_api,
        storage,
        config,
        encryption,
    })
}

#[derive(Clone)]
pub struct CoreApplication {
    pub hcs_api: Arc<dyn HcsEndpoints>,
    pub config: Arc<HcsAppConfig>,
    pub storage: Arc<dyn Storage>,
    pub encryption: Arc<dyn Encryption>,
}

#[derive(Clone)]
pub struct GuestApplication {
    core: CoreApplication,
}

impl GuestApplication {
    pub fn new(core: CoreApplication) -> Self {
        Self { core }
    }
}

impl GuestApplication {
    pub async fn login(
        &self,
        login_req: &LoginReq,
    ) -> eyre::Result<Either<AuthenticatedApplication, InitiatedOtp>, ErrorMessage> {
        let internal_auth_error: ErrorMessage = ErrorMessage("#Authentication failure".into());

        let login_res = self.core.hcs_api.login(login_req).await?;
        if let Some(otp) = login_res.otp {
            return Ok(Either::Right(otp));
        }

        if let Some(authenticated) = login_res.auth {
            self.handle_credentials(&authenticated).await?;
            return Ok(Either::Left(AuthenticatedApplication::new(
                authenticated.principal,
                self.core.clone(),
            )));
        }

        Err(internal_auth_error)
    }

    async fn handle_credentials(
        &self,
        authenticated: &AuthenticatedUser,
    ) -> Result<(), ErrorMessage> {
        let internal_auth_error: ErrorMessage = ErrorMessage("#Authentication failure".into());

        match &authenticated.token {
            None => {
                error!(message = "failed to store token in storage");
                return Err("Missing auth token".into());
            }
            Some(token) => {
                if let Err(error_message) =
                    self.core.storage.set(AUTH_TOKEN_STORAGE_KEY, &token).await
                {
                    error!(message="failed to store token in storage", error_message=?error_message);
                    return Err(internal_auth_error);
                }
            }
        };

        let mut is_valid_workspace_code = false;
        if let Ok(Some(workspace_code)) = self.core.storage.get(CURRENT_WORKSPACE_STORAGE_KEY).await
        {
            is_valid_workspace_code = authenticated
                .principal
                .accounts
                .iter()
                .any(|a| a.institution.workspace_code.eq(&workspace_code));
        }

        if !is_valid_workspace_code {
            let workspace = authenticated.principal.accounts.first();
            match workspace {
                None => {
                    return Err(ErrorMessage("User does not have any workspace".into()));
                }
                Some(user) => {
                    if let Err(error_message) = self
                        .core
                        .storage
                        .set(
                            CURRENT_WORKSPACE_STORAGE_KEY,
                            &user.institution.workspace_code,
                        )
                        .await
                    {
                        error!(message="failed to store workspace code in storage", error_message=?error_message);
                        return Err(internal_auth_error);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn verify_otp(
        &self,
        verify_otp: &VerifyOTP,
    ) -> eyre::Result<AuthenticatedApplication, ErrorMessage> {
        let authenticated = self.core.hcs_api.verify_otp(verify_otp).await?;
        self.handle_credentials(&authenticated).await?;
        return Ok(AuthenticatedApplication::new(
            authenticated.principal,
            self.core.clone(),
        ));
    }
}

#[derive(Clone)]
pub struct AuthenticatedApplication {
    user: AuthorizedUser,
    core: CoreApplication,
}

impl AuthenticatedApplication {
    pub fn new(user: AuthorizedUser, core: CoreApplication) -> Self {
        Self { core, user }
    }
}

impl AuthenticatedApplication {
    pub async fn extract_from_storage<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let key: String = format!("{}|{}", self.get_workspace().await, key);
        match self.core.storage.get(&key).await {
            Err(error_message) => {
                error!(message="DB error while getting key", key=key, err=?error_message);
                return None;
            }
            Ok(saved_data) => match saved_data {
                None => {
                    return None;
                }
                Some(data_str) => match serde_json::from_str(&data_str) {
                    Err(error_message) => {
                        error!(message="Unable to serialized saved data", key=key, err=?error_message);
                        return None;
                    }
                    Ok(data) => {
                        return Some(data);
                    }
                },
            },
        }
    }

    pub async fn save_to_storage<T: Serialize>(&self, key: &str, data: &T) -> eyre::Result<()> {
        let data = serde_json::to_string(data)?;
        let key: String = format!("{}|{}", self.get_workspace().await, key);
        self.core.storage.set(&key, &data).await?;
        Ok(())
    }

    pub async fn save_to_storage_ignore_error<T: Serialize>(&self, key: &str, data: &T) {
        if let Err(error_message) = self.save_to_storage(key, data).await {
            error!(message="Error saving data into db storage", key=key, err=?error_message);
        }
    }

    pub async fn get_workspace(&self) -> String {
        match self.core.storage.get(CURRENT_WORKSPACE_STORAGE_KEY).await {
            Err(error_message) => {
                error!(message="Storage error while getting workspace key", err=?error_message);
            }
            Ok(Some(workspace_code)) => {
                return workspace_code;
            }
            _ => {}
        };

        return "Default".to_string();
    }
}

impl AuthenticatedApplication {
    pub async fn get_auth_profile(&self) -> eyre::Result<StaffDto, ErrorMessage> {
        let api = &self.core.hcs_api;
        return api.get_auth_profile().await;
        // if api.is_online().await {
        // }
        // return Ok(self.user.clone());
    }

    pub async fn get_staff_details(
        &self,
        staff_id: &StaffShadowId,
    ) -> eyre::Result<StaffDto, ErrorMessage> {
        let staff = self
            .search_staffs_directory(None)
            .await?
            .into_iter()
            .find(|staff| staff.id.eq(&staff_id.0));
        staff.ok_or(ErrorMessage("Staff not found".into()))
    }

    pub async fn search_staffs_directory(
        &self,
        query: Option<&str>,
    ) -> eyre::Result<Vec<StaffDto>, ErrorMessage> {
        let storage_key = "DIR|STAFFS";
        let api = &self.core.hcs_api;
        let mut staffs = {
            if api.is_online().await {
                let staffs = api.search_staffs_directory().await?;
                self.save_to_storage_ignore_error(storage_key, &staffs)
                    .await;
                staffs
            } else {
                self.extract_from_storage(storage_key)
                    .await
                    .unwrap_or(vec![])
            }
        };

        if let Some(query) = query {
            staffs = staffs
                .into_iter()
                .filter(|x| {
                    x.first_name.contains(query)
                        || x.last_name.contains(query)
                        || x.title.contains(query)
                })
                .collect();
        }

        return Ok(staffs);
    }

    pub async fn search_departments(
        &self,
        query: Option<&str>,
    ) -> eyre::Result<Vec<DepartmentDto>, ErrorMessage> {
        let storage_key = "DIR|DEPARTMENTS";
        let api = &self.core.hcs_api;
        let mut departments = {
            if api.is_online().await {
                let staffs = api.search_departments().await?;
                self.save_to_storage_ignore_error(storage_key, &staffs)
                    .await;
                staffs
            } else {
                self.extract_from_storage(storage_key)
                    .await
                    .unwrap_or(vec![])
            }
        };

        if let Some(query) = query {
            let query = query.to_lowercase();
            departments = departments
                .into_iter()
                .filter(|x| x.name.to_lowercase().contains(&query))
                .collect();
        }

        return Ok(departments);
    }

    pub async fn get_department(
        &self,
        department_id: &DepartmentShadowId,
    ) -> eyre::Result<DepartmentDto, ErrorMessage> {
        let department = self
            .search_departments(None)
            .await?
            .into_iter()
            .find(|department| department.id.eq(&department_id.0));
        department.ok_or(ErrorMessage("Department not found".into()))
    }
}
