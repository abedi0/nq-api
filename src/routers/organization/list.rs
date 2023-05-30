use crate::{
    error::RouterError,
    models::{Account, Organization, OrganizationName},
    DbPool,
};
use actix_web::web;
use chrono::NaiveDate;
use diesel::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct OrgWithName {
    pub username: String,
    pub primary_name: String,
    pub profile_image: Option<String>,
    pub established_date: NaiveDate,
    pub national_id: String,
}

pub async fn get_list_of_organizations(
    pool: web::Data<DbPool>,
) -> Result<web::Json<Vec<OrgWithName>>, RouterError> {
    use crate::schema::app_accounts::dsl::app_accounts;
    use crate::schema::app_organization_names::dsl::{
        app_organization_names, language as name_lang,
    };
    use crate::schema::app_organizations::dsl::app_organizations;

    let organizations: Result<Vec<OrgWithName>, RouterError> = web::block(move || {
        let mut conn = pool.get().unwrap();

        let Ok(select_all) = app_organizations
            .inner_join(app_accounts.inner_join(app_organization_names))
            .filter(name_lang.eq("default"))
            .select((Organization::as_select(), Account::as_select(), OrganizationName::as_select()))
            .load::<(Organization, Account, OrganizationName)>(&mut conn) else {
                return Err(RouterError::InternalError);
            };

        let result = select_all.iter().map(|(org, account, name)| OrgWithName {
            established_date: org.established_date,
            national_id: org.national_id.clone(),
            primary_name: name.name.clone(),
            profile_image: org.profile_image.clone(),
            username: account.username.clone()
        }).collect::<Vec<OrgWithName>>();

        Ok(result)
    })
    .await
    .unwrap();

    match organizations {
        Ok(orgs) => Ok(web::Json(orgs)),
        Err(err) => Err(err),
    }
}
