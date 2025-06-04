use std::str::FromStr;

use reqwest::header;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use tracing::info;

use self::params::SendMessage;

#[derive(Clone, Debug)]
pub struct SendchampApi<'a> {
    pub base_url: &'a str,
    pub api_key: &'a str,
    pub sendchamp_sender_id: &'a str,
}

mod params {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Clone)]
    pub(super) struct SendMessage {
        pub to: String,
        pub message: String,
        pub sender_name: String,
        pub route: String,
    }
}

impl<'a> SendchampApi<'a> {
    pub async fn send_message(&self, recipient: &str, message: &str) -> eyre::Result<()> {
        let client = self.create_client()?;

        let url = format!("{}/api/v1/sms/send", self.base_url);

        let body = SendMessage {
            to: recipient.to_owned(),
            message: message.to_owned(),
            sender_name: self.sendchamp_sender_id.to_owned(),
            route: "dnd".to_owned(),
        };
        let body = serde_json::to_string(&body)?;
        info!(message ="send sms req", url=&url, body= ?body);

        let response = client.post(url.as_str()).body(body).send().await?;
        let response = response.text().await?;
        info!(message="send sms response", response=?response);

        Ok(())
    }

    fn create_client(&self) -> eyre::Result<Client> {
        let mut headers = header::HeaderMap::new();

        let api_key = format!("Bearer {}", self.api_key);
        let content_type = mime::APPLICATION_JSON.to_string();

        headers.insert(
            AUTHORIZATION,
            header::HeaderValue::from_str(api_key.as_str())?,
        );
        headers.insert(
            CONTENT_TYPE,
            header::HeaderValue::from_str(content_type.as_str())?,
        );

        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(client)
    }
}

#[cfg(test)]
mod tests {

    use tryhcs_commons_be::env::EnvConfig;

    use super::*;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let env = envy::from_env::<EnvConfig>()?;

        tracing_subscriber::fmt::init();

        let api = get_api(&env);

        api.send_message("+2348149464288", "Testing message")
            .await
            .expect("failed sms");

        Ok(())
    }

    fn get_api<'a>(env: &'a EnvConfig) -> SendchampApi<'a> {
        let api = SendchampApi {
            base_url: env.sendchamp_base_url.as_str(),
            api_key: env.sendchamp_api_key.as_str(),
            sendchamp_sender_id: env.sendchamp_sender_id.as_str(),
        };

        api
    }
}
