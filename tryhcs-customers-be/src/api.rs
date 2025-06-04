use std::{collections::BTreeMap, fmt::format};

use bon::Builder;
use chrono::{format, DateTime, Utc};
use either::Either;
// use hyper::StatusCode;
use redis::AsyncCommands;
use reqwest::StatusCode;
use scrypt::{
    password_hash::{
        self, rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Scrypt,
};
use serde::{Deserialize, Serialize};
use tryhcs_commons_be::{
    api_response::{ApiResponse, ErrorMessage},
    auth::{is_adminstrative_department, InstitutionAdminUser},
    file_upload::{get_presigned_object_url, upload_base64_file},
    utils::{generate_otp, generate_password, mask_email, mask_phone},
    ADMIN_DOMAIN, BAD_REQUEST_API_STATUS_CODE, DUPLICATE_API_STATUS_CODE,
    FORBIDDEN_API_STATUS_CODE, NOT_FOUND_API_STATUS_CODE, SUCCESS_API_STATUS_CODE,
    UNAUTHORIZED_API_STATUS_CODE,
};
use tryhcs_notifications_be::{send_email, send_sms, EmailMessage, NotificationChannel};
use tryhcs_shared::{
    api_params::PaginatedQuery,
    institution_params::{
        AuthenticatedUser, AuthorizedInstitutionUser, AuthorizedUser, CreateDepartment,
        CreateInstitution, DepartmentAndStaffDto, DepartmentDto, InitiatedOtp, InstitutionDto,
        LoginReq, LoginResponse, NewStaff, StaffDto, StaffId, VerifyOTP,
    },
    APIFileUpload, APIFileUploadResponse,
};
use uuid::Uuid;

use eyre::{eyre, Context};
use futures::{future::join_all, join};
use tracing::info;

use serde_json::{json, Value};
use std::sync::Arc;

use crate::{app::App, db_models::Staff};

// InstitutionRegistration
pub async fn create_institution_init(
    app: &App,
    create_req: &CreateInstitution,
) -> eyre::Result<ApiResponse<InitiatedOtp>> {
    if let Some(_) = app
        .db_pool
        .find_institution_by_email(&create_req.email)
        .await?
    {
        return Ok((
            DUPLICATE_API_STATUS_CODE,
            // I am thinking of internalization, just not sure yet!
            Either::Right(ErrorMessage(
                "Email is already registered to another institution".into(),
            )),
        ));
    }

    let session_id = format!("SZX-CRI-{}", Uuid::new_v4());
    let initated_otp = send_otp(
        app,
        NotificationChannel::Email(create_req.email.clone()),
        &session_id,
    )
    .await?;

    let req_cache = format!("REQC-{}", &session_id);
    app.redis
        .set_key(
            &req_cache,
            &serde_json::to_string(&create_req)?,
            Some(initated_otp.duration),
        )
        .await?;
    return Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(initated_otp))));
}

pub async fn create_institution_complete(
    app: &App,
    verify_req: &VerifyOTP,
) -> eyre::Result<ApiResponse<InstitutionDto>> {
    if let Either::Right(err_message) = verify_otp(app, verify_req).await? {
        return Ok((BAD_REQUEST_API_STATUS_CODE, Either::Right(err_message)));
    }

    let req_cache = format!("REQC-{}", &verify_req.session_id);
    let mut cached_req = app
        .redis
        .get_key(&req_cache)
        .await?
        .map(|v| serde_json::from_str::<CreateInstitution>(&v).ok())
        .flatten();

    match cached_req {
        None => {
            return Ok((
                BAD_REQUEST_API_STATUS_CODE,
                Either::Right(ErrorMessage("OTP Expired".into())),
            ));
        }
        Some(mut cached_req) => {
            cached_req.password = hash_password(&cached_req.password)?;

            let (institution, staff) = app.db_pool.create_institution(cached_req).await?;
            let institution_dto: InstitutionDto = institution.into();
            return Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(institution_dto))));
        }
    }
}

