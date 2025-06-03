use async_trait::async_trait;
use bon::Builder;
use chrono::{DateTime, Utc};
use eyre::{Context, Ok, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, query_as, Executor, PgPool};

#[derive(Serialize, Deserialize, Debug)]
pub struct NewStaff {
    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub profile_image: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Builder)]
pub struct CreateInstitution {
    pub institution_name: String,
    pub email: String,
    pub classification: String,
    pub setting: String,
    pub address: Option<String>,
    pub town: Option<String>,
    pub state: Option<String>,

    pub first_name: String,
    pub last_name: String,
    pub mobile: String,
    pub title: String,
    pub password: String,

    pub logo: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Builder)]
pub struct DepartmentMember {
    pub staff_id: i64,
    pub role: String,
}



