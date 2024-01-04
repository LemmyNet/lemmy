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
use lemmy_utils::error::{LemmyError, LemmyErrorType};
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

    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          let langs = local_user_language
            .filter(local_user_id.eq(for_local_user_id))
            .order(language_id)
            .select(language_id)
            .get_results(conn)
            .await?;
          convert_read_languages(conn, langs).await
        }) as _
      })
      .await
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
          use crate::schema::local_user_language::dsl::{local_user_id, local_user_language};
          // Clear the current user languages
          delete(local_user_language.filter(local_user_id.eq(for_local_user_id)))
            .execute(conn)
            .await?;

          for l in lang_ids {
            let form = LocalUserLanguageForm {
              local_user_id: for_local_user_id,
              language_id: l,
            };
            insert_into(local_user_language)
              .values(form)
              .get_result::<Self>(conn)
              .await?;
          }
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
          use crate::schema::site_language::dsl::{site_id, site_language};

          // Clear the current languages
          delete(site_language.filter(site_id.eq(for_site_id)))
            .execute(conn)
            .await?;

          for l in lang_ids {
            let form = SiteLanguageForm {
              site_id: for_site_id,
              language_id: l,
            };
            insert_into(site_language)
              .values(form)
              .get_result::<Self>(conn)
              .await?;
          }

          CommunityLanguage::limit_languages(conn, instance_id).await?;

          Ok(())
        }) as _
      })
      .await
  }
}

impl CommunityLanguage {
  /// Returns true if the given language is one of configured languages for given community
  pub async fn is_allowed_community_language(
    pool: &mut DbPool<'_>,
    for_language_id: Option<LanguageId>,
    for_community_id: CommunityId,
  ) -> Result<(), LemmyError> {
    use crate::schema::community_language::dsl::community_language;
    let conn = &mut get_conn(pool).await?;

    if let Some(for_language_id) = for_language_id {
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
    } else {
      Ok(())
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
      .into_iter()
      .map(|language_id| CommunityLanguageForm {
        community_id: for_community_id,
        language_id,
      })
      .collect::<Vec<_>>();

    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          use crate::schema::community_language::dsl::{community_id, community_language};
          use diesel::result::DatabaseErrorKind::UniqueViolation;
          // Clear the current languages
          delete(community_language.filter(community_id.eq(for_community_id)))
            .execute(conn)
            .await?;

          let insert_res = insert_into(community_language)
            .values(form)
            .get_result::<Self>(conn)
            .await;

          if let Err(Error::DatabaseError(UniqueViolation, _info)) = insert_res {
            // race condition: this function was probably called simultaneously from another caller. ignore error
            // tracing::warn!("unique error: {_info:#?}");
            // _info.constraint_name() should be = "community_language_community_id_language_id_key"
            return Ok(());
          } else {
            insert_res?;
          }
          Ok(())
        }) as _
      })
      .await
  }
}

