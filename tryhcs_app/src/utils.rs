use either::Either;
use serde::de::DeserializeOwned;
use tracing::{debug, error};
use tryhcs_shared::{api_params::{ApiResponseError, ErrorMessage}, encryption::Encryption};

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

pub async fn extract_response(
    req_result: Result<reqwest::Response, reqwest::Error>,
) -> eyre::Result<Either<String, ErrorMessage>, ErrorMessage> {
    match req_result {
        Err(error) => {
            error!(message="Request failure: ", url=?error.url(), error=?error);
            return Err("failed to send request".into());
        }
        Ok(response) => {
            let is_success = response.status().is_success();
            let response_str = {
                match response.text().await {
                    Err(err) => {
                        error!(message="Failed to read response body ", error=?err);
                        return Err("Failed to read response body".into());
                    }
                    Ok(raw_str) => raw_str,
                }
            };
            if !is_success {
                return Ok(Either::Right(ErrorMessage(response_str)));
            }
            return Ok(Either::Left(response_str));
        }
    }
}

pub async fn decrypt_response<A: Encryption + ?Sized, T: DeserializeOwned, E: DeserializeOwned + Into<ErrorMessage>>(
    encryption: &A,
    response: &Either<String, ErrorMessage>,
) -> eyre::Result<T, ErrorMessage> {
    match response {
        Either::Left(success) => {
            let result = encryption.decrypt(&success)?;
            match serde_json::from_str::<T>(&result) {
                Err(e) => {
                    error!(message="Success response deserialization failed. ", err=?e);
                    return Err("Deserialization failed".into());
                }
                Ok(v) => {
                    return Ok(v);
                }
            };
        }
        Either::Right(ErrorMessage(error)) => {
            let result = encryption.decrypt(&error)?;
            match serde_json::from_str::<E>(&result) {
                Err(e) => {
                    error!(message="Failure deserialization failed. ", err=?e);
                    return Err("Deserialization failed".into());
                }
                Ok(v) => {
                    return Err(v.into());
                }
            };
        }
    }
}


pub async fn extract_hcs_response<A: Encryption + ?Sized, T: DeserializeOwned>(encryption: &A, req_result: Result<reqwest::Response, reqwest::Error>,) -> eyre::Result<T, ErrorMessage> {
        let encrypted_response = extract_response(req_result).await?;
        decrypt_response::<A, T, ApiResponseError>(encryption, &encrypted_response).await
}