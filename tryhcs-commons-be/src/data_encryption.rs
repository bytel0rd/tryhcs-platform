use std::fmt::Debug;

use aes_gcm::{
    aead::{Aead, AeadMutInPlace},
    AeadCore, Aes256Gcm, KeyInit,
};
use eyre::eyre;
use lettre::transport::smtp::response;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::error;

pub trait EncryptableData {
    type Data: Serialize + DeserializeOwned;
    fn encrypt(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<()>;
    fn decrypt(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<()>;
    fn get_data(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<Self::Data>;
    fn get_encrypted_data(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<Vec<u8>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum DeterministicField<T> {
    V1(SecretField<T>), // using Hmac algorithm
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SecretFieldInternalRep<T> {
    pub data: T,
}

impl<T> SecretFieldInternalRep<T> {
    pub(crate) fn new(data: T) -> Self {
        SecretFieldInternalRep { data }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum SecretField<T> {
    Encrypted(Vec<u8>),
    Decrypted(SecretFieldInternalRep<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicEncrypted<T> {
    pub(crate) field_data: DeterministicField<T>,
}

impl<T> DeterministicEncrypted<T> {
    pub fn from_raw(data: T) -> Self {
        DeterministicEncrypted {
            field_data: DeterministicField::V1(SecretField::Decrypted(
                SecretFieldInternalRep::new(data),
            )),
        }
    }

    pub fn from_encrypted(data: Vec<u8>) -> Self {
        DeterministicEncrypted {
            field_data: DeterministicField::V1(SecretField::Encrypted(data)),
        }
    }

    pub fn is_encrypted(&self) -> bool {
        match &self.field_data {
            DeterministicField::V1(secret_field) => match secret_field {
                SecretField::Encrypted(_) => true,
                SecretField::Decrypted(_) => false,
            },
        }
    }
}

impl<T: Serialize + DeserializeOwned + Clone> EncryptableData for DeterministicEncrypted<T> {
    type Data = T;
    fn encrypt(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<()> {
        let encrypted = {
            match &self.field_data {
                DeterministicField::V1(secret_field) => match secret_field {
                    SecretField::Encrypted(_) => {
                        return Err(eyre!("Data is already encrypted, can't silently fail due to a possible key mismatch"));
                    }
                    SecretField::Decrypted(d) => {
                        let cipher = aes_gcm_siv::Aes256GcmSiv::new(key.try_into()?);
                        let nonce = aes_gcm_siv::Nonce::from_slice(nonce);
                        let data = serde_json::to_string(d)?;
                        match cipher.encrypt(nonce, data.as_bytes()) {
                            Err(error_message) => {
                                error!(message="Deterministic field encryption failed", err=?error_message);
                                return Err(eyre!("Deterministic field encryption failed"));
                            }
                            Ok(data) => data,
                        }
                    }
                },
            }
        };

        dbg!(&encrypted);
        self.field_data = DeterministicField::V1(SecretField::Encrypted(encrypted));
        Ok(())
    }

    fn decrypt(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<()> {
        let decrypted = {
            match &self.field_data {
                DeterministicField::V1(secret_field) => match secret_field {
                    SecretField::Decrypted(_) => {
                        return Err(eyre!("Data is already encrypted, can't silently fail due to a possible key mismatch or user expectation"));
                    }
                    SecretField::Encrypted(encrypted_data) => {
                        let cipher = aes_gcm_siv::Aes256GcmSiv::new(key.try_into()?);
                        let nonce = aes_gcm_siv::Nonce::from_slice(nonce);
                        match cipher.decrypt(nonce, encrypted_data.iter().as_slice()) {
                            Err(error_message) => {
                                error!(message="Deterministic field decryption failed", err=?error_message);
                                return Err(eyre!("Deterministic field decryption failed"));
                            }
                            Ok(data) => serde_json::from_slice(&data)?,
                        }
                    }
                },
            }
        };

        self.field_data = DeterministicField::V1(SecretField::Decrypted(decrypted));
        Ok(())
    }

    fn get_data(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<T> {
        let data_is_encrypted_before = self.is_encrypted();
        if data_is_encrypted_before {
            let _ = self.decrypt(key, nonce);
        }

        let result = match &self.field_data {
            DeterministicField::V1(secret_field) => {
                match secret_field {
                    // already tried decrypting ahead
                    SecretField::Encrypted(_) => {
                        return Err(eyre!("Determinisitc field is still encrypted"));
                    }
                    SecretField::Decrypted(secret_field_internal_rep) => {
                        Ok(secret_field_internal_rep.data.clone())
                    }
                }
            }
        };

        if data_is_encrypted_before {
            self.encrypt(key, nonce)?;
        }
        result
    }

    fn get_encrypted_data(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<Vec<u8>> {
        let data_is_encrypted_before = self.is_encrypted();
        if !data_is_encrypted_before {
            let _ = self.encrypt(key, nonce);
        }

        let result = match &self.field_data {
            DeterministicField::V1(secret_field) => {
                match secret_field {
                    // already tried decrypting ahead
                    SecretField::Decrypted(_) => {
                        return Err(eyre!("Determinisitc field is still unencrypted"));
                    }
                    SecretField::Encrypted(data) => Ok(data.clone()),
                }
            }
        };

        if !data_is_encrypted_before {
            self.encrypt(key, nonce)?;
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum NonDeterministicField<T> {
    V1 {
        data_encyption_key: SecretField<Vec<u8>>,
        nonce: Vec<u8>,
        data: SecretField<T>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonDeterministicEncrypted<T> {
    pub(crate) field_data: NonDeterministicField<T>,
}

impl<T: DeserializeOwned> NonDeterministicEncrypted<T> {
    pub fn from_raw(data: T) -> Self {
        let key = aes_gcm::Aes256Gcm::generate_key(aes_gcm::aead::OsRng).to_vec(); // Safe to use because the key is never repeated
        let nonce = aes_gcm::Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng).to_vec();
        NonDeterministicEncrypted {
            field_data: NonDeterministicField::V1 {
                data: SecretField::Decrypted(SecretFieldInternalRep::new(data)),
                data_encyption_key: SecretField::Decrypted(SecretFieldInternalRep::new(key)),
                nonce,
            },
        }
    }

    pub fn from_encrypted(data: Vec<u8>) -> eyre::Result<Self> {
        // The encrypted data should be serializable into NonDeterministicField<T>
        let field = serde_json::from_slice::<NonDeterministicField<T>>(&data)?;
        Ok(Self { field_data: field })
    }

    pub fn is_encrypted(&self) -> bool {
        match &self.field_data {
            NonDeterministicField::V1 {
                data,
                data_encyption_key,
                nonce: _,
            } => match (data, data_encyption_key) {
                (SecretField::Decrypted(_), SecretField::Decrypted(_)) => false,
                _ => true,
            },
        }
    }
}

impl<T: Serialize + DeserializeOwned + Clone> EncryptableData for NonDeterministicEncrypted<T> {
    type Data = T;

    fn encrypt(
        &mut self,
        key_encryption_key: &[u8],
        key_encryption_nonce: &[u8],
    ) -> eyre::Result<()> {
        let (encrypted_data, encrypted_data_key, data_nonce) = {
            match &self.field_data {
                NonDeterministicField::V1 {
                    data_encyption_key,
                    data,
                    nonce,
                } => match data {
                    SecretField::Encrypted(_) => {
                        return Err(eyre!("Data is already encrypted, can't silently fail due to a possible key mismatch"));
                    }
                    SecretField::Decrypted(decrypted_data) => {
                        // encrypt the data with the DEK
                        // encrypt the DEK with KEK

                        let data_encryption_key = match data_encyption_key {
                            SecretField::Decrypted(key) => key.data.clone(),
                            // decrypt the key with the KEK
                            SecretField::Encrypted(encrypted_data_key) => {
                                let cipher =
                                    aes_gcm_siv::Aes256GcmSiv::new(key_encryption_key.try_into()?);
                                match cipher.decrypt(
                                    key_encryption_nonce.try_into()?,
                                    encrypted_data_key.as_slice(),
                                ) {
                                    Err(error_message) => {
                                        error!(message="Nondeterministic data key decryption failed", err=?error_message);
                                        return Err(eyre!(
                                            "Nondeterministic data key decryption failed"
                                        ));
                                    }
                                    Ok(data) => data,
                                }
                            }
                        };

                        let data_field_cipher =
                            aes_gcm::Aes256Gcm::new(data_encryption_key.as_slice().try_into()?);
                        let data_nonce = aes_gcm::Nonce::from_slice(&nonce);
                        let data = serde_json::to_string(decrypted_data)?;
                        match data_field_cipher.encrypt(data_nonce, data.as_bytes()) {
                            Err(error_message) => {
                                error!(message="Nondeterministic field encryption failed", err=?error_message);
                                return Err(eyre!("Nondeterministic field encryption failed"));
                            }
                            Ok(data) => {
                                // encrypt the data key
                                let data_key_cipher =
                                    aes_gcm_siv::Aes256GcmSiv::new(key_encryption_key.try_into()?);

                                let encrypted_data_key = match data_key_cipher.encrypt(
                                    key_encryption_nonce.try_into()?,
                                    data_encryption_key.as_slice(),
                                ) {
                                    Err(error_message) => {
                                        error!(message="Nondeterministic data key encryption failed", err=?error_message);
                                        return Err(eyre!(
                                            "Nondeterministic data key encryption failed"
                                        ));
                                    }
                                    Ok(data) => data,
                                };

                                (data, encrypted_data_key, nonce.to_owned())
                            }
                        }
                    }
                },
            }
        };

        self.field_data = NonDeterministicField::V1 {
            data: SecretField::Encrypted(encrypted_data),
            data_encyption_key: SecretField::Encrypted(encrypted_data_key),
            nonce: data_nonce,
        };
        Ok(())
    }

    fn decrypt(
        &mut self,
        key_encryption_key: &[u8],
        key_encryption_nonce: &[u8],
    ) -> eyre::Result<()> {
        let (decrypted, data_encyption_key, nonce) = {
            match &self.field_data {
                NonDeterministicField::V1 {
                    data: secret_field,
                    data_encyption_key,
                    nonce,
                } => {
                    match secret_field {
                        SecretField::Decrypted(_) => {
                            return Err(eyre!("Data is already decrypted, can't silently fail due to a possible key mismatch or user expectation"));
                        }
                        SecretField::Encrypted(encrypted_data) => {
                            // Decrypt the DEK with KEK
                            // Decrypt the data with DEK

                            let data_encryption_key = match data_encyption_key {
                                SecretField::Decrypted(key) => key.data.clone(),
                                // decrypt the key with the KEK
                                SecretField::Encrypted(encrypted_data_key) => {
                                    let cipher = aes_gcm_siv::Aes256GcmSiv::new(
                                        key_encryption_key.try_into()?,
                                    );
                                    match cipher.decrypt(
                                        key_encryption_nonce.try_into()?,
                                        encrypted_data_key.as_slice(),
                                    ) {
                                        Err(error_message) => {
                                            error!(message="Nondeterministic data key decryption failed", err=?error_message);
                                            return Err(eyre!(
                                                "Nondeterministic data key decryption failed"
                                            ));
                                        }
                                        Ok(data) => data,
                                    }
                                }
                            };

                            let data_field_cipher =
                                aes_gcm::Aes256Gcm::new(data_encryption_key.as_slice().try_into()?);
                            let data_encryption_nonce = aes_gcm::Nonce::from_slice(nonce);
                            let decrypted_data = {
                                match data_field_cipher.decrypt(
                                    data_encryption_nonce,
                                    encrypted_data.iter().as_slice(),
                                ) {
                                    Err(error_message) => {
                                        error!(message="Field decryption failed", err=?error_message);
                                        return Err(eyre!("Field decryption failed"));
                                    }
                                    Ok(data) => {
                                        serde_json::from_slice::<SecretFieldInternalRep<T>>(&data)?
                                            .data
                                    }
                                }
                            };
                            (decrypted_data, data_encryption_key, nonce.clone())
                        }
                    }
                }
            }
        };

        self.field_data = NonDeterministicField::V1 {
            data: SecretField::Decrypted(SecretFieldInternalRep::new(decrypted)),
            data_encyption_key: SecretField::Decrypted(SecretFieldInternalRep::new(
                data_encyption_key,
            )),
            nonce,
        };
        Ok(())
    }

    fn get_data(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<T> {
        let data_is_encrypted_before = self.is_encrypted();
        if data_is_encrypted_before {
            let _ = self.decrypt(key, nonce);
        }

        let result = match &self.field_data {
            NonDeterministicField::V1 {
                data,
                data_encyption_key: _,
                nonce: _,
            } => {
                match data {
                    // already tried decrypting ahead
                    SecretField::Encrypted(_) => {
                        return Err(eyre!("Determinisitc field is still encrypted"));
                    }
                    SecretField::Decrypted(secret_field_internal_rep) => {
                        Ok(secret_field_internal_rep.data.clone())
                    }
                }
            }
        };

        if data_is_encrypted_before {
            self.encrypt(key, nonce)?;
        }

        result
    }

    fn get_encrypted_data(&mut self, key: &[u8], nonce: &[u8]) -> eyre::Result<Vec<u8>> {
        let data_is_encrypted_before = self.is_encrypted();
        if !data_is_encrypted_before {
            let _ = self.encrypt(key, nonce)?;
        }

        let result = match &self.field_data {
            NonDeterministicField::V1 {
                data,
                data_encyption_key: _,
                nonce: _,
            } => {
                match data {
                    // already tried decrypting ahead
                    SecretField::Decrypted(_) => {
                        return Err(eyre!("Determinisitc field is still unencrypted"));
                    }
                    SecretField::Encrypted(_) => {
                        let data = serde_json::to_vec(&self.field_data)?;
                        Ok(data)
                    }
                }
            }
        };

        if !data_is_encrypted_before {
            self.decrypt(key, nonce)?;
        }

        result
    }
}

#[derive(Clone)]
pub struct Encryptor {
    key: Vec<u8>,
    nonce: Vec<u8>,
}

impl Encryptor {
    pub fn new(key: Vec<u8>, nonce: Vec<u8>) -> Self {
        Encryptor { key, nonce }
    }

    pub fn generate() -> Self {
        let key = aes_gcm::Aes256Gcm::generate_key(aes_gcm::aead::OsRng).to_vec(); // Safe to use because the key is never repeated
        let nonce = aes_gcm::Aes256Gcm::generate_nonce(&mut aes_gcm::aead::OsRng).to_vec();

        Encryptor { key, nonce }
    }

    pub fn set_deterministic<T: Serialize + DeserializeOwned + Clone>(
        &self,
        val: T,
    ) -> eyre::Result<Vec<u8>> {
        let Encryptor { key, nonce } = &self;
        DeterministicEncrypted::from_raw(val).get_encrypted_data(&key, &nonce)
    }

    pub fn set_non_deterministic<T: Serialize + DeserializeOwned + Clone>(
        &self,
        val: T,
    ) -> eyre::Result<Vec<u8>> {
        let Encryptor { key, nonce } = &self;
        NonDeterministicEncrypted::from_raw(val).get_encrypted_data(&key, &nonce)
    }

    pub fn get_deterministic<T: Serialize + DeserializeOwned + Clone>(
        &self,
        val: Vec<u8>,
    ) -> eyre::Result<T> {
        let Encryptor { key, nonce } = &self;
        DeterministicEncrypted::from_encrypted(val).get_data(&key, &nonce)
    }

    pub fn get_non_deterministic<T: Serialize + DeserializeOwned + Clone>(
        &self,
        val: Vec<u8>,
    ) -> eyre::Result<T> {
        let Encryptor { key, nonce } = &self;
        NonDeterministicEncrypted::from_encrypted(val)?.get_data(&key, &nonce)
    }
}

pub trait Encryptable {
    fn encrypt_deterministic(&self, enc: &Encryptor) -> eyre::Result<Vec<u8>>;
    fn encrypt_randomized(&self, enc: &Encryptor) -> eyre::Result<Vec<u8>>;
}

impl<T> Encryptable for T
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn encrypt_deterministic(&self, enc: &Encryptor) -> eyre::Result<Vec<u8>> {
        enc.set_deterministic(self.clone())
    }

    fn encrypt_randomized(&self, enc: &Encryptor) -> eyre::Result<Vec<u8>> {
        enc.set_non_deterministic(self.clone())
    }
}

pub trait Encrypted {
    type Output;

    fn encrypt(&self, encryptor: &Encryptor) -> eyre::Result<Self::Output>;
}

