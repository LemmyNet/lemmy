use crate::{
  newtypes::{DbUrl, LocalUserId},
  schema::{
    local_image::dsl::{local_image, local_user_id, pictrs_alias},
    remote_image::dsl::{link, remote_image},
  },
  source::images::{LocalImage, LocalImageForm, RemoteImage, RemoteImageForm},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::exists,
  insert_into,
  result::Error,
  select,
  ExpressionMethods,
  NotFound,
  QueryDsl,
  Table,
};
use diesel_async::RunQueryDsl;
use url::Url;

impl LocalImage {
  pub async fn create(pool: &mut DbPool<'_>, form: &LocalImageForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_image)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn get_all_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: &LocalUserId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    local_image
      .filter(local_user_id.eq(user_id))
      .select(local_image::all_columns())
      .load::<LocalImage>(conn)
      .await
  }

  pub async fn delete_by_alias(pool: &mut DbPool<'_>, alias: &str) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_image.filter(pictrs_alias.eq(alias)))
      .execute(conn)
      .await
  }
}

impl RemoteImage {
  pub async fn create(pool: &mut DbPool<'_>, links: Vec<Url>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    let forms = links
      .into_iter()
      .map(|url| RemoteImageForm { link: url.into() })
      .collect::<Vec<_>>();
    insert_into(remote_image)
      .values(forms)
      .on_conflict_do_nothing()
      .execute(conn)
      .await
  }

  pub async fn validate(pool: &mut DbPool<'_>, link_: DbUrl) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;

    let exists = select(exists(remote_image.filter((link).eq(link_))))
      .get_result::<bool>(conn)
      .await?;
    if exists {
      Ok(())
    } else {
      Err(NotFound)
    }
  }
}
