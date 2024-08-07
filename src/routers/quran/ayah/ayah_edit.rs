use crate::error::RouterError;
use crate::DbPool;
use actix_web::web;
use diesel::prelude::*;
use uuid::Uuid;

use super::SimpleAyah;

/// Update's single ayah
pub async fn ayah_edit(
    path: web::Path<Uuid>,
    new_ayah: web::Json<SimpleAyah>,
    pool: web::Data<DbPool>,
) -> Result<&'static str, RouterError> {
    use crate::schema::quran_ayahs::dsl::{
        ayah_number, quran_ayahs, sajdeh as ayah_sajdeh, uuid as ayah_uuid,
    };

    let new_ayah = new_ayah.into_inner();
    let target_ayah_uuid = path.into_inner();

    web::block(move || {
        let mut conn = pool.get().unwrap();

        let new_sajdeh = new_ayah.sajdeh.map(|sajdeh| sajdeh.to_string());

        diesel::update(quran_ayahs.filter(ayah_uuid.eq(target_ayah_uuid)))
            .set((
                ayah_number.eq(new_ayah.ayah_number),
                ayah_sajdeh.eq(new_sajdeh),
            ))
            .execute(&mut conn)?;

        Ok("Edited")
    })
    .await
    .unwrap()
}
