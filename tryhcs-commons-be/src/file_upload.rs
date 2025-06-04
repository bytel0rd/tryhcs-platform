use aws_config::Region;
use aws_sdk_s3::{
    config::{Credentials, SharedCredentialsProvider},
    primitives::ByteStream,
    Client,
};

use aws_sdk_s3::presigning::PresigningConfig;
use base64::prelude::{Engine as _, BASE64_STANDARD_NO_PAD, BASE64_URL_SAFE};
use bon::Builder;
use bytes::Bytes;
use chrono::format;
use either::Either;
use eyre::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;
use tryhcs_shared::{APIFileUpload, APIFileUploadResponse};

use crate::env::EnvConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct UploadedFile {
    pub base64: String,
    pub content_type: String,
    pub name: Option<String>,
}

pub async fn get_upload_client(config: &EnvConfig) -> eyre::Result<Client> {
    let EnvConfig {
        cloudflare_r2_url,

        cloudflare_r2_access_key_id,
        cloudflare_r2_secret_access_key,
        ..
    } = config;
    let credentials = Credentials::new(
        cloudflare_r2_access_key_id,
        cloudflare_r2_secret_access_key,
        None,
        None,
        "custom",
    );
    let r2_config = aws_config::from_env()
        .region(Region::new("auto"))
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .endpoint_url(cloudflare_r2_url)
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&r2_config);
    Ok(client)
}

pub async fn upload_file_to_bucket<S: Into<String>>(
    client: &Client,
    bucket: S,
    remote_path: S,
    file_content: &[u8],
    content_type: Option<String>,
) -> eyre::Result<()> {
    let remote_path: String = remote_path.into();
    client
        .put_object()
        .bucket(bucket.into())
        .key(&remote_path)
        // replace the copy allocation
        .body(ByteStream::from(Bytes::copy_from_slice(file_content)))
        .set_content_type(content_type)
        .send()
        .await
        .wrap_err("failed to upload file to bucket")?;

    info!("Uploaded: {} successfully to bucket", remote_path);
    Ok(())
}

pub async fn get_file<S: Into<String>>(
    client: &Client,
    bucket: S,
    remote_path: S,
) -> eyre::Result<ByteStream> {
    let remote_path = remote_path.into();
    let response = client
        .get_object()
        .bucket(bucket.into())
        .key(&remote_path)
        .send()
        .await
        .wrap_err("failed to fetch file to bucket")?;

    info!("Fetch: {} successfully to bucket", remote_path);

    Ok(response.body)
}

// #[tokio::test]
// async fn file_upload_s3_test() {
//     dotenv::dotenv().ok();

//     tracing_subscriber::fmt().init();

//     let env = envy::from_env::<EnvConfig>().expect("loaded config files");

//     let client = get_upload_client(&env)
//         .await
//         .expect("bucket initalization failed");

//     info!("initalized client configuration");
//     upload_file_to_bucket(
//         &client,
//         &env.cloudflare_r2_bucket,
//         &"init.sql".to_string(),
//         b"SELECT 1;",
//         None
//     )
//     .await
//     .expect("failed to upload file");
// }

pub async fn upload_base64_file<S: AsRef<str> + Into<String>>(
    client: &Client,
    bucket: S,
    path: S,
    content: S,
    content_type: Option<String>,
) -> eyre::Result<()> {
    info!("uploading file...");
    let content = base64::decode(content.as_ref().trim().as_bytes())
        .wrap_err("failed to decrypt file to base64")?;
    return upload_file_to_bucket(client, bucket, path, &content, content_type).await;
}

pub async fn get_presigned_object_url(
    client: &Client,
    bucket: &str,
    object: &str,
    expires_in: u64,
) -> eyre::Result<String> {
    let expires_in = Duration::from_secs(expires_in);
    let presigned_request = client
        .get_object()
        .bucket(bucket)
        .key(object)
        .presigned(PresigningConfig::expires_in(expires_in)?)
        .await?;

    Ok(presigned_request.uri().to_string())
}

// #[tokio::test]
// async fn decode_base64() {
//     dotenv::dotenv().ok();

//     tracing_subscriber::fmt().init();

//     let env = envy::from_env::<EnvConfig>().expect("loaded config files");

//     let base_64_img = include_str!("../assets/tests/base64_file.txt");

//     let r =
//         base64::decode(base_64_img.trim().as_bytes()).expect("failed to decrypt file to base64");
// }
