use either::Either;
use serde::de::DeserializeOwned;
use tracing::{debug, error};
use tryhcs_shared::{
    api_params::{ApiResponseError, ErrorMessage},
    encryption::Encryption,
};

pub fn encrypt_payload<T, V>(encryption: &T, value: &V) -> eyre::Result<String, ErrorMessage>
where
    T: Encryption + ?Sized,
    V: serde::Serialize,
{
    match serde_json::to_string(value) {
        Ok(str) => {
            return encryption.encrypt(&str);
        }
        Err(err) => {
            error!("payload serialization failed: {:?}", err);
            return Err("JSON payload serialization failed".into());
        }
    }
}
