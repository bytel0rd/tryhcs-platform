use either::Either;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct ErrorMessage(pub String);
impl<T: AsRef<str>> From<T> for ErrorMessage {
    fn from(value: T) -> Self {
        ErrorMessage(format!("{}", value.as_ref()))
    }
}

pub type ApiResponse<T> = (StatusCode, Either<Option<T>, ErrorMessage>);

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct ApiResponseData<T> {
    pub data: T,
    pub status_code: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct ApiResponseError {
    pub error_message: Option<String>,
    pub message: Option<String>,
}

impl Into<ErrorMessage> for ApiResponseError {
    fn into(self) -> ErrorMessage {
        if let Some(message) = self.error_message {
            return ErrorMessage(message);
        }

        if let Some(message) = self.message {
            return ErrorMessage(message);
        }

        return ErrorMessage("#Error".into());
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub struct PaginatedResult<T> {
    pub page_size: i32,
    pub page_number: i32,
    pub total: i64,
    pub data: Vec<T>,
    pub status: u16,
    pub error_message: Option<String>
}

impl <T> Default for PaginatedResult<T> {
    fn default() -> Self {
        Self { page_size: 0,
             page_number: 0, total: 0, data: vec![], status: 500, 
             error_message: None }
    }
}

impl<T: Serialize> Into<serde_json::Value> for PaginatedResult<T> {
    fn into(self) -> serde_json::Value {
        serde_json::json!(&self)
    }
}

#[derive(Serialize, Deserialize, Debug, TS, Default, Clone)]
#[ts(export)]
pub struct PaginatedQuery {
    pub query: Option<String>,
    pub page_size: Option<i32>,
    pub page_number: Option<i32>,
}
