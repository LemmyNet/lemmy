use crate::structs::LocalUserDiscussionLanguageView;
use diesel::{result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  schema::{language, local_user, local_user_language},
  source::{
    language::Language,
    local_user::{LocalUser, LocalUserSettings},
  },
  traits::ToSafeSettings,
};

type LocalUserDiscussionLanguageViewTuple = (LocalUserSettings, Language);

impl LocalUserDiscussionLanguageView {
  pub fn read(conn: &PgConnection, local_user_id: LocalUserId) -> Result<Vec<Self>, Error> {
    let res = local_user_language::table
      .inner_join(local_user::table)
      .inner_join(language::table)
      .select((
        LocalUser::safe_settings_columns_tuple(),
        language::all_columns,
      ))
      .filter(local_user::id.eq(local_user_id))
      .load::<LocalUserDiscussionLanguageViewTuple>(conn)?;

    Ok(
      res
        .into_iter()
        .map(|a| Self {
          local_user: a.0,
          language: a.1,
        })
        .collect::<Vec<Self>>(),
    )
  }
}
