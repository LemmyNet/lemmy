use crate::{
  newtypes::{CommunityId, LanguageId, LocalUserId, SiteId},
  source::{actor_language::*, language::Language},
};
use diesel::{
  delete, insert_into, result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
};

impl LocalUserLanguage {
  /// Update the user's languages.
  ///
  /// If no language_id vector is given, it will show all languages
  pub fn update_user_languages(
    conn: &mut PgConnection,
    language_ids: Vec<LanguageId>,
    for_local_user_id: LocalUserId,
  ) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      use crate::schema::local_user_language::dsl::*;
      // Clear the current user languages
      delete(local_user_language.filter(local_user_id.eq(for_local_user_id))).execute(conn)?;

      let lang_ids = update_languages(conn, language_ids)?;
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

impl SiteLanguage {
  pub fn update_site_languages(
    conn: &mut PgConnection,
    language_ids: Vec<LanguageId>,
    for_site_id: SiteId,
  ) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      use crate::schema::site_language::dsl::*;
      // Clear the current languages
      delete(site_language.filter(site_id.eq(for_site_id))).execute(conn)?;

      let lang_ids = update_languages(conn, language_ids)?;
      for l in lang_ids {
        let form = SiteLanguageForm {
          site_id: for_site_id,
          language_id: l,
        };
        insert_into(site_language)
          .values(form)
          .get_result::<Self>(conn)?;
      }
      Ok(())
    })
  }
}

impl CommunityLanguage {
  pub fn update_community_languages(
    conn: &mut PgConnection,
    language_ids: Vec<LanguageId>,
    for_community_id: CommunityId,
  ) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      use crate::schema::community_language::dsl::*;
      // Clear the current languages
      delete(community_language.filter(community_id.eq(for_community_id))).execute(conn)?;

      let lang_ids = update_languages(conn, language_ids)?;
      for l in lang_ids {
        let form = CommunityLanguageForm {
          community_id: for_community_id,
          language_id: l,
        };
        insert_into(community_language)
          .values(form)
          .get_result::<Self>(conn)?;
      }
      Ok(())
    })
  }
}

// If no language is given, set all languages
fn update_languages(
  conn: &mut PgConnection,
  language_ids: Vec<LanguageId>,
) -> Result<Vec<LanguageId>, Error> {
  if language_ids.is_empty() {
    Ok(
      Language::read_all(conn)?
        .into_iter()
        .map(|l| l.id)
        .collect(),
    )
  } else {
    Ok(language_ids)
  }
}
