use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use either::Either;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage(pub String);

pub type ApiResponse<T> = (StatusCode, Either<Option<T>, ErrorMessage>);