pub async fn default_post_language(
  pool: &mut DbPool<'_>,
  community_id: CommunityId,
  local_user_id: LocalUserId,
) -> Result<Option<LanguageId>, Error> {
  use crate::schema::{community_language::dsl as cl, local_user_language::dsl as ul};
  let conn = &mut get_conn(pool).await?;
  let mut intersection = ul::local_user_language
    .inner_join(cl::community_language.on(ul::language_id.eq(cl::language_id)))
    .filter(ul::local_user_id.eq(local_user_id))
    .filter(cl::community_id.eq(community_id))
    .select(cl::language_id)
    .get_results::<LanguageId>(conn)
    .await?;

  if intersection.len() == 1 {
    Ok(intersection.pop())
  } else if intersection.len() == 2 && intersection.contains(&UNDETERMINED_ID) {
    intersection.retain(|i| i != &UNDETERMINED_ID);
    Ok(intersection.pop())
  } else {
    Ok(None)
  }
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
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::*;
  use crate::{
    impls::actor_language::{
      convert_read_languages,
      convert_update_languages,
      default_post_language,
      get_conn,
      CommunityLanguage,
      DbPool,
      Language,
      LanguageId,
      LocalUserLanguage,
      QueryDsl,
      RunQueryDsl,
      SiteLanguage,
    },
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_site::{LocalSite, LocalSiteInsertForm},
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      site::{Site, SiteInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn test_langs1(pool: &mut DbPool<'_>) -> Vec<LanguageId> {
    vec![
      Language::read_id_from_code(pool, Some("en"))
        .await
        .unwrap()
        .unwrap(),
      Language::read_id_from_code(pool, Some("fr"))
        .await
        .unwrap()
        .unwrap(),
      Language::read_id_from_code(pool, Some("ru"))
        .await
        .unwrap()
        .unwrap(),
    ]
  }
  async fn test_langs2(pool: &mut DbPool<'_>) -> Vec<LanguageId> {
    vec![
      Language::read_id_from_code(pool, Some("fi"))
        .await
        .unwrap()
        .unwrap(),
      Language::read_id_from_code(pool, Some("se"))
        .await
        .unwrap()
        .unwrap(),
    ]
  }

  async fn create_test_site(pool: &mut DbPool<'_>) -> (Site, Instance) {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let site_form = SiteInsertForm::builder()
      .name("test site".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let site = Site::create(pool, &site_form).await.unwrap();

    // Create a local site, since this is necessary for local languages
    let local_site_form = LocalSiteInsertForm::builder().site_id(site.id).build();
    LocalSite::create(pool, &local_site_form).await.unwrap();

    (site, inserted_instance)
  }

  #[tokio::test]
  #[serial]
  async fn test_convert_update_languages() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    // call with empty vec, returns all languages
    let conn = &mut get_conn(pool).await.unwrap();
    let converted1 = convert_update_languages(conn, vec![]).await.unwrap();
    assert_eq!(184, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(&mut conn.into()).await;
    let converted2 = convert_update_languages(conn, test_langs.clone())
      .await
      .unwrap();
    assert_eq!(test_langs, converted2);
  }
  #[tokio::test]
  #[serial]
  async fn test_convert_read_languages() {
    use crate::schema::language::dsl::{id, language};
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    // call with all languages, returns empty vec
    let conn = &mut get_conn(pool).await.unwrap();
    let all_langs = language.select(id).get_results(conn).await.unwrap();
    let converted1: Vec<LanguageId> = convert_read_languages(conn, all_langs).await.unwrap();
    assert_eq!(0, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(&mut conn.into()).await;
    let converted2 = convert_read_languages(conn, test_langs.clone())
      .await
      .unwrap();
    assert_eq!(test_langs, converted2);
  }

  #[tokio::test]
  #[serial]
  async fn test_site_languages() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let (site, instance) = create_test_site(pool).await;
    let site_languages1 = SiteLanguage::read_local_raw(pool).await.unwrap();
    // site is created with all languages
    assert_eq!(184, site_languages1.len());

    let test_langs = test_langs1(pool).await;
    SiteLanguage::update(pool, test_langs.clone(), &site)
      .await
      .unwrap();

    let site_languages2 = SiteLanguage::read_local_raw(pool).await.unwrap();
    // after update, site only has new languages
    assert_eq!(test_langs, site_languages2);

    Site::delete(pool, site.id).await.unwrap();
    Instance::delete(pool, instance.id).await.unwrap();
    LocalSite::delete(pool).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_user_languages() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let (site, instance) = create_test_site(pool).await;
    let mut test_langs = test_langs1(pool).await;
    SiteLanguage::update(pool, test_langs.clone(), &site)
      .await
      .unwrap();

    let person_form = PersonInsertForm::builder()
      .name("my test person".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let person = Person::create(pool, &person_form).await.unwrap();
    let local_user_form = LocalUserInsertForm::builder()
      .person_id(person.id)
      .password_encrypted("my_pw".to_string())
      .build();

    let local_user = LocalUser::create(pool, &local_user_form).await.unwrap();
    let local_user_langs1 = LocalUserLanguage::read(pool, local_user.id).await.unwrap();

    // new user should be initialized with site languages and undetermined
    //test_langs.push(UNDETERMINED_ID);
    //test_langs.sort();
    test_langs.insert(0, UNDETERMINED_ID);
    assert_eq!(test_langs, local_user_langs1);

    // update user languages
    let test_langs2 = test_langs2(pool).await;
    LocalUserLanguage::update(pool, test_langs2, local_user.id)
      .await
      .unwrap();
    let local_user_langs2 = LocalUserLanguage::read(pool, local_user.id).await.unwrap();
    assert_eq!(3, local_user_langs2.len());

    Person::delete(pool, person.id).await.unwrap();
    LocalUser::delete(pool, local_user.id).await.unwrap();
    Site::delete(pool, site.id).await.unwrap();
    LocalSite::delete(pool).await.unwrap();
    Instance::delete(pool, instance.id).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_community_languages() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let (site, instance) = create_test_site(pool).await;
    let test_langs = test_langs1(pool).await;
    SiteLanguage::update(pool, test_langs.clone(), &site)
      .await
      .unwrap();

    let read_site_langs = SiteLanguage::read(pool, site.id).await.unwrap();
    assert_eq!(test_langs, read_site_langs);

    // Test the local ones are the same
    let read_local_site_langs = SiteLanguage::read_local_raw(pool).await.unwrap();
    assert_eq!(test_langs, read_local_site_langs);

    let community_form = CommunityInsertForm::builder()
      .name("test community".to_string())
      .title("test community".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let community = Community::create(pool, &community_form).await.unwrap();
    let community_langs1 = CommunityLanguage::read(pool, community.id).await.unwrap();

    // community is initialized with site languages
    assert_eq!(test_langs, community_langs1);

    let allowed_lang1 =
      CommunityLanguage::is_allowed_community_language(pool, Some(test_langs[0]), community.id)
        .await;
    assert!(allowed_lang1.is_ok());

    let test_langs2 = test_langs2(pool).await;
    let allowed_lang2 =
      CommunityLanguage::is_allowed_community_language(pool, Some(test_langs2[0]), community.id)
        .await;
    assert!(allowed_lang2.is_err());

    // limit site languages to en, fi. after this, community languages should be updated to
    // intersection of old languages (en, fr, ru) and (en, fi), which is only fi.
    SiteLanguage::update(pool, vec![test_langs[0], test_langs2[0]], &site)
      .await
      .unwrap();
    let community_langs2 = CommunityLanguage::read(pool, community.id).await.unwrap();
    assert_eq!(vec![test_langs[0]], community_langs2);

    // update community languages to different ones
    CommunityLanguage::update(pool, test_langs2.clone(), community.id)
      .await
      .unwrap();
    let community_langs3 = CommunityLanguage::read(pool, community.id).await.unwrap();
    assert_eq!(test_langs2, community_langs3);

    Community::delete(pool, community.id).await.unwrap();
    Site::delete(pool, site.id).await.unwrap();
    LocalSite::delete(pool).await.unwrap();
    Instance::delete(pool, instance.id).await.unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn test_default_post_language() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let (site, instance) = create_test_site(pool).await;
    let test_langs = test_langs1(pool).await;
    let test_langs2 = test_langs2(pool).await;

    let community_form = CommunityInsertForm::builder()
      .name("test community".to_string())
      .title("test community".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let community = Community::create(pool, &community_form).await.unwrap();
    CommunityLanguage::update(pool, test_langs, community.id)
      .await
      .unwrap();

    let person_form = PersonInsertForm::builder()
      .name("my test person".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let person = Person::create(pool, &person_form).await.unwrap();
    let local_user_form = LocalUserInsertForm::builder()
      .person_id(person.id)
      .password_encrypted("my_pw".to_string())
      .build();
    let local_user = LocalUser::create(pool, &local_user_form).await.unwrap();
    LocalUserLanguage::update(pool, test_langs2, local_user.id)
      .await
      .unwrap();

    // no overlap in user/community languages, so defaults to undetermined
    let def1 = default_post_language(pool, community.id, local_user.id)
      .await
      .unwrap();
    assert_eq!(None, def1);

    let ru = Language::read_id_from_code(pool, Some("ru"))
      .await
      .unwrap()
      .unwrap();
    let test_langs3 = vec![
      ru,
      Language::read_id_from_code(pool, Some("fi"))
        .await
        .unwrap()
        .unwrap(),
      Language::read_id_from_code(pool, Some("se"))
        .await
        .unwrap()
        .unwrap(),
      UNDETERMINED_ID,
    ];
    LocalUserLanguage::update(pool, test_langs3, local_user.id)
      .await
      .unwrap();

    // this time, both have ru as common lang
    let def2 = default_post_language(pool, community.id, local_user.id)
      .await
      .unwrap();
    assert_eq!(Some(ru), def2);

    Person::delete(pool, person.id).await.unwrap();
    Community::delete(pool, community.id).await.unwrap();
    LocalUser::delete(pool, local_user.id).await.unwrap();
    Site::delete(pool, site.id).await.unwrap();
    LocalSite::delete(pool).await.unwrap();
    Instance::delete(pool, instance.id).await.unwrap();
  }
}
