use crate::{
  newtypes::{ImageUploadId, LocalUserId},
  schema::image_upload::dsl::{image_upload, local_user_id, pictrs_alias},
  source::image_upload::{ImageUpload, ImageUploadForm},
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl, Table};
use diesel_async::RunQueryDsl;

impl ImageUpload {
  pub async fn create(pool: &mut DbPool<'_>, form: &ImageUploadForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(image_upload)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn get_all_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: &LocalUserId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    image_upload
      .filter(local_user_id.eq(user_id))
      .select(image_upload::all_columns())
      .load::<ImageUpload>(conn)
      .await
  }

  pub async fn delete(
    pool: &mut DbPool<'_>,
    image_upload_id: ImageUploadId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(image_upload.find(image_upload_id))
      .execute(conn)
      .await
  }

  pub async fn delete_by_alias(pool: &mut DbPool<'_>, alias: &str) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(image_upload.filter(pictrs_alias.eq(alias)))
      .execute(conn)
      .await
  }
}
