use crate::{newtypes::LocalUserId, source::local_user_language::*};
use diesel::{result::Error, PgConnection, RunQueryDsl, *};

impl LocalUserLanguage {
  pub fn update_user_languages(
    conn: &PgConnection,
    languages: Vec<LocalUserLanguageForm>,
    for_local_user_id: LocalUserId,
  ) -> Result<(), Error> {
    use crate::schema::local_user_language::dsl::*;
    conn.build_transaction().read_write().run(|| {
      delete(local_user_language.filter(local_user_id.eq(for_local_user_id))).execute(conn)?;

      for l in languages {
        insert_into(local_user_language)
          .values(l)
          .get_result::<Self>(conn)?;
      }
      Ok(())
    })
  }
}
