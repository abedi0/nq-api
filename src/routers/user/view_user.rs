use actix_web::web;
use diesel::prelude::*;
use uuid::Uuid;

use super::FullUserProfile;
use crate::error::RouterError;
use crate::models::{Account, Email, User, UserName};
use crate::DbPool;

pub async fn view_user(
    path: web::Path<Uuid>,
    pool: web::Data<DbPool>,
) -> Result<web::Json<FullUserProfile>, RouterError> {
    use crate::schema::app_accounts::dsl::{app_accounts, uuid as uuid_from_accounts};
    use crate::schema::app_user_names::dsl::primary_name;

    let requested_account_uuid = path.into_inner();

    // select user form db
    // with user_id
    web::block(move || {
        let mut conn = pool.get().unwrap();

        let account: Account = app_accounts
            .filter(uuid_from_accounts.eq(requested_account_uuid))
            .get_result(&mut conn)?;

        let user: User = User::belonging_to(&account).get_result(&mut conn)?;

        let email = Email::belonging_to(&account).first::<Email>(&mut conn)?;

        // Now get the user names
        let names = UserName::belonging_to(&account)
            .filter(primary_name.eq(true))
            .load::<UserName>(&mut conn)?;

        // Is user have any names ?
        let names = if names.is_empty() { None } else { Some(names) };

        let profile = match names {
            Some(names) => {
                // Its must be always > 1 element
                let name: &UserName = names.first().unwrap();

                FullUserProfile {
                    uuid: account.uuid.to_string(),
                    email: email.email,
                    username: account.username.to_owned(),
                    first_name: Some(name.first_name.to_owned()),
                    last_name: Some(name.last_name.to_owned()),
                    birthday: user.clone().birthday,
                    profile_image: user.clone().profile_image,
                    language: user.clone().language,
                }
            }

            None => FullUserProfile {
                uuid: account.uuid.to_string(),
                email: email.email,
                username: account.username.to_owned(),
                first_name: None,
                last_name: None,
                birthday: user.clone().birthday,
                profile_image: user.clone().profile_image,
                language: user.clone().language,
            },
        };

        Ok(web::Json(profile))
    })
    .await
    .unwrap()
}
