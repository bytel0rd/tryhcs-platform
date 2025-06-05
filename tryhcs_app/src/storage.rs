#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};
use tryhcs_shared::{
    api_params::{ApiResponseData, ErrorMessage},
    encryption::Encryption,
    institution_params::{AuthenticatedUser, LoginResponse},
};

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> eyre::Result<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> eyre::Result<()>;
    async fn delete(&self, key: &str) -> eyre::Result<()>;
}

#[cfg(all(not(target_arch = "wasm32"), any(unix, windows)))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppStorage {
    InMemory,
    Native(String),
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppStorage {
    InMemory,
    Browser,
}

#[cfg(target_arch = "wasm32")]
pub mod browser {
    use gloo_storage::{LocalStorage, Storage};
    use std::sync::Arc;
    use tracing::{debug, error, info};
    use tryhcs_shared::encryption::Encryption;

    #[derive(Clone)]
    pub struct BrowserStorage {
        encryption: Arc<dyn Encryption>,
    }

    impl BrowserStorage {
        pub fn open(encryption: Arc<dyn Encryption>) -> eyre::Result<Self> {
            Ok(BrowserStorage { encryption })
        }
    }

    #[async_trait::async_trait]
    impl super::Storage for BrowserStorage {
        async fn get(&self, key: &str) -> eyre::Result<Option<String>> {
            Ok(LocalStorage::get::<String>(key).ok())
        }

        async fn set(&self, key: &str, value: &str) -> eyre::Result<()> {
            LocalStorage::set(key, value)?;
            Ok(())
        }

        async fn delete(&self, key: &str) -> eyre::Result<()> {
            Ok(LocalStorage::delete(key))
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), any(unix, windows)))]
pub mod native {
    use std::sync::Arc;
    use tracing::{debug, error, info};
    use tryhcs_shared::encryption::Encryption;

    #[derive(Clone)]
    pub struct NativeStorage {
        database: sled::Db,
        encryption: Arc<dyn Encryption>,
    }

    impl NativeStorage {
        pub fn open<T: AsRef<std::path::Path>>(
            path: T,
            encryption: Arc<dyn Encryption>,
        ) -> eyre::Result<Self> {
            let database = sled::open(path)?;
            Ok(NativeStorage {
                database,
                encryption,
            })
        }
    }

    #[async_trait::async_trait]
    impl super::Storage for NativeStorage {
        async fn get(&self, key: &str) -> eyre::Result<Option<String>> {
            if let Some(value) = self.database.get(key)? {
                let value = String::from_utf8(value.to_vec())?;
                let value = match self.encryption.decrypt(&value) {
                    Err(err) => {
                        error!("failed to decrypt key {} value", key);
                        return Err(eyre::eyre!("Failed to decrypt key"));
                    }
                    Ok(value) => value,
                };
                return Ok(Some(value));
            }

            return Ok(None);
        }

        async fn set(&self, key: &str, value: &str) -> eyre::Result<()> {
            let encrypted = match self.encryption.encrypt(value) {
                Err(err) => {
                    error!("failed to encrypt key {} value", key);
                    return Err(eyre::eyre!("Failed to encrypt key value"));
                }
                Ok(value) => value,
            };
            self.database.insert(key, encrypted.as_str())?;
            self.database.flush()?;
            Ok(())
        }

        async fn delete(&self, key: &str) -> eyre::Result<()> {
            self.database.remove(key)?;
            self.database.flush()?;
            Ok(())
        }
    }
}

pub mod memory {
    use async_trait::async_trait;
    use eyre::{eyre, Result};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub struct InMemoryStorage {
        inner: Arc<Mutex<HashMap<String, String>>>,
    }

    impl InMemoryStorage {
        pub fn open() -> Self {
            Self {
                inner: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl super::Storage for InMemoryStorage {
        async fn get(&self, key: &str) -> Result<Option<String>> {
            match self.inner.lock() {
                Ok(map) => Ok(map.get(key).cloned()),
                Err(err) => Err(eyre!("Failed to acquire lock: {}", err)),
            }
        }

        async fn set(&self, key: &str, value: &str) -> Result<()> {
            match self.inner.lock() {
                Ok(mut map) => {
                    map.insert(key.to_string(), value.to_string());
                    Ok(())
                }
                Err(err) => Err(eyre!("Failed to acquire lock: {}", err)),
            }
        }

        async fn delete(&self, key: &str) -> Result<()> {
            match self.inner.lock() {
                Ok(mut map) => {
                    map.remove(key);
                    Ok(())
                }
                Err(err) => Err(eyre!("Failed to acquire lock: {}", err)),
            }
        }
    }
}
