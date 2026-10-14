use crate::source::images::{
  ImageDetails,
  ImageDetailsInsertForm,
  LocalImage,
  LocalImageForm,
  RemoteImage,
};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  QueryDsl,
  dsl::exists,
  insert_into,
  select,
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_db_schema_file::{
  PersonId,
  schema::{image_details, local_image, remote_image},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  dburl::DbUrl,
};
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
      .run_transaction(|conn| {
        async move {
          let local_insert = insert_into(local_image::table)
            .values(form)
            .get_result::<Self>(conn)
            .await
            .with_lemmy_type(LemmyErrorType::CouldntCreate);

          ImageDetails::create(&mut conn.into(), image_details_form).await?;

          local_insert
        }
        .scope_boxed()
      })
      .await
  }

  pub async fn validate_by_alias_and_user(
    pool: &mut DbPool<'_>,
    alias: &str,
    person_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;

    select(exists(
      local_image::table.filter(
        local_image::pictrs_alias
          .eq(alias)
          .and(local_image::person_id.eq(person_id)),
      ),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotFound.into())
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
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
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
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }
}
