use either::Either;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tryhcs_shared::{
    api_params::ErrorMessage,
    institution_params::{AuthorizedInstitutionUser, AuthorizedUser, InstitutionId},
};

use crate::{ADMIN_DOMAIN, FORBIDDEN_API_STATUS_CODE};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstitutionAdminUser(pub AuthorizedInstitutionUser);

impl InstitutionAdminUser {
    pub async fn new(user: AuthorizedInstitutionUser) -> Result<Self, ErrorMessage> {
        if !(InstitutionAdminUser::is_workspace_admin(&user).await) {
            return Err(ErrorMessage("Unauthorized".into()));
        }

        return Ok(InstitutionAdminUser(user));
    }

    async fn is_workspace_admin(user: &AuthorizedInstitutionUser) -> bool {
        user.departments
            .iter()
            .any(|d| ADMIN_DOMAIN.eq_ignore_ascii_case(&d.name))
    }
}

pub fn is_adminstrative_department(dept_name: &str) -> bool {
    ADMIN_DOMAIN.eq_ignore_ascii_case(&dept_name.to_lowercase())
}

pub trait TypeAuthenticated {
    type Authorized;

    fn is_workspace_admin(&self) -> bool;

    fn from_authorized_user(
        user: AuthorizedUser,
        identifier: &Either<InstitutionId, String>,
    ) -> eyre::Result<Either<Self::Authorized, (StatusCode, ErrorMessage)>>;
}

impl TypeAuthenticated for AuthorizedInstitutionUser {
    type Authorized = Self;

    fn is_workspace_admin(&self) -> bool {
        self.departments
            .iter()
            .any(|d| ADMIN_DOMAIN.eq_ignore_ascii_case(&d.name))
    }

    fn from_authorized_user(
        user: AuthorizedUser,
        identifier: &Either<InstitutionId, String>,
    ) -> eyre::Result<Either<Self, (StatusCode, ErrorMessage)>> {
        let account = user
            .accounts
            .iter()
            .find(|i| match identifier {
                Either::Left(InstitutionId(institution_id)) => *institution_id == i.institution.id,
                Either::Right(institution_code) => {
                    i.institution.workspace_code.eq(institution_code)
                }
            })
            .map(|v| v.clone());

        Ok(match account {
            None => Either::Right((
                FORBIDDEN_API_STATUS_CODE,
                ErrorMessage("Unknown Workspace/Institution".into()),
            )),
            Some(account) => Either::Left(account),
        })
    }
}
