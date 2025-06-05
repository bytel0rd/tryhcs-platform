use reqwest::StatusCode;

pub mod api_response;
pub mod auth;
pub mod data_encryption;
pub mod encryption_context;
pub mod env;
pub mod file_upload;
pub mod redis;
pub mod utils;

pub const ADMIN_DOMAIN: &str = "Admin";
pub const ADMIN_ROLE: &str = "Superuser";

pub const SUCCESS_API_STATUS_CODE: StatusCode = StatusCode::OK;
pub const BAD_REQUEST_API_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;
pub const UNAUTHORIZED_API_STATUS_CODE: StatusCode = StatusCode::UNAUTHORIZED;
pub const FORBIDDEN_API_STATUS_CODE: StatusCode = StatusCode::FORBIDDEN;
pub const NOT_FOUND_API_STATUS_CODE: StatusCode = StatusCode::NOT_FOUND;
pub const DUPLICATE_API_STATUS_CODE: StatusCode = StatusCode::CONFLICT;

pub static AUTH_ID_HEADER_FIELD: &str = "Authorization";

pub static WORKSPACE_CODE_HEADER_FIELD: &str = "Workspace";
