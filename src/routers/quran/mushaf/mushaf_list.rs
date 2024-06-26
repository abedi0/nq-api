use crate::error::RouterError;
use crate::filter::Filter;
use crate::models::QuranMushaf;
use crate::DbPool;
use actix_web::web;
use diesel::prelude::*;

use super::MushafListQuery;

/// Get the lists of mushafs
pub async fn mushaf_list(
    pool: web::Data<DbPool>,
    web::Query(query): web::Query<MushafListQuery>,
) -> Result<web::Json<Vec<QuranMushaf>>, RouterError> {

    let pool = pool.into_inner();

    web::block(move || {
        let mut conn = pool.get().unwrap();

        // Get the list of mushafs from the database
        let quran_mushafs = match QuranMushaf::filter(Box::from(query)) {
            Ok(filtred) => filtred,
            Err(err) => return Err(err.log_to_db(pool)),
        }
        .load::<QuranMushaf>(&mut conn)?;

        Ok(web::Json(quran_mushafs))
    })
    .await
    .unwrap()
}
