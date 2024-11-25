use crate::{
  diesel::JoinOnDsl,
  newtypes::{CommunityId, InstanceId, LanguageId, LocalUserId, SiteId},
  schema::{local_site, site, site_language},
  source::{
    actor_language::{
      CommunityLanguage,
      CommunityLanguageForm,
      LocalUserLanguage,
      LocalUserLanguageForm,
      SiteLanguage,
      SiteLanguageForm,
    },
    language::Language,
    site::Site,
  },
  utils::{get_conn, DbPool},
};
use diesel::{
  delete,
  dsl::{count, exists},
  insert_into,
  result::Error,
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use tokio::sync::OnceCell;

pub const UNDETERMINED_ID: LanguageId = LanguageId(0);

impl LocalUserLanguage {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_local_user_id: LocalUserId,
  ) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::local_user_language::dsl::{
      language_id,
      local_user_id,
      local_user_language,
    };
    let conn = &mut get_conn(pool).await?;

    let langs = local_user_language
      .filter(local_user_id.eq(for_local_user_id))
      .order(language_id)
      .select(language_id)
      .get_results(conn)
      .await?;
    convert_read_languages(conn, langs).await
  }

  /// Update the user's languages.
  ///
  /// If no language_id vector is given, it will show all languages
  pub async fn update(
    pool: &mut DbPool<'_>,
    language_ids: Vec<LanguageId>,
    for_local_user_id: LocalUserId,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    let mut lang_ids = convert_update_languages(conn, language_ids).await?;

    // No need to update if languages are unchanged
    let current = LocalUserLanguage::read(&mut conn.into(), for_local_user_id).await?;
    if current == lang_ids {
      return Ok(());
    }

    // TODO: Force enable undetermined language for all users. This is necessary because many posts
    //       don't have a language tag (e.g. those from other federated platforms), so Lemmy users
    //       won't see them if undetermined language is disabled.
    //       This hack can be removed once a majority of posts have language tags, or when it is
    //       clearer for new users that they need to enable undetermined language.
    //       See https://github.com/LemmyNet/lemmy-ui/issues/999
    if !lang_ids.contains(&UNDETERMINED_ID) {
      lang_ids.push(UNDETERMINED_ID);
    }

    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          use crate::schema::local_user_language::dsl::{
            language_id,
            local_user_id,
            local_user_language,
          };
          // Delete old languages, not including new languages
          let delete_old = delete(local_user_language)
            .filter(local_user_id.eq(for_local_user_id))
            .filter(language_id.ne_all(&lang_ids))
            .execute(conn);

          let forms = lang_ids
            .iter()
            .map(|&l| LocalUserLanguageForm {
              local_user_id: for_local_user_id,
              language_id: l,
            })
            .collect::<Vec<_>>();

          // Insert new languages
          let insert_new = insert_into(local_user_language)
            .values(forms)
            .on_conflict((language_id, local_user_id))
            .do_nothing()
            .execute(conn);

          tokio::try_join!(delete_old, insert_new)?;
          Ok(())
        }) as _
      })
      .await
  }
}

impl SiteLanguage {
  pub async fn read_local_raw(pool: &mut DbPool<'_>) -> Result<Vec<LanguageId>, Error> {
    let conn = &mut get_conn(pool).await?;
    site::table
      .inner_join(local_site::table)
      .inner_join(site_language::table)
      .order(site_language::language_id)
      .select(site_language::language_id)
      .load(conn)
      .await
  }

  pub async fn read(pool: &mut DbPool<'_>, for_site_id: SiteId) -> Result<Vec<LanguageId>, Error> {
    let conn = &mut get_conn(pool).await?;
    let langs = site_language::table
      .filter(site_language::site_id.eq(for_site_id))
      .order(site_language::language_id)
      .select(site_language::language_id)
      .load(conn)
      .await?;

    convert_read_languages(conn, langs).await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    language_ids: Vec<LanguageId>,
    site: &Site,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    let for_site_id = site.id;
    let instance_id = site.instance_id;
    let lang_ids = convert_update_languages(conn, language_ids).await?;

    // No need to update if languages are unchanged
    let current = SiteLanguage::read(&mut conn.into(), site.id).await?;
    if current == lang_ids {
      return Ok(());
    }

    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          use crate::schema::site_language::dsl::{language_id, site_id, site_language};

          // Delete old languages, not including new languages
          let delete_old = delete(site_language)
            .filter(site_id.eq(for_site_id))
            .filter(language_id.ne_all(&lang_ids))
            .execute(conn);

          let forms = lang_ids
            .iter()
            .map(|&l| SiteLanguageForm {
              site_id: for_site_id,
              language_id: l,
            })
            .collect::<Vec<_>>();

          // Insert new languages
          let insert_new = insert_into(site_language)
            .values(forms)
            .on_conflict((site_id, language_id))
            .do_nothing()
            .execute(conn);

          tokio::try_join!(delete_old, insert_new)?;

          CommunityLanguage::limit_languages(conn, instance_id).await?;

          Ok(())
        }) as _
      })
      .await
  }
}

