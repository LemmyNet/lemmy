use crate::structs::LocalUserDiscussionLanguageView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  schema::{language, local_user, local_user_language},
  source::{
    language::Language,
    local_user::{LocalUser, LocalUserSettings},
  },
  traits::ToSafeSettings,
  utils::{get_conn, DbPool},
};

type LocalUserDiscussionLanguageViewTuple = (LocalUserSettings, Language);

impl LocalUserDiscussionLanguageView {
  pub async fn read_languages(
    pool: &DbPool,
    local_user_id: LocalUserId,
  ) -> Result<Vec<Language>, Error> {
    let conn = &mut get_conn(pool).await?;

    let res = local_user_language::table
      .inner_join(local_user::table)
      .inner_join(language::table)
      .select((
        LocalUser::safe_settings_columns_tuple(),
        language::all_columns,
      ))
      .filter(local_user::id.eq(local_user_id))
      .load::<LocalUserDiscussionLanguageViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(|a| a.1).collect::<Vec<Language>>())
  }
}
