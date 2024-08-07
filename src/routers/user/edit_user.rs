use actix_web::web;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    error::RouterError,
    models::{Account, Email, User, UserName},
    validate::validate,
    DbPool,
};

use super::EditableUser;

/// Edit the profile
/// wants a new profile and token
pub async fn edit_user(
    path: web::Path<Uuid>,
    pool: web::Data<DbPool>,
    new_user: web::Json<EditableUser>,
) -> Result<&'static str, RouterError> {
    use crate::schema::app_accounts::dsl::{app_accounts, username, uuid as uuid_from_account};
    use crate::schema::app_emails::dsl::email as app_email;
    use crate::schema::app_user_names::dsl::{first_name, last_name, primary_name};
    use crate::schema::app_users::dsl::*;

    let target_account_uuid = path.into_inner();
    let new_user = new_user.into_inner();

    validate(&new_user)?;

    web::block(move || {
        let mut conn = pool.get().unwrap();

        // First find the account from id
        let account: Account = app_accounts
            .filter(uuid_from_account.eq(target_account_uuid))
            .get_result(&mut conn)?;

        let user: User = User::belonging_to(&account).get_result(&mut conn)?;

        let email: Email = Email::belonging_to(&account).get_result(&mut conn)?;

        // Update Email
        diesel::update(&email)
            .set(app_email.eq(new_user.primary_email))
            .execute(&mut conn)?;

        // Now update the account username
        diesel::update(&account)
            .set(username.eq(new_user.username))
            .execute(&mut conn)?;

        // And update the other data
        diesel::update(&user)
            .set((
                birthday.eq(new_user.birthday),
                profile_image.eq(new_user.profile_image),
                language.eq(new_user.language),
            ))
            .execute(&mut conn)?;

        // Also edit the primary name

        // First We get the user_names of the account
        // We assume that user has at least primary name
        let name = UserName::belonging_to(&account)
            .filter(primary_name.eq(true))
            .first::<UserName>(&mut conn)?;

        // Now we update it
        diesel::update(&name)
            .set((
                first_name.eq(new_user.first_name),
                last_name.eq(new_user.last_name),
            ))
            .execute(&mut conn)?;

        Ok("Edited")
    })
    .await
    .unwrap()
}
