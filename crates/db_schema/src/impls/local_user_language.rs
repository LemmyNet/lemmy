use crate::{
  newtypes::{LanguageId, LocalUserId},
  source::{language::Language, local_user_language::*},
};
use diesel::{result::Error, PgConnection, RunQueryDsl, *};

impl LocalUserLanguage {
  /// Update the user's languages.
  ///
  /// If no language_id vector is given, it will show all languages
  pub fn update_user_languages(
    conn: &mut PgConnection,
    language_ids: Option<Vec<LanguageId>>,
    for_local_user_id: LocalUserId,
  ) -> Result<(), Error> {
    use crate::schema::local_user_language::dsl::*;

    // If no language is given, read all languages
    let lang_ids = language_ids.unwrap_or(
      Language::read_all(conn)?
        .into_iter()
        .map(|l| l.id)
        .collect(),
    );

    conn.build_transaction().read_write().run(|conn| {
      // Clear the current user languages
      delete(local_user_language.filter(local_user_id.eq(for_local_user_id))).execute(conn)?;

      for l in lang_ids {
        let form = LocalUserLanguageForm {
          local_user_id: for_local_user_id,
          language_id: l,
        };
        insert_into(local_user_language)
          .values(form)
          .get_result::<Self>(conn)?;
      }
      Ok(())
    })
  }
}
