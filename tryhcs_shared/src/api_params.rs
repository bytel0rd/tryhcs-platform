use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use either::Either;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage(pub String);
impl <T: AsRef<str>> From<T> for ErrorMessage {
    fn from(value: T) -> Self {
        ErrorMessage(format!("{}", value.as_ref()))
    }
}


pub type ApiResponse<T> = (StatusCode, Either<Option<T>, ErrorMessage>);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  ApiResponseData<T> {
    pub data: T,
    pub  status: u16
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  ApiResponseError {
    pub error_message: Option<String>,
    pub message: Option<String>,
    pub  status: u16
}

impl Into<ErrorMessage> for ApiResponseError  {
    fn into(self) -> ErrorMessage {
        if let Some(message) = self.error_message  {
            return ErrorMessage(message);
        }

        if let Some(message) = self.message {
            return ErrorMessage(message);
        }

       return ErrorMessage("#Error".into());
    }
}
