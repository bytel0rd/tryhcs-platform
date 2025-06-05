use eyre::Ok;

use crate::{data_encryption::EncryptableData, redis::Cache};

const INSTITUTION_ENCRYPTION_CONTEXT_KEY: &str = "SYSTEM|ECTX|";

// calls kms to get new key
pub async fn get_system_context_keys<T: Cache>(redis: &T) -> eyre::Result<(Vec<u8>, Vec<u8>)> {
    return Ok((vec![], vec![]));
}

// calls kms to decrypt the DEK
pub async fn get_institution_context_keys<T: Cache>(
    redis: &T,
    institution_dek: &str,
) -> eyre::Result<(Vec<u8>, Vec<u8>)> {
    return Ok((vec![], vec![]));
}

pub async fn set_institution_context_keys<T: Cache>(
    redis: &T,
    institution_id: &str,
) -> eyre::Result<(Vec<u8>, Vec<u8>)> {
    return Ok((vec![], vec![]));
}
