use crate::{
    error::RouterError,
    models::{NewPermission, NewPermissionCondition, Permission},
    DbPool,
};
use actix_web::web;
use diesel::prelude::*;

use super::NewPermissionData;

pub async fn add_permission(
    data: web::ReqData<u32>,
    new_permission: web::Json<NewPermissionData>,
    pool: web::Data<DbPool>,
) -> Result<&'static str, RouterError> {
    use crate::schema::app_accounts::dsl::{app_accounts, id as acc_id, uuid as acc_uuid};
    use crate::schema::app_permission_conditions::dsl::app_permission_conditions;
    use crate::schema::app_permissions::dsl::app_permissions;
    use crate::schema::app_users::dsl::{account_id as user_acc_id, app_users, id as user_id};

    let new_permission_data = new_permission.into_inner();
    let data = data.into_inner();

    web::block(move || {
        let mut conn = pool.get().unwrap();

        let account: i32 = app_accounts
            .filter(acc_uuid.eq(new_permission_data.subject))
            .select(acc_id)
            .get_result(&mut conn)?;

        let user: i32 = app_users
            .filter(user_acc_id.eq(data as i32))
            .select(user_id)
            .get_result(&mut conn)?;

        // First Insert a brand new Permission
        let new_permission: Permission = NewPermission {
            creator_user_id: user,
            account_id: account,
            object: &new_permission_data.object,
            action: &new_permission_data.action,
        }
        .insert_into(app_permissions)
        .get_result(&mut conn)?;

        // Now We must insert the Conditions
        // however We must make sure the request conditions
        // actually exists
        let mut insertable_conditions: Vec<NewPermissionCondition> = Vec::new();

        for condition in new_permission_data.conditions {
            condition.validate()?;

            insertable_conditions.push(NewPermissionCondition {
                creator_user_id: user,
                permission_id: new_permission.id,
                name: condition.name,
                value: condition.value,
            });
        }

        insertable_conditions
            .insert_into(app_permission_conditions)
            .execute(&mut conn)?;

        Ok("Added")
    })
    .await
    .unwrap()
}
