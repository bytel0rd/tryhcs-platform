use crate::api_params::ErrorMessage;

// #[async_trait::async_trait]
pub trait Encryption: Send + Sync {
    fn encrypt(&self, data: &str) -> eyre::Result<String, ErrorMessage>;
    fn decrypt(&self, data: &str) -> eyre::Result<String, ErrorMessage>;
}

pub struct NoEncryption;
impl Encryption for NoEncryption {
    fn encrypt(&self, data: &str) -> eyre::Result<String, ErrorMessage> {
        Ok(data.to_string())
    }

    fn decrypt(&self, data: &str) -> eyre::Result<String, ErrorMessage> {
        Ok(data.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ClientEncryption {
    pub timestamp: i64,
    pub iv: String,
}

#[derive(Debug, Clone)]
pub struct PublicEncyption {
    pub public_key: String,
    pub timestamp: i64,
}