pub async fn login_init(
    app: &App,
    login_req: &LoginReq,
) -> eyre::Result<ApiResponse<LoginResponse>> {
    let staff_institutions = app
        .db_pool
        .find_staff_institutions_by_mobile(&login_req.phone_number)
        .await?;
    if staff_institutions.is_empty() {
        return Ok((
            NOT_FOUND_API_STATUS_CODE,
            Either::Right(ErrorMessage("Invalid credentials".into())),
        ));
    }

    let mut login_response = LoginResponse::default();

    let user = app.db_pool.get_user(&login_req.phone_number).await?;
    match user {
        None => {
            return Ok((
                NOT_FOUND_API_STATUS_CODE,
                Either::Right(ErrorMessage("Invalid credentials".into())),
            ));
        }
        Some(user) => {
            if user.failed_attempts >= app.env.max_failed_attempts {
                return Ok((
                    UNAUTHORIZED_API_STATUS_CODE,
                    Either::Right(ErrorMessage(
                        "Account disabled, max attempts reached".into(),
                    )),
                ));
            }
            if verify_password(&login_req.password, &user.password).is_err() {
                let remaining_attempts = app.env.max_failed_attempts - user.failed_attempts;

                let mut error_message = "Invalid credentials".into();
                if remaining_attempts < 3 {
                    error_message = format!("{error_message}, {remaining_attempts} remaining!")
                }

                if remaining_attempts < 1 {
                    error_message = "Account disabled".into();
                }

                return Ok((
                    NOT_FOUND_API_STATUS_CODE,
                    Either::Right(ErrorMessage(error_message)),
                ));
            }

            if !user.device_ids.is_empty() {
                // check that the device id exist here, else send OTP;

                if !user.device_ids.contains(&login_req.device_id) {
                    let session_id = format!("SZX-LGN-{}", Uuid::new_v4());
                    let initated_otp = send_otp(
                        app,
                        NotificationChannel::Mobile(login_req.phone_number.clone()),
                        &session_id,
                    )
                    .await?;

                    let req_cache = format!("REQC-{}", &session_id);
                    app.redis
                        .set_key(
                            &req_cache,
                            &serde_json::to_string(&login_req)?,
                            Some(initated_otp.duration),
                        )
                        .await?;

                    login_response.otp = Some(initated_otp);
                    return Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(login_response))));
                }
            }

            let auth = setup_auth_profile_and_token(app, login_req).await?;
            login_response.auth = Some(auth);
            return Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(login_response))));
        }
    }
}

pub async fn login_complete(
    app: &App,
    verify_req: &VerifyOTP,
) -> eyre::Result<ApiResponse<AuthenticatedUser>> {
    if let Either::Right(err_message) = verify_otp(app, verify_req).await? {
        return Ok((BAD_REQUEST_API_STATUS_CODE, Either::Right(err_message)));
    }

    let req_cache = format!("REQC-{}", &verify_req.session_id);
    let cached_req = app
        .redis
        .get_key(&req_cache)
        .await?
        .map(|v| serde_json::from_str::<LoginReq>(&v).ok())
        .flatten();

    match cached_req {
        None => {
            return Ok((
                BAD_REQUEST_API_STATUS_CODE,
                Either::Right(ErrorMessage("OTP Expired".into())),
            ));
        }
        Some(cached_req) => {
            let authenticated_user = setup_auth_profile_and_token(app, &cached_req).await?;
            return Ok((
                SUCCESS_API_STATUS_CODE,
                Either::Left(Some(authenticated_user)),
            ));
        }
    }
}

// impl Into<StaffDto> for AuthorizedInstitutionUser {
//     fn into(self) -> StaffDto {
//         StaffDto {
//             id: self.staff_id,
//             first_name: self.first_name,
//             last_name: self.last_name,
//             mobile: self.mobile,
//             title: self.title,
//             institution_id: Some(self.institution.id),
//             profile_image: self.profile_image_url,
//         }
//     }
// }

async fn setup_auth_profile_and_token(
    app: &App,
    cached_req: &LoginReq,
) -> eyre::Result<AuthenticatedUser> {
    let staff_institutions = app
        .db_pool
        .find_staff_institutions_by_mobile(&cached_req.phone_number)
        .await?
        .into_iter()
        .map(|i| (i.id, i))
        .collect::<BTreeMap<_, _>>();
    //staff_id
    let staff_accounts = app
        .db_pool
        .find_staff_accounts_by_mobile(&cached_req.phone_number)
        .await?;
    let mut accounts = vec![];
    for s in staff_accounts {
        if let Some(institution) = s
            .institution_id
            .map(|id| staff_institutions.get(&id).cloned())
            .flatten()
        {
            let departments = app
                .db_pool
                .find_staff_departments(institution.id, s.id)
                .await?
                .into_iter()
                .map(|v| v.into())
                .collect();

            let user = AuthorizedInstitutionUser {
                staff_id: s.shadow_id,
                first_name: s.first_name,
                last_name: s.last_name,
                mobile: s.mobile,
                title: s.title,
                profile_image_url: s.profile_image,
                departments,
                institution: institution.into(),
            };
            accounts.push(user);
        }
    }

    let authorized_user = AuthorizedUser {
        mobile: cached_req.phone_number.clone(),
        accounts: accounts,
    };

    let authenticated_user = AuthenticatedUser {
        principal: authorized_user,
        token: Some(Uuid::new_v4().to_string()),
    };

    return Ok(authenticated_user);
}

