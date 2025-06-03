use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::params::DepartmentMember;


#[derive(Serialize, Deserialize, Debug, Clone, Builder, sqlx::FromRow)]
pub struct Staff {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub institution_id: Option<i64>,
    pub profile_image: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub mobile: String,
    pub password: String,
    pub failed_attempts: i32,
    pub device_ids: Vec<String>,
    pub last_login_time: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}


#[derive(Serialize, Deserialize, Debug, Builder, sqlx::FromRow, Clone)]
pub struct Institution {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub classification: String,
    pub setting: String,
    pub address: Option<String>,
    pub town: Option<String>,
    pub state: Option<String>,
    pub created_by: i64,
    pub logo: Option<String>,
    pub workspace_code: String,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Builder, sqlx::FromRow)]
pub struct Department {
    pub id: i64,
    pub name: String,
    pub domain: String,
    pub institution_id: i64,
    pub head_staff_id: Option<i64>,
    pub staffs_ids: Vec<serde_json::Value>,
    pub phone_no: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
