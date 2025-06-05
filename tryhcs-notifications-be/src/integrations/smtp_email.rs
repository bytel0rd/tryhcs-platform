use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use tryhcs_commons_be::env::EnvConfig;

#[derive(Serialize, Deserialize, Debug, Default)]

pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub content: String,
}

pub async fn send_email(env: &EnvConfig, message: EmailMessage) -> eyre::Result<()> {
    dbg!(&message);

    if true {
        return Ok(());
    }

    let email = Message::builder()
        .from(env.no_reply_email_address.parse()?)
        // .reply_to("Yuin <yuin@domain.tld>".parse()?)
        .to(message.to.parse()?)
        .subject(&message.subject)
        .header(ContentType::TEXT_HTML)
        .body(message.content.to_owned())
        .unwrap();

    let creds = Credentials::new(
        env.no_reply_email_address.to_owned(),
        env.no_reply_email_password.to_owned(),
    );
    let mailer = SmtpTransport::relay(&env.smtp_server)?
        .credentials(creds)
        .build();

    let result = mailer.send(&email)?;

    Ok(())
}