pub async fn get_staff_profile(
    app: &App,
    auth: &AuthorizedInstitutionUser,
    staff_id: &str,
) -> eyre::Result<ApiResponse<StaffDto>> {
    match app
        .db_pool
        .find_institution_staff_by_id_opts(auth.institution.px, staff_id)
        .await?
    {
        None => {
            return Ok((
                NOT_FOUND_API_STATUS_CODE,
                Either::Right(ErrorMessage("Staff not found".into())),
            ));
        }
        Some(staff) => {
            return Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(staff.into()))));
        }
    }
}

pub async fn find_staffs(
    app: &App,
    auth: &AuthorizedInstitutionUser,
    pagination: &PaginatedQuery,
) -> eyre::Result<ApiResponse<Vec<StaffDto>>> {
    let staffs = app
        .db_pool
        .find_institution_staffs(auth.institution.px, pagination.query.to_owned())
        .await?
        .into_iter()
        .map(|s| s.into())
        .collect();
    Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(staffs))))
}

pub async fn find_departments(
    app: &App,
    auth: &AuthorizedInstitutionUser,
    search: &PaginatedQuery,
) -> eyre::Result<ApiResponse<Vec<DepartmentDto>>> {
    let departments = app
        .db_pool
        .find_institution_departments(auth.institution.px, search.query.to_owned())
        .await?
        .into_iter()
        .map(|s| s.into())
        .collect();
    Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(departments))))
}

pub async fn send_otp<K: AsRef<str>>(
    app: &App,
    channel: NotificationChannel,
    key: K,
) -> eyre::Result<InitiatedOtp> {
    let req_cache: String = format!("OTP-{}", key.as_ref());
    let otp = generate_otp(app.env.otp_length as u32)?;
    let otp_expires_in_sec = app.env.otp_expires_in_sec as u64;
    let mut notification_message = "Please enter the OTP sent to".into();

    dbg!((&req_cache, &otp, &otp_expires_in_sec));

    app.redis
        .set_key(&req_cache, &otp, Some(otp_expires_in_sec))
        .await?;

    let result = match channel {
        NotificationChannel::Email(email) => {
            let content = include_str!("../assets/templates/email_otp.html")
                .replace("{{otp_code}}", &otp)
                .replace("{{duration}}", &format!("{}", otp_expires_in_sec));

            let message = EmailMessage {
                to: email.to_owned(),
                subject: "OTP Verification".to_owned(),
                content,
            };

            notification_message = format!("{} {}", notification_message, mask_email(&email));

            send_email(&app.env, message).await
        }
        NotificationChannel::Mobile(mobile) => {
            let message = format!("Your OTP code is {}", otp);
            notification_message = format!("{} {}", notification_message, mask_phone(&mobile));

            send_sms(&app.env, &mobile, &&message).await
        }
    };

    if let Err(err) = result {
        tracing::error!(message="Failed to send otp", err=?err);
    }

    Ok(InitiatedOtp {
        session_id: key.as_ref().to_owned(),
        duration: otp_expires_in_sec,
        message: notification_message,
    })
}

pub async fn verify_otp(app: &App, verify: &VerifyOTP) -> eyre::Result<Either<(), ErrorMessage>> {
    let req_cache: String = format!("OTP-{}", &verify.session_id);

    dbg!((&req_cache));

    match app.redis.get_key(&req_cache).await?.map(|cached_value| {
        dbg!((&req_cache, &cached_value));

        cached_value.eq_ignore_ascii_case(&verify.otp_code)
    }) {
        Some(true) => {
            return Ok(Either::Left(()));
        }
        _ => return Ok(Either::Right(ErrorMessage("Invalid or Expired OTP".into()))),
    }
}

fn hash_password(value: &str) -> eyre::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Scrypt.hash_password(value.as_bytes(), &salt)?.to_string())
}

fn verify_password(value: &str, hash: &str) -> eyre::Result<()> {
    let hash = PasswordHash::new(hash)?;
    Scrypt.verify_password(value.as_bytes(), &hash)?;
    Ok(())
}

