use crate::{
  newtypes::{CommunityId, LanguageId, LocalUserId, SiteId},
  source::{actor_language::*, language::Language},
};
use diesel::{
  delete, insert_into, result::Error, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
};
use lemmy_utils::error::LemmyError;

impl LocalUserLanguage {
  pub fn read_user_langs(
    conn: &PgConnection,
    for_local_user_id: LocalUserId,
  ) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::local_user_language::dsl::*;

    local_user_language
      .filter(local_user_id.eq(for_local_user_id))
      .select(language_id)
      .get_results(conn)
  }

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
  pub fn read(conn: &PgConnection, for_site_id: SiteId) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::site_language::dsl::*;
    site_language
      .filter(site_id.eq(for_site_id))
      .select(language_id)
      .load(conn)
  }

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
  /// Returns true if the given language is one of configured languages for given community
  pub fn is_allowed_community_language(
    conn: &PgConnection,
    for_language_id: LanguageId,
    for_community_id: CommunityId,
  ) -> Result<(), LemmyError> {
    use crate::schema::community_language::dsl::*;
    let count = community_language
      .filter(language_id.eq(for_language_id))
      .filter(community_id.eq(for_community_id))
      .count()
      .get_result::<i64>(conn)?;

    if count == 1 {
      Ok(())
    } else {
      Err(LemmyError::from_message("language_not_allowed"))
    }
  }

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
