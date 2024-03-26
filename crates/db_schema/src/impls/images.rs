use crate::{
  newtypes::{DbUrl, LocalUserId},
  schema::{local_image, remote_image},
  source::images::{LocalImage, LocalImageForm, RemoteImage, RemoteImageForm},
  utils::{get_conn, limit_and_offset, DbPool},
};
use diesel::{
  dsl::exists,
  insert_into,
  result::Error,
  select,
  ExpressionMethods,
  NotFound,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use url::Url;

impl LocalImage {
  pub async fn create(pool: &mut DbPool<'_>, form: &LocalImageForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_image::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn get_all_paged_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    Self::get_all_helper(pool, Some(user_id), page, limit, false).await
  }

  pub async fn get_all_by_local_user_id(
    pool: &mut DbPool<'_>,
    user_id: LocalUserId,
  ) -> Result<Vec<Self>, Error> {
    Self::get_all_helper(pool, Some(user_id), None, None, true).await
  }

  pub async fn get_all(
    pool: &mut DbPool<'_>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    Self::get_all_helper(pool, None, page, limit, false).await
  }

  async fn get_all_helper(
    pool: &mut DbPool<'_>,
    user_id: Option<LocalUserId>,
    page: Option<i64>,
    limit: Option<i64>,
    ignore_page_limits: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = local_image::table
      .select(local_image::all_columns)
      .order_by(local_image::published.desc())
      .into_boxed();

    if let Some(user_id) = user_id {
      query = query.filter(local_image::local_user_id.eq(user_id))
    }

    if !ignore_page_limits {
      let (limit, offset) = limit_and_offset(page, limit)?;
      query = query.limit(limit).offset(offset);
    }

    query.load::<LocalImage>(conn).await
  }

  pub async fn delete_by_alias(pool: &mut DbPool<'_>, alias: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_image::table.filter(local_image::pictrs_alias.eq(alias)))
      .get_result(conn)
      .await
  }

  pub async fn delete_by_url(pool: &mut DbPool<'_>, url: &DbUrl) -> Result<Self, Error> {
    let alias = url.as_str().split('/').last().ok_or(NotFound)?;
    Self::delete_by_alias(pool, alias).await
  }
}

impl RemoteImage {
  pub async fn create(pool: &mut DbPool<'_>, links: Vec<Url>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    let forms = links
      .into_iter()
      .map(|url| RemoteImageForm { link: url.into() })
      .collect::<Vec<_>>();
    insert_into(remote_image::table)
      .values(forms)
      .on_conflict_do_nothing()
      .execute(conn)
      .await
  }

  pub async fn validate(pool: &mut DbPool<'_>, link_: DbUrl) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;

    let exists = select(exists(
      remote_image::table.filter(remote_image::link.eq(link_)),
    ))
    .get_result::<bool>(conn)
    .await?;
    if exists {
      Ok(())
    } else {
      Err(NotFound)
    }
  }
}