pub async fn get_department_and_staffs(
    app: &App,
    auth: &AuthorizedInstitutionUser,
    department_id: &str,
) -> eyre::Result<ApiResponse<DepartmentAndStaffDto>> {
    let department = app
        .db_pool
        .get_institution_department(auth.institution.px, department_id)
        .await?;

    match department {
        None => {
            return Ok((
                NOT_FOUND_API_STATUS_CODE,
                Either::Right(ErrorMessage("Department not found".into())),
            ));
        }
        Some(department) => {
            let staffs = app
                .db_pool
                .find_department_staffs(auth.institution.px, department_id)
                .await?
                .into_iter()
                .map(|s| s.into())
                .collect::<Vec<StaffDto>>();

            let department_head = staffs
                .iter()
                .find(|s| {
                    department
                        .head_staff_id
                        .as_ref()
                        .map(|id| id.eq_ignore_ascii_case(&s.id))
                        .unwrap_or(false)
                })
                .cloned();

            let department_and_staffs = DepartmentAndStaffDto {
                staffs,
                department: department.into(),
                department_head,
            };

            Ok((
                SUCCESS_API_STATUS_CODE,
                Either::Left(Some(department_and_staffs)),
            ))
        }
    }
}

pub async fn create_department(
    app: &App,
    InstitutionAdminUser(auth): &InstitutionAdminUser,
    create_department: CreateDepartment,
) -> eyre::Result<ApiResponse<DepartmentDto>> {
    let insitution_departments = app
        .db_pool
        .find_institution_departments(auth.institution.px, None)
        .await?;
    let insitution_departments = insitution_departments
        .iter()
        .find(|v| v.deleted_at.is_none() && !(is_adminstrative_department(&v.name)));

    if insitution_departments.is_none() {
        return Ok((
            BAD_REQUEST_API_STATUS_CODE,
            Either::Right(ErrorMessage("Department already exists".into())),
        ));
    }

    let department = app
        .db_pool
        .create_department(
            auth.institution.px,
            create_department.name,
            create_department.head_staff_id,
            create_department.phone_no,
            &create_department.staff_ids,
            &create_department.domain,
        )
        .await?;

    Ok((
        SUCCESS_API_STATUS_CODE,
        Either::Left(Some(department.into())),
    ))
}

pub async fn edit_department(
    app: &App,
    InstitutionAdminUser(auth): &InstitutionAdminUser,
    department_id: &str,
    create_department: CreateDepartment,
) -> eyre::Result<ApiResponse<DepartmentDto>> {
    let insitution_departments = app
        .db_pool
        .find_institution_departments(auth.institution.px, None)
        .await?;
    let insitution_departments = insitution_departments.iter().find(|v| {
        v.deleted_at.is_none() && !(is_adminstrative_department(&v.name)) && v.shadow_id.eq_ignore_ascii_case(department_id)
    });

    if insitution_departments.is_none() {
        return Ok((
            NOT_FOUND_API_STATUS_CODE,
            Either::Right(ErrorMessage("Department not found".into())),
        ));
    }

    let department = app
        .db_pool
        .edit_department(
            department_id,
            create_department.name,
            create_department.head_staff_id,
            create_department.phone_no,
            &create_department.staff_ids,
            &create_department.domain,
        )
        .await?;

    Ok((
        SUCCESS_API_STATUS_CODE,
        Either::Left(Some(department.into())),
    ))
}

pub async fn delete_department(
    app: &App,
    InstitutionAdminUser(auth): &InstitutionAdminUser,
    department_id: &str,
) -> eyre::Result<ApiResponse<()>> {
    let insitution_departments = app
        .db_pool
        .find_institution_departments(auth.institution.px, None)
        .await?;
    let insitution_departments = insitution_departments.iter().find(|v| {
        v.deleted_at.is_none() && !(is_adminstrative_department(&v.name)) && v.shadow_id.eq_ignore_ascii_case(department_id)
    });

    if insitution_departments.is_none() {
        return Ok((
            NOT_FOUND_API_STATUS_CODE,
            Either::Right(ErrorMessage("Department not found".into())),
        ));
    }

    app.db_pool.delete_department(department_id).await?;

    Ok((SUCCESS_API_STATUS_CODE, Either::Left(None)))
}