impl CommunityLanguage {
  /// Returns true if the given language is one of configured languages for given community
  async fn is_allowed_community_language(
    pool: &mut DbPool<'_>,
    for_language_id: LanguageId,
    for_community_id: CommunityId,
  ) -> LemmyResult<()> {
    use crate::schema::community_language::dsl::community_language;
    let conn = &mut get_conn(pool).await?;

    let is_allowed = select(exists(
      community_language.find((for_community_id, for_language_id)),
    ))
    .get_result(conn)
    .await?;

    if is_allowed {
      Ok(())
    } else {
      Err(LemmyErrorType::LanguageNotAllowed)?
    }
  }

  /// When site languages are updated, delete all languages of local communities which are not
  /// also part of site languages. This is because post/comment language is only checked against
  /// community language, and it shouldnt be possible to post content in languages which are not
  /// allowed by local site.
  async fn limit_languages(
    conn: &mut AsyncPgConnection,
    for_instance_id: InstanceId,
  ) -> Result<(), Error> {
    use crate::schema::{
      community::dsl as c,
      community_language::dsl as cl,
      site_language::dsl as sl,
    };
    let community_languages: Vec<LanguageId> = cl::community_language
      .left_outer_join(sl::site_language.on(cl::language_id.eq(sl::language_id)))
      .inner_join(c::community)
      .filter(c::instance_id.eq(for_instance_id))
      .filter(sl::language_id.is_null())
      .select(cl::language_id)
      .get_results(conn)
      .await?;

    for c in community_languages {
      delete(cl::community_language.filter(cl::language_id.eq(c)))
        .execute(conn)
        .await?;
    }
    Ok(())
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
  ) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::community_language::dsl::{community_id, community_language, language_id};
    let conn = &mut get_conn(pool).await?;
    let langs = community_language
      .filter(community_id.eq(for_community_id))
      .order(language_id)
      .select(language_id)
      .get_results(conn)
      .await?;
    convert_read_languages(conn, langs).await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    mut language_ids: Vec<LanguageId>,
    for_community_id: CommunityId,
  ) -> Result<(), Error> {
    if language_ids.is_empty() {
      language_ids = SiteLanguage::read_local_raw(pool).await?;
    }
    let conn = &mut get_conn(pool).await?;
    let lang_ids = convert_update_languages(conn, language_ids).await?;

    // No need to update if languages are unchanged
    let current = CommunityLanguage::read(&mut conn.into(), for_community_id).await?;
    if current == lang_ids {
      return Ok(());
    }

    let form = lang_ids
      .iter()
      .map(|&language_id| CommunityLanguageForm {
        community_id: for_community_id,
        language_id,
      })
      .collect::<Vec<_>>();

    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          use crate::schema::community_language::dsl::{
            community_id,
            community_language,
            language_id,
          };
          // Delete old languages, not including new languages
          let delete_old = delete(community_language)
            .filter(community_id.eq(for_community_id))
            .filter(language_id.ne_all(&lang_ids))
            .execute(conn);

          // Insert new languages
          let insert_new = insert_into(community_language)
            .values(form)
            .on_conflict((community_id, language_id))
            .do_nothing()
            .execute(conn);

          tokio::try_join!(delete_old, insert_new)?;

          Ok(())
        }) as _
      })
      .await
  }
}

pub async fn validate_post_language(
  pool: &mut DbPool<'_>,
  language_id: Option<LanguageId>,
  community_id: CommunityId,
  local_user_id: LocalUserId,
) -> LemmyResult<LanguageId> {
  use crate::schema::{community_language::dsl as cl, local_user_language::dsl as ul};
  let conn = &mut get_conn(pool).await?;
  let language_id = match language_id {
    None | Some(LanguageId(0)) => {
      let mut intersection = ul::local_user_language
        .inner_join(cl::community_language.on(ul::language_id.eq(cl::language_id)))
        .filter(ul::local_user_id.eq(local_user_id))
        .filter(cl::community_id.eq(community_id))
        .select(cl::language_id)
        .get_results::<LanguageId>(conn)
        .await?;

      if intersection.len() == 1 {
        intersection.pop().unwrap_or(UNDETERMINED_ID)
      } else if intersection.len() == 2 && intersection.contains(&UNDETERMINED_ID) {
        intersection.retain(|i| i != &UNDETERMINED_ID);
        intersection.pop().unwrap_or(UNDETERMINED_ID)
      } else {
        UNDETERMINED_ID
      }
    }
    Some(lid) => lid,
  };

  CommunityLanguage::is_allowed_community_language(pool, language_id, community_id).await?;
  Ok(language_id)
}

