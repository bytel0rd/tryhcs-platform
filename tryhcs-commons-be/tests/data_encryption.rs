use std::sync::Once;

use aes_gcm::{AeadCore, KeyInit};
use serde_json::json;
use tracing::info;
use tryhcs_commons_be::data_encryption::{
    DeterministicEncrypted, EncryptableData, NonDeterministicEncrypted,
};

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_test_writer() // optional, routes to test framework output
            .try_init();
    });
}

const AES_GSM_ENCRYPTION_KEY: &[u8] = &[
    92, 253, 227, 130, 126, 68, 219, 61, 189, 209, 242, 195, 48, 127, 146, 8, 3, 189, 138, 137,
    113, 169, 138, 95, 116, 85, 164, 232, 187, 253, 216, 67,
];
const AES_GSM_ENCRYPTION_NONCE: &[u8] = &[44, 192, 77, 210, 77, 9, 86, 21, 44, 86, 43, 166];
const RAW_PLAIN_TEXT: &str = "THE ONE PIECE";
const ENCRYPTED_RAW_TEST_DETERMINIC_VALUE: &[u8] = &[
    213, 193, 8, 143, 230, 164, 17, 168, 169, 34, 255, 4, 238, 73, 176, 85, 173, 87, 242, 138, 81,
    246, 157, 230, 193, 15, 202, 47, 35, 89, 120, 187, 46, 254, 32, 13, 229, 125, 180, 64,
];

#[test]
fn should_encrypt_deterministic() {
    init_logger();

    let mut derministic_field = DeterministicEncrypted::from_raw(RAW_PLAIN_TEXT.to_string());
    derministic_field
        .encrypt(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to encrypt deterministic field");

    assert!(derministic_field.is_encrypted());

    let encrypted_data = derministic_field
        .get_encrypted_data(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to get the encrypted data");
    assert_eq!(
        ENCRYPTED_RAW_TEST_DETERMINIC_VALUE,
        encrypted_data.as_slice()
    )
}

#[test]
fn should_decrypt_deterministic() {
    init_logger();

    let mut derministic_field: DeterministicEncrypted<String> =
        DeterministicEncrypted::from_encrypted(ENCRYPTED_RAW_TEST_DETERMINIC_VALUE.to_vec());
    derministic_field
        .decrypt(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to decrypt deterministic field");

    assert!(!derministic_field.is_encrypted());

    let encrypted_data = derministic_field
        .get_data(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to get the decrypted data");
    assert_eq!(RAW_PLAIN_TEXT, &encrypted_data);
}

#[test]
fn should_encrypt_and_decrypt_non_deterministic() {
    init_logger();

    let mut non_derministic_field = NonDeterministicEncrypted::from_raw(RAW_PLAIN_TEXT.to_string());
    non_derministic_field
        .encrypt(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to encrypt deterministic field");

    assert!(non_derministic_field.is_encrypted());

    let encrypted_data = non_derministic_field
        .get_encrypted_data(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to get encrypted data");

    non_derministic_field = NonDeterministicEncrypted::from_encrypted(encrypted_data)
        .expect("Failed to serialize from not encrypted field");
    non_derministic_field
        .decrypt(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to decrypt deterministic field");

    assert!(!non_derministic_field.is_encrypted());

    let decrypted_data = non_derministic_field
        .get_data(AES_GSM_ENCRYPTION_KEY, AES_GSM_ENCRYPTION_NONCE)
        .expect("Failed to get the decrypted data");
    assert_eq!(RAW_PLAIN_TEXT, &decrypted_data);
}
