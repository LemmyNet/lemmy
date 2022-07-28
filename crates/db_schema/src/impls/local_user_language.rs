use crate::{
  newtypes::{LocalUserId, LocalUserLanguageId},
  source::local_user_language::*,
  traits::Crud,
};
use diesel::{result::Error, PgConnection, RunQueryDsl, *};

impl Crud for LocalUserLanguage {
  type Form = LocalUserLanguageForm;
  type IdType = LocalUserLanguageId;
  fn read(conn: &PgConnection, local_user_language_id: LocalUserLanguageId) -> Result<Self, Error> {
    use crate::schema::local_user_language::dsl::*;
    local_user_language
      .find(local_user_language_id)
      .first::<Self>(conn)
  }

  fn create(
    conn: &PgConnection,
    local_user_language_form: &LocalUserLanguageForm,
  ) -> Result<Self, Error> {
    use crate::schema::local_user_language::dsl::*;
    insert_into(local_user_language)
      .values(local_user_language_form)
      .on_conflict((local_user_id, language_id))
      .do_update()
      .set(local_user_language_form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    local_user_language_id: LocalUserLanguageId,
    local_user_language_form: &LocalUserLanguageForm,
  ) -> Result<Self, Error> {
    use crate::schema::local_user_language::dsl::*;
    diesel::update(local_user_language.find(local_user_language_id))
      .set(local_user_language_form)
      .get_result::<Self>(conn)
  }
}

impl LocalUserLanguage {
  pub fn clear_all_for_local_user(
    conn: &PgConnection,
    for_local_user_id: LocalUserId,
  ) -> Result<usize, Error> {
    use crate::schema::local_user_language::dsl::*;
    diesel::delete(local_user_language.filter(local_user_id.eq(for_local_user_id))).execute(conn)
  }
}