pub async fn add_staff(
    app: &App,
    InstitutionAdminUser(auth): &InstitutionAdminUser,
    new_staff: NewStaff,
) -> eyre::Result<ApiResponse<StaffDto>> {
    let institution_id = auth.institution.px;
    let existing_staff = app
        .db_pool
        .find_staff_by_mobile_and_insitution_id_opts(institution_id, &new_staff.mobile)
        .await?;
    match existing_staff {
        Some(mut staff) => {
            if staff.deleted_at.is_none() {
                return Ok((
                    BAD_REQUEST_API_STATUS_CODE,
                    Either::Right(ErrorMessage("Staff already exists".into())),
                ));
            }

            staff.first_name = new_staff.first_name;
            staff.last_name = new_staff.last_name;
            staff.title = new_staff.title;
            staff.profile_image = new_staff.profile_image;
            staff.deleted_at = None;

            staff = app.db_pool.update_staff(staff).await?;
            Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(staff.into()))))
        }
        None => {
            let password = generate_password(app.env.min_password_length as usize);
            if let None = app.db_pool.get_user(&new_staff.mobile).await? {
                let password_hashed = hash_password(&password)?;
                app.db_pool
                    .create_user(&new_staff.mobile, &password_hashed)
                    .await?;
            }

            let staff = app.db_pool.create_staff(institution_id, &new_staff).await?;
            let message = format!(
                "You have been invited to {} by {}",
                app.env.app_url, auth.institution.institution_name
            );
            send_sms(&app.env, &new_staff.mobile, &message).await;

            let message = format!("Your login password is {}", password);
            send_sms(&app.env, &new_staff.mobile, &message).await;
            Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(staff.into()))))
        }
    }
}

pub async fn edit_staff(
    app: &App,
    InstitutionAdminUser(auth): &InstitutionAdminUser,
    staff_id: &str,
    new_staff: NewStaff,
) -> eyre::Result<ApiResponse<StaffDto>> {
    let institution_id = auth.institution.px;
    let existing_staff: Option<Staff> = app
        .db_pool
        .find_institution_staff_by_id_opts(institution_id, staff_id)
        .await?;
    match existing_staff {
        None => {
            return Ok((
                NOT_FOUND_API_STATUS_CODE,
                Either::Right(ErrorMessage("Staff not found".into())),
            ));
        }
        Some(mut staff) => {
            staff.first_name = new_staff.first_name;
            staff.last_name = new_staff.last_name;
            staff.title = new_staff.title;
            staff.profile_image = new_staff.profile_image;

            staff = app.db_pool.update_staff(staff).await?;
            Ok((SUCCESS_API_STATUS_CODE, Either::Left(Some(staff.into()))))
        }
    }
}

pub async fn delete_staff(
    app: &App,
    InstitutionAdminUser(auth): &InstitutionAdminUser,
    staff_id: &str,
) -> eyre::Result<ApiResponse<()>> {
    let institution_id = auth.institution.px;

    let existing_staff: Option<Staff> = app
        .db_pool
        .find_institution_staff_by_id_opts(institution_id, staff_id)
        .await?;
    match existing_staff {
        None => {
            return Ok((
                NOT_FOUND_API_STATUS_CODE,
                Either::Right(ErrorMessage("Staff not found".into())),
            ));
        }
        Some(staff) => {
            app.db_pool.delete_staff(staff_id).await?;
            Ok((SUCCESS_API_STATUS_CODE, Either::Left(None)))
        }
    }
}

pub async fn upload_base64_file_api(
    app: &App,
    user: &AuthorizedInstitutionUser,
    upload_req: &APIFileUpload,
) -> eyre::Result<ApiResponse<APIFileUploadResponse>> {
    let mut file_name = "".to_string();
    if let Some(name) = &upload_req.file_name {
        file_name = name.to_owned();
    }

    let path = format!(
        "{}/I{}S{}-T{}--{}",
        &upload_req.service,
        user.institution.id,
        user.staff_id,
        chrono::Utc::now().timestamp_millis(),
        file_name
    );
    upload_base64_file(
        &app.s3_client,
        &app.env.cloudflare_r2_bucket,
        &path,
        &upload_req.base64_data,
        upload_req.content_type.to_owned(),
    )
    .await?;
    let mut response_data = APIFileUploadResponse {
        file_key: path,
        file_non_perment_link: None,
    };

    if let Some(duration) = upload_req.link_expires_duration {
        response_data.file_non_perment_link = Some(
            get_presigned_object_url(
                &app.s3_client,
                &app.env.cloudflare_r2_bucket,
                &response_data.file_key,
                duration,
            )
            .await?,
        );
    }
    return Ok((StatusCode::CREATED, Either::Left(Some(response_data))));
}