/// If no language is given, set all languages
async fn convert_update_languages(
  conn: &mut AsyncPgConnection,
  language_ids: Vec<LanguageId>,
) -> Result<Vec<LanguageId>, Error> {
  if language_ids.is_empty() {
    Ok(
      Language::read_all(&mut conn.into())
        .await?
        .into_iter()
        .map(|l| l.id)
        .collect(),
    )
  } else {
    Ok(language_ids)
  }
}

/// If all languages are returned, return empty vec instead
#[allow(clippy::expect_used)]
async fn convert_read_languages(
  conn: &mut AsyncPgConnection,
  language_ids: Vec<LanguageId>,
) -> Result<Vec<LanguageId>, Error> {
  static ALL_LANGUAGES_COUNT: OnceCell<usize> = OnceCell::const_new();
  let count = ALL_LANGUAGES_COUNT
    .get_or_init(|| async {
      use crate::schema::language::dsl::{id, language};
      let count: i64 = language
        .select(count(id))
        .first(conn)
        .await
        .expect("read number of languages");
      count as usize
    })
    .await;

  if &language_ids.len() == count {
    Ok(vec![])
  } else {
    Ok(language_ids)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use crate::{
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_site::{LocalSite, LocalSiteInsertForm},
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      site::SiteInsertForm,
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use diesel::result::Error;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn test_langs1(pool: &mut DbPool<'_>) -> Result<Vec<LanguageId>, Error> {
    Ok(vec![
      Language::read_id_from_code(pool, "en").await?,
      Language::read_id_from_code(pool, "fr").await?,
      Language::read_id_from_code(pool, "ru").await?,
    ])
  }
  async fn test_langs2(pool: &mut DbPool<'_>) -> Result<Vec<LanguageId>, Error> {
    Ok(vec![
      Language::read_id_from_code(pool, "fi").await?,
      Language::read_id_from_code(pool, "se").await?,
    ])
  }

  async fn create_test_site(pool: &mut DbPool<'_>) -> Result<(Site, Instance), Error> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let site_form = SiteInsertForm::new("test site".to_string(), inserted_instance.id);
    let site = Site::create(pool, &site_form).await?;

    // Create a local site, since this is necessary for local languages
    let local_site_form = LocalSiteInsertForm::new(site.id);
    LocalSite::create(pool, &local_site_form).await?;

    Ok((site, inserted_instance))
  }

  #[tokio::test]
  #[serial]
  async fn test_convert_update_languages() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // call with empty vec, returns all languages
    let conn = &mut get_conn(pool).await?;
    let converted1 = convert_update_languages(conn, vec![]).await?;
    assert_eq!(184, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(&mut conn.into()).await?;
    let converted2 = convert_update_languages(conn, test_langs.clone()).await?;
    assert_eq!(test_langs, converted2);

    Ok(())
  }
  #[tokio::test]
  #[serial]
  async fn test_convert_read_languages() -> Result<(), Error> {
    use crate::schema::language::dsl::{id, language};
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // call with all languages, returns empty vec
    let conn = &mut get_conn(pool).await?;
    let all_langs = language.select(id).get_results(conn).await?;
    let converted1: Vec<LanguageId> = convert_read_languages(conn, all_langs).await?;
    assert_eq!(0, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(&mut conn.into()).await?;
    let converted2 = convert_read_languages(conn, test_langs.clone()).await?;
    assert_eq!(test_langs, converted2);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_site_languages() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (site, instance) = create_test_site(pool).await?;
    let site_languages1 = SiteLanguage::read_local_raw(pool).await?;
    // site is created with all languages
    assert_eq!(184, site_languages1.len());

    let test_langs = test_langs1(pool).await?;
    SiteLanguage::update(pool, test_langs.clone(), &site).await?;

    let site_languages2 = SiteLanguage::read_local_raw(pool).await?;
    // after update, site only has new languages
    assert_eq!(test_langs, site_languages2);

    Site::delete(pool, site.id).await?;
    Instance::delete(pool, instance.id).await?;
    LocalSite::delete(pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_user_languages() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let (site, instance) = create_test_site(pool).await?;

    let person_form = PersonInsertForm::test_form(instance.id, "my test person");
    let person = Person::create(pool, &person_form).await?;
    let local_user_form = LocalUserInsertForm::test_form(person.id);

    let local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;
    let local_user_langs1 = LocalUserLanguage::read(pool, local_user.id).await?;

    // new user should be initialized with all languages
    assert_eq!(0, local_user_langs1.len());

    // update user languages
    let test_langs2 = test_langs2(pool).await?;
    LocalUserLanguage::update(pool, test_langs2, local_user.id).await?;
    let local_user_langs2 = LocalUserLanguage::read(pool, local_user.id).await?;
    assert_eq!(3, local_user_langs2.len());

    Person::delete(pool, person.id).await?;
    LocalUser::delete(pool, local_user.id).await?;
    Site::delete(pool, site.id).await?;
    LocalSite::delete(pool).await?;
    Instance::delete(pool, instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_community_languages() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let (site, instance) = create_test_site(pool).await?;
    let test_langs = test_langs1(pool).await?;
    SiteLanguage::update(pool, test_langs.clone(), &site).await?;

    let read_site_langs = SiteLanguage::read(pool, site.id).await?;
    assert_eq!(test_langs, read_site_langs);

    // Test the local ones are the same
    let read_local_site_langs = SiteLanguage::read_local_raw(pool).await?;
    assert_eq!(test_langs, read_local_site_langs);

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community".to_string(),
      "test community".to_string(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;
    let community_langs1 = CommunityLanguage::read(pool, community.id).await?;

    // community is initialized with site languages
    assert_eq!(test_langs, community_langs1);

    let allowed_lang1 =
      CommunityLanguage::is_allowed_community_language(pool, test_langs[0], community.id).await;
    assert!(allowed_lang1.is_ok());

    let test_langs2 = test_langs2(pool).await?;
    let allowed_lang2 =
      CommunityLanguage::is_allowed_community_language(pool, test_langs2[0], community.id).await;
    assert!(allowed_lang2.is_err());

    // limit site languages to en, fi. after this, community languages should be updated to
    // intersection of old languages (en, fr, ru) and (en, fi), which is only fi.
    SiteLanguage::update(pool, vec![test_langs[0], test_langs2[0]], &site).await?;
    let community_langs2 = CommunityLanguage::read(pool, community.id).await?;
    assert_eq!(vec![test_langs[0]], community_langs2);

    // update community languages to different ones
    CommunityLanguage::update(pool, test_langs2.clone(), community.id).await?;
    let community_langs3 = CommunityLanguage::read(pool, community.id).await?;
    assert_eq!(test_langs2, community_langs3);

    Community::delete(pool, community.id).await?;
    Site::delete(pool, site.id).await?;
    LocalSite::delete(pool).await?;
    Instance::delete(pool, instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_validate_post_language() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let (site, instance) = create_test_site(pool).await?;
    let test_langs = test_langs1(pool).await?;
    let test_langs2 = test_langs2(pool).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "test community".to_string(),
      "test community".to_string(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;
    CommunityLanguage::update(pool, test_langs, community.id).await?;

    let person_form = PersonInsertForm::test_form(instance.id, "my test person");
    let person = Person::create(pool, &person_form).await?;
    let local_user_form = LocalUserInsertForm::test_form(person.id);
    let local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;
    LocalUserLanguage::update(pool, test_langs2, local_user.id).await?;

    // no overlap in user/community languages, so defaults to undetermined
    let def1 = validate_post_language(pool, None, community.id, local_user.id).await;
    assert_eq!(
      Some(LemmyErrorType::LanguageNotAllowed),
      def1.err().map(|e| e.error_type)
    );

    let ru = Language::read_id_from_code(pool, "ru").await?;
    let test_langs3 = vec![
      ru,
      Language::read_id_from_code(pool, "fi").await?,
      Language::read_id_from_code(pool, "se").await?,
      UNDETERMINED_ID,
    ];
    LocalUserLanguage::update(pool, test_langs3, local_user.id).await?;

    // this time, both have ru as common lang
    let def2 = validate_post_language(pool, None, community.id, local_user.id).await?;
    assert_eq!(ru, def2);

    Person::delete(pool, person.id).await?;
    Community::delete(pool, community.id).await?;
    LocalUser::delete(pool, local_user.id).await?;
    Site::delete(pool, site.id).await?;
    LocalSite::delete(pool).await?;
    Instance::delete(pool, instance.id).await?;

    Ok(())
  }
}
