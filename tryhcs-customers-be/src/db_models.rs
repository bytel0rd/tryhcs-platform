use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tryhcs_shared::institution_params::*;
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug, Clone, Builder, sqlx::FromRow)]
pub struct Staff {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub shadow_id: String,
    pub institution_id: Option<i64>,
    pub profile_image: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<Staff> for StaffDto {
    fn from(s: Staff) -> Self {
        StaffDto {
            id: s.shadow_id,
            first_name: s.first_name,
            last_name: s.last_name,
            mobile: s.mobile,
            title: s.title,
            institution_id: s.institution_id,
            profile_image: s.profile_image,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub mobile: String,
    pub password: String,
    pub failed_attempts: i32,
    pub device_ids: Vec<String>,
    pub shadow_id: String,
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
    pub shadow_id: String,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<Institution> for InstitutionDto {
    fn from(i: Institution) -> Self {
        InstitutionDto {
            id: i.shadow_id,
            px: i.id,
            institution_name: i.name,
            email: i.email,
            classification: i.classification,
            workspace_code: i.workspace_code,
            setting: i.setting,
            address: i.address,
            town: i.town,
            state: i.state,
            logo: i.logo,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Builder, sqlx::FromRow)]
pub struct Department {
    pub id: i64,
    pub name: String,
    pub domain: String,
    pub institution_id: i64,
    pub head_staff_id: Option<String>,
    pub shadow_id: String,
    pub staffs_ids: serde_json::Value,
    pub phone_no: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<Department> for DepartmentDto {
    fn from(d: Department) -> Self {
        DepartmentDto {
            id: d.shadow_id,
            name: d.name,
            institution_id: d.institution_id,
            created_at: d.created_at,
            modified_at: d.modified_at,
        }
    }
}
