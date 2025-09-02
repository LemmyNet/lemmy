use crate::{
  newtypes::LocalUserId,
  source::oauth_account::{OAuthAccount, OAuthAccountInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{oauth_account, oauth_account::dsl::local_user_id};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl OAuthAccount {
  pub async fn create(pool: &mut DbPool<'_>, form: &OAuthAccountInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(oauth_account::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn delete_user_accounts(
    pool: &mut DbPool<'_>,
    for_local_user_id: LocalUserId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

    diesel::delete(oauth_account::table.filter(local_user_id.eq(for_local_user_id)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }
}
