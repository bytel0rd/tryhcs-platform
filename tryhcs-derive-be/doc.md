use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tryhcs_commons_be::data_encryption::{Encrypted, Encryptor, Encryptable};

use tryhcs_derive_be::Encrypted as EncryptedDerive;
#[derive(EncryptedDerive, Serialize, Deserialize, Debug, Clone)]
pub struct Institution {
    pub id: i64,
    
    #[deterministic]
    pub shadow_id: String,

    #[randomized]
    pub name: String,
    
    #[deterministic]
    pub email: String,
    
    #[deterministic]
    pub classification: String,
    
    #[deterministic]
    pub setting: String,
    
    #[randomized]
    pub address: Option<String>,

    #[randomized]
    pub town: Option<String>,

    #[randomized]
    pub state: Option<String>,
    
    #[deterministic]
    pub created_by: i64,
    
    #[randomized]
    pub logo: Option<String>,
    
    pub workspace_code: String,
    
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}