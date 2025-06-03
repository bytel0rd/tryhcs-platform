use std::time;

use crate::{db_models::{Department, Institution, Staff, User}, params::{CreateInstitution, DepartmentMember, NewStaff}};
use async_trait::async_trait;
use bon::Builder;
use chrono::{DateTime, Utc};
use eyre::{Context, Ok, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{query, query_as, Executor, PgPool};
use tryhcs_commons_be::{data_encryption::{DeterministicEncrypted, Encryptable, EncryptableData, Encryptor, NonDeterministicEncrypted}, ADMIN_DOMAIN, ADMIN_ROLE};
use uuid::Uuid;


#[async_trait]
pub trait EhrDataRepo: Send + Sync {

 async fn find_institution_by_email(
    &self,
    email: &str,
) -> Result<Option<Institution>>;

 async fn create_institution(
    &self,
    c: CreateInstitution,
) -> Result<(Institution, Staff)>;



 async fn create_department(
    &self,
    institution_id: i64,
    dept_name: String,
    head_staff_id: Option<i64>,
    phone_no: Option<String>,
    staffs_ids: &[i64],
    domain: &str
) -> Result<Department>;

 async fn find_staff_institutions_by_mobile(
    &self,
    mobile: &str,
) -> Result<Vec<Institution>>;

 async fn find_staff_accounts_by_mobile(&self, mobile: &str) -> Result<Vec<Staff>>;

 async fn find_staff_by_mobile_and_insitution_id_opts(
    &self,
    institution_id: i64,
    mobile: &str,
) -> Result<Option<Staff>>;

 async fn find_institution_staff_by_id_opts(
    &self,
    institution_id: i64,
    staff_id: i64,
) -> Result<Option<Staff>>;

 async fn get_user(&self, mobile: &str) -> Result<Option<User>>;

 async fn record_failed_attempts_user(&self, mobile: &str) -> Result<User>;

 async fn record_user_login(&self, mobile: &str, device_id: &str) -> Result<User>;


 async fn edit_department(
    &self,
    department_id: i64,
    dept_name: String,
    head_staff_id: Option<i64>,
    phone_no: Option<String>,
    staffs_ids: &[DepartmentMember],
    domain: &str
) -> Result<Department>;


 async fn delete_department(&self, department_id: i64) -> Result<()>;

 async fn create_user(
    &self,
    mobile: &str,
    password: &str,
) -> Result<User>;

     async fn create_staff(&self, institution_id: i64, s: &NewStaff) -> Result<Staff>;
    
     async fn update_staff(&self, s: Staff) -> Result<Staff>;
    
     async fn delete_staff(&self, staff_id: i64) -> Result<()>;
    
     async fn find_institution_staffs(
        &self,
        institution_id: i64,
        search_query: Option<String>,
    ) -> Result<Vec<Staff>>;

     async fn find_staff_departments(
    &self,
    institution_id: i64,
    staff_id: i64,
) -> Result<Vec<Department>>;


 async fn find_institution_departments(
    &self,
    institution_id: i64,
    query: Option<String>,
) -> Result<Vec<Department>>;

 async fn get_institution_department(
    &self,
    institution_id: i64,
    department_id: i64,
) -> Result<Option<Department>>;


 async fn find_department_staffs(
    &self,
    institution_id: i64,
    department_id: i64,
) -> Result<Vec<Staff>>;


}

#[derive(Clone)]
pub struct CustomerDB {
    customer_db: PgPool,
    credential_db: PgPool,
    global_encryptor: Encryptor,
}

impl CustomerDB  {
    
    async fn get_institution_keys(&self, institution_id: &str) -> eyre::Result<(Vec<u8>, Vec<u8>)> {
        todo!()
    }

    async fn generate_institution_keys(&self, institution_id: &str) -> eyre::Result<(Vec<u8>, Vec<u8>)> {
        todo!()
    }

}



#[async_trait]
impl EhrDataRepo for CustomerDB {
     async fn create_staff(&self, institution_id: i64, s: &NewStaff) -> Result<Staff> {
        query_as!(
            Staff,
            "insert into staffs
            (first_name, last_name, mobile, title, institution_id,  profile_image)
        values
            ($1, $2, $3, $4, $5, $6)
        returning id, first_name, last_name, mobile, title, institution_id,  profile_image, deleted_at, modified_at, created_at ",
            s.first_name,
            s.last_name,
            s.mobile,
            s.title,
            Some(institution_id),
            s.profile_image
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Error creating staff")
    }
    
     async fn update_staff(&self, s: Staff) -> Result<Staff> {
        query_as!(
            Staff,
            "update staffs set
                first_name =$2 , last_name=$3, mobile=$4, title=$5, profile_image=$6
            where id = $1
            returning id, first_name, last_name, mobile, title, institution_id,  profile_image, deleted_at, modified_at, created_at",
            s.id,
            s.first_name,
            s.last_name,
            s.mobile,
            s.title,
            s.profile_image
        )
        .fetch_one(&self.customer_db)
        .await
        .wrap_err("Failed to update staff")
    }
    
     async fn delete_staff(&self, staff_id: i64) -> Result<()> {
        query!(
            "update staffs set
                deleted_at = Now()
            where id = $1",
            staff_id
        )
        .execute(&self.customer_db)
        .await
        .wrap_err("Failed to delete staff")?;
    
        Ok(())
    }
    
     async fn find_institution_staffs(
        &self,
        institution_id: i64,
        search_query: Option<String>,
    ) -> Result<Vec<Staff>> {
        query_as!(
            Staff,
            "select id, first_name, last_name, mobile, title, institution_id,  profile_image, deleted_at, modified_at, created_at from staffs
            where institution_id = $1 and
            ( ($2::varchar is null) or
              (first_name ilike concat('%', $2::varchar, '%')) or
              (last_name ilike concat('%', $2::varchar, '%')) or
              (title ilike concat('%', $2::varchar, '%')) or
              (mobile ilike concat('%', $2::varchar, '%'))
            )
            and deleted_at is null
            ORDER BY first_name asc
            ",
            institution_id,
            search_query
        )
        .fetch_all(&self.customer_db)
        .await
        .wrap_err("Failed to fetch institution staffs")
    }



 async fn create_institution(
    &self,
    c: CreateInstitution,
) -> Result<(Institution, Staff)> {
    let mut txn = self.customer_db.begin().await?;

    let workspace_code = format!("hcs_{}", Uuid::new_v4().to_string().replace("-", ""));

    let institution = query_as!(
        Institution,
        "insert into institutions
            (name, email, classification, setting, address, town, state, workspace_code)
        values
            ($1, $2, $3, $4, $5, $6, $7, $8)
        returning id, name, email, classification, setting, address, town, state, created_by, workspace_code, logo, deleted_at, modified_at, created_at",
        c.institution_name,
        c.email,
        c.classification,
        c.setting,
        c.address,
        c.town,
        c.state,
        workspace_code
    )
    .fetch_one(&mut *txn)
    .await
    .wrap_err("Unable to create institution")?;

    query!("insert into users (mobile, password) values ($1::varchar, $2::varchar) ON CONFLICT (mobile) DO UPDATE SET deleted_at = null, password = $2::varchar",
&c.mobile,
&c.password
).execute(&mut *txn)
.await
.wrap_err(format!("Failed to create user account"))?;

let admin_staff = query_as!(
    Staff,
    "insert into staffs
    (first_name, last_name, mobile, title, institution_id)
values
    ($1, $2, $3, $4, $5)
returning id, first_name, last_name, mobile, title, institution_id,  profile_image, deleted_at, modified_at, created_at",
    c.first_name,
    c.last_name,
    c.mobile,
    c.title,
    &institution.id,
)
.fetch_one(&mut *txn)
.await
.wrap_err("Error creating staff")?;

    let mut inital_staff_ids = json!(vec![DepartmentMember { staff_id: admin_staff.id, role: ADMIN_ROLE.to_string() }]);

    let department = query_as!(
            Department,
            "insert into departments
                (name, institution_id, staffs_ids, domain)
            values
                ($1, $2, $3, $4)
            returning id, name, institution_id, staffs_ids, head_staff_id, phone_no, deleted_at, modified_at, created_at,domain",
            ADMIN_DOMAIN,
            &institution.id,
            &inital_staff_ids,
            ADMIN_DOMAIN
        )
        .fetch_one(&mut *txn)
        .await
        .wrap_err(format!("Failed to create department"))?;

    

    txn.commit().await?;

    return Ok((institution, admin_staff));
}


 async fn create_department(
    &self,
    institution_id: i64,
    dept_name: String,
    head_staff_id: Option<i64>,
    phone_no: Option<String>,
    staffs_ids: &[i64],
    domain: &str
) -> Result<Department> {
    query_as!(
        Department,
        "insert into departments
            (name, institution_id, head_staff_id, phone_no, staffs_ids, domain)
        values
            ($1, $2, $3, $4, $5, $6)
        returning id, name, institution_id, staffs_ids, head_staff_id, phone_no, deleted_at, modified_at, created_at, domain",
        dept_name,
        institution_id,
        head_staff_id,
        phone_no,
        staffs_ids,
        domain
    )
    .fetch_one(&self.customer_db)
    .await
    .wrap_err(format!(
        "Failed to create department: {dept_name} institution_id: {institution_id}"
    ))
}

 async fn find_institution_by_email(
    &self,
    email: &str,
) -> Result<Option<Institution>> {
    query_as!(
        Institution,
        "select id, name, email, classification, setting, address, town, state, created_by, workspace_code, logo, deleted_at, modified_at, created_at from institutions
        where email = $1",
        email
    )
    .fetch_optional(&self.customer_db)
    .await
    .wrap_err(format!(
        "Failed to execute find institution: {}",
        email
    ))
}

 async fn find_institution_departments(
    &self,
    institution_id: i64,
    query: Option<String>,
) -> Result<Vec<Department>> {
    query_as!(
        Department,
        "select id, name, institution_id, staffs_ids, head_staff_id, phone_no, deleted_at, modified_at, created_at, domain from departments
        where deleted_at is null and institution_id = $1 and
        ($2::text is null or $2 ILIKE CONCAT('%', $2, '%') )
        order by name asc
",
        institution_id,
        query.clone()
    )
    .fetch_all(&self.customer_db)
    .await
    .wrap_err(format!(
        "Failed to execute find institution: {} query: {:?} ",
        institution_id, query
    ))
}

 async fn get_institution_department(
    &self,
    institution_id: i64,
    department_id: i64,
) -> Result<Option<Department>> {
    query_as!(
        Department,
        "select id, name, institution_id, staffs_ids, head_staff_id, phone_no, deleted_at, modified_at, created_at, domain from departments
        where deleted_at is null and
         institution_id = $1 and
         id = $2
        
",
        institution_id,
        department_id
    )
    .fetch_optional(&self.customer_db)
    .await
    .wrap_err(format!(
        "Failed to execute find institution: {} department_id: {} ",
        institution_id, department_id
    ))
}


 async fn find_department_staffs(
    &self,
    institution_id: i64,
    department_id: i64,
) -> Result<Vec<Staff>> {
    query_as!(
        Staff,
        "SELECT id, first_name, last_name, mobile, title, institution_id,
       profile_image, deleted_at, modified_at, created_at
FROM staffs
WHERE deleted_at IS NULL
  AND institution_id = $1
  AND id IN (
      SELECT (elem->>'staff_id')::uuid
      FROM departments d,
           jsonb_array_elements(d.staffs_ids) AS elem
      WHERE d.id = $2
      LIMIT 1
  )
ORDER BY first_name ASC
        ",
        institution_id,
        department_id
    )
    .fetch_all(&self.customer_db)
    .await
    .wrap_err(format!(
        "Failed to fetch institution: {institution_id} department: {department_id} staffs"
    ))
}


 async fn find_staff_institutions_by_mobile(
    &self,
    mobile: &str,
) -> Result<Vec<Institution>> {
    query_as!(
        Institution,
        "select i.id, i.name, i.email, i.classification, i.setting, i.address, i.town, i.state, i.created_by, i.workspace_code, i.logo, i.deleted_at, i.modified_at, i.created_at from staffs s join institutions i on i.id = s.institution_id
        where s.mobile = $1 and s.deleted_at is null
        ",
        mobile
    )
    .fetch_all(&self.customer_db)
    .await
    .wrap_err("Error fetching staff institution with mobile")
}

 async fn find_staff_accounts_by_mobile(&self, mobile: &str) -> Result<Vec<Staff>> {
    query_as!(
        Staff,
        "select id, first_name, last_name, mobile, title, institution_id,  profile_image, deleted_at, modified_at, created_at from staffs s
        where mobile = $1 and deleted_at is null
        ",
        mobile
    )
    .fetch_all(&self.customer_db)
    .await
    .wrap_err("Error fetching staff with mobile")
}

 async fn find_staff_by_mobile_and_insitution_id_opts(
    &self,
    institution_id: i64,
    mobile: &str,
) -> Result<Option<Staff>> {
    query_as!(
        Staff,
        "select s.id, s.first_name, s.last_name, s.mobile, s.title, s.institution_id,  s.profile_image, s.deleted_at, s.modified_at, s.created_at from staffs s
        where s.mobile = $1 and s.institution_id = $2 and s.deleted_at is null
        ",
        mobile,
        institution_id
    )
    .fetch_optional(&self.customer_db)
    .await
    .wrap_err("Error fetching staff with mobile")
}

 async fn find_institution_staff_by_id_opts(
    &self,
    institution_id: i64,
    staff_id: i64,
) -> Result<Option<Staff>> {
    query_as!(
        Staff,
        "select s.id, s.first_name, s.last_name, s.mobile, s.title, s.institution_id,  s.profile_image, s.deleted_at, s.modified_at, s.created_at from staffs s
        where s.id = $1 and s.institution_id = $2 and s.deleted_at is null
        ",
        institution_id,
        staff_id
    )
    .fetch_optional(&self.customer_db)
    .await
    .wrap_err("Error fetching instituion staff by id")
}





 async fn get_user(&self, mobile: &str) -> Result<Option<User>> {
    query_as!(
        User,
        "select id, mobile, password, failed_attempts, device_ids, last_login_time, deleted_at, modified_at, created_at  from users where mobile = $1 and deleted_at is null",
        mobile
    )
    .fetch_optional(&self.customer_db)
    .await
    .wrap_err("Error fetching user")
}


 async fn record_failed_attempts_user(&self, mobile: &str) -> Result<User> {
    query_as!(
        User,
        "update users  set failed_attempts = failed_attempts + 1 where mobile = $1 returning id, mobile, password, failed_attempts, device_ids, last_login_time, deleted_at, modified_at, created_at",
        mobile
    )
    .fetch_one(&self.customer_db)
    .await
    .wrap_err("Error fetching user")
}

 async fn record_user_login(&self, mobile: &str, device_id: &str) -> Result<User> {
    query_as!(
        User,
        "update users  set failed_attempts = failed_attempts + 1 where mobile = $1 returning id, mobile, password, failed_attempts, device_ids, last_login_time, deleted_at, modified_at, created_at",
        mobile
    )
    .fetch_one(&self.customer_db)
    .await
    .wrap_err("Error fetching user")
}

 async fn find_staff_departments(
    &self,
    institution_id: i64,
    staff_id: i64,
) -> Result<Vec<Department>> {
    query_as!(
        Department,
        "SELECT id, name, institution_id, staffs_ids, head_staff_id, phone_no,
       deleted_at, modified_at, created_at, domain
FROM departments
WHERE deleted_at IS NULL
  AND institution_id = $1
  AND EXISTS (
    SELECT 1
    FROM jsonb_array_elements(departments.staffs_ids) AS elem
    WHERE elem->>'staff_id' = $2
  )
",
        institution_id,
        staff_id
    )
    .fetch_all(&self.customer_db)
    .await
    .wrap_err(format!(
        "Failed to execute find staff departments institution: {} staff_id: {} ",
        institution_id, staff_id
    ))
}

 async fn edit_department(
    &self,
    department_id: i64,
    dept_name: String,
    head_staff_id: Option<i64>,
    phone_no: Option<String>,
    staffs_ids: &[DepartmentMember],
    domain: &str
) -> Result<Department> {
    query_as!(
        Department,
        "update departments set
            name = $2, head_staff_id = $3, phone_no=$4, staffs_ids = $5
        where id = $1
        returning id, name, institution_id, staffs_ids, head_staff_id, phone_no, deleted_at, modified_at, created_at, domain ",
        department_id,
        dept_name,
        head_staff_id,
        phone_no,
        &[json!(staffs_ids)]
    )
    .fetch_one(&self.customer_db)
    .await
    .wrap_err(format!(
        "Failed to edit department_id: {department_id}"
    ))
}


 async fn delete_department(&self, department_id: i64) -> Result<()> {
    query!(
        "update departments set
            deleted_at = Now()
        where id = $1",
        department_id
    )
    .execute(&self.customer_db)
    .await
    .wrap_err("Failed to delete department")?;

    Ok(())
}

 async fn create_user(
    &self,
    mobile: &str,
    password: &str,
) -> Result<User> {

    let user = query_as!(User, "insert into users (mobile, password) values ($1::varchar, $2::varchar) ON CONFLICT (mobile) DO UPDATE SET deleted_at = null, password = $2::varchar returning id, mobile, password, failed_attempts, device_ids, last_login_time, deleted_at, modified_at, created_at",
&mobile,
&password
).fetch_one(&self.customer_db)
.await
.wrap_err(format!("Failed to create user account"))?;


    return Ok(user);
}


}