use crate::{
  newtypes::{DbUrl, LocalUserId},
  source::images::{ImageDetails, ImageDetailsInsertForm, LocalImage, LocalImageForm, RemoteImage},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::exists,
  insert_into,
  result::Error,
  select,
  BoolExpressionMethods,
  ExpressionMethods,
  NotFound,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::{image_details, local_image, remote_image};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use url::Url;

impl LocalImage {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &LocalImageForm,
    image_details_form: &ImageDetailsInsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    conn
      .transaction::<_, Error, _>(|conn| {
        async move {
          let local_insert = insert_into(local_image::table)
            .values(form)
            .get_result::<Self>(conn)
            .await;

          ImageDetails::create(&mut conn.into(), image_details_form)
            .await
            .map_err(|_e| diesel::result::Error::NotFound)?;

          local_insert
        }
        .scope_boxed()
      })
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateImage)
  }

  pub async fn delete_by_alias(pool: &mut DbPool<'_>, alias: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_image::table.filter(local_image::pictrs_alias.eq(alias)))
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }

  /// Delete many aliases. Should be used with a pictrs purge.
  pub async fn delete_by_aliases(pool: &mut DbPool<'_>, aliases: &[String]) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_image::table.filter(local_image::pictrs_alias.eq_any(aliases)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }

  pub async fn delete_by_alias_and_user(
    pool: &mut DbPool<'_>,
    alias: &str,
    local_user_id: LocalUserId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      local_image::table.filter(
        local_image::pictrs_alias
          .eq(alias)
          .and(local_image::local_user_id.eq(local_user_id)),
      ),
    )
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::Deleted)
  }

  pub async fn delete_by_url(pool: &mut DbPool<'_>, url: &DbUrl) -> LemmyResult<Self> {
    let alias = url.as_str().split('/').next_back().ok_or(NotFound)?;
    Self::delete_by_alias(pool, alias).await
  }
}

impl RemoteImage {
  pub async fn create(pool: &mut DbPool<'_>, links: Vec<Url>) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    let forms = links
      .into_iter()
      .map(|url| remote_image::dsl::link.eq::<DbUrl>(url.into()))
      .collect::<Vec<_>>();
    insert_into(remote_image::table)
      .values(forms)
      .on_conflict_do_nothing()
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateImage)
  }

  pub async fn validate(pool: &mut DbPool<'_>, link_: DbUrl) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;

    select(exists(
      remote_image::table.filter(remote_image::link.eq(link_)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotFound.into())
  }
}

impl ImageDetails {
  pub async fn create(pool: &mut DbPool<'_>, form: &ImageDetailsInsertForm) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

    insert_into(image_details::table)
      .values(form)
      .on_conflict_do_nothing()
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateImage)
  }
}
