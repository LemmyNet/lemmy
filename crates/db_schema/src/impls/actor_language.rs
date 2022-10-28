use crate::{
  diesel::JoinOnDsl,
  newtypes::{CommunityId, InstanceId, LanguageId, LocalUserId, SiteId},
  source::{actor_language::*, language::Language, site::Site},
};
use diesel::{
  delete,
  dsl::*,
  insert_into,
  result::Error,
  select,
  ExpressionMethods,
  PgConnection,
  QueryDsl,
  RunQueryDsl,
};
use lemmy_utils::error::LemmyError;
use once_cell::sync::OnceCell;

impl LocalUserLanguage {
  pub fn read(
    conn: &mut PgConnection,
    for_local_user_id: LocalUserId,
  ) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::local_user_language::dsl::*;

    let langs = local_user_language
      .filter(local_user_id.eq(for_local_user_id))
      .select(language_id)
      .get_results(conn)?;
    convert_read_languages(conn, langs)
  }

  /// Update the user's languages.
  ///
  /// If no language_id vector is given, it will show all languages
  pub fn update(
    conn: &mut PgConnection,
    language_ids: Vec<LanguageId>,
    for_local_user_id: LocalUserId,
  ) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      use crate::schema::local_user_language::dsl::*;
      // Clear the current user languages
      delete(local_user_language.filter(local_user_id.eq(for_local_user_id))).execute(conn)?;

      let lang_ids = convert_update_languages(conn, language_ids)?;
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
  pub fn read_local(conn: &mut PgConnection) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::{local_site, site, site_language};
    site::table
      .inner_join(local_site::table)
      .inner_join(site_language::table)
      .select(site_language::language_id)
      .load(conn)
  }

  pub fn read(conn: &mut PgConnection, for_site_id: SiteId) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::site_language::dsl::*;
    let langs = site_language
      .filter(site_id.eq(for_site_id))
      .select(language_id)
      .load(conn)?;
    convert_read_languages(conn, langs)
  }

  pub fn update(
    conn: &mut PgConnection,
    language_ids: Vec<LanguageId>,
    site: &Site,
  ) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      use crate::schema::site_language::dsl::*;
      // Clear the current languages
      delete(site_language.filter(site_id.eq(site.id))).execute(conn)?;

      let lang_ids = convert_update_languages(conn, language_ids)?;
      for l in lang_ids {
        let form = SiteLanguageForm {
          site_id: site.id,
          language_id: l,
        };
        insert_into(site_language)
          .values(form)
          .get_result::<Self>(conn)?;
      }

      CommunityLanguage::limit_languages(conn, site.instance_id)?;

      Ok(())
    })
  }
}

impl CommunityLanguage {
  /// Returns true if the given language is one of configured languages for given community
  pub fn is_allowed_community_language(
    conn: &mut PgConnection,
    for_language_id: Option<LanguageId>,
    for_community_id: CommunityId,
  ) -> Result<(), LemmyError> {
    use crate::schema::community_language::dsl::*;
    if let Some(for_language_id) = for_language_id {
      let is_allowed = select(exists(
        community_language
          .filter(language_id.eq(for_language_id))
          .filter(community_id.eq(for_community_id)),
      ))
      .get_result(conn)?;

      if is_allowed {
        Ok(())
      } else {
        Err(LemmyError::from_message("language_not_allowed"))
      }
    } else {
      Ok(())
    }
  }

  /// When site languages are updated, delete all languages of local communities which are not
  /// also part of site languages. This is because post/comment language is only checked against
  /// community language, and it shouldnt be possible to post content in languages which are not
  /// allowed by local site.
  fn limit_languages(conn: &mut PgConnection, for_instance_id: InstanceId) -> Result<(), Error> {
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
      .get_results(conn)?;

    for c in community_languages {
      delete(cl::community_language.filter(cl::language_id.eq(c))).execute(conn)?;
    }
    Ok(())
  }

  pub fn read(
    conn: &mut PgConnection,
    for_community_id: CommunityId,
  ) -> Result<Vec<LanguageId>, Error> {
    use crate::schema::community_language::dsl::*;
    let langs = community_language
      .filter(community_id.eq(for_community_id))
      .select(language_id)
      .get_results(conn)?;
    convert_read_languages(conn, langs)
  }

  pub fn update(
    conn: &mut PgConnection,
    mut language_ids: Vec<LanguageId>,
    for_community_id: CommunityId,
  ) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      use crate::schema::community_language::dsl::*;
      // Clear the current languages
      delete(community_language.filter(community_id.eq(for_community_id))).execute(conn)?;

      if language_ids.is_empty() {
        language_ids = SiteLanguage::read_local(conn)?;
      }
      for l in language_ids {
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

pub fn default_post_language(
  conn: &mut PgConnection,
  community_id: CommunityId,
  local_user_id: LocalUserId,
) -> Result<Option<LanguageId>, Error> {
  use crate::schema::{community_language::dsl as cl, local_user_language::dsl as ul};
  let intersection = ul::local_user_language
    .inner_join(cl::community_language.on(ul::language_id.eq(cl::language_id)))
    .filter(ul::local_user_id.eq(local_user_id))
    .filter(cl::community_id.eq(community_id))
    .select(cl::language_id)
    .get_results::<LanguageId>(conn)?;

  if intersection.len() == 1 {
    Ok(Some(intersection[0]))
  } else {
    Ok(None)
  }
}

/// If no language is given, set all languages
fn convert_update_languages(
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

/// If all languages are returned, return empty vec instead
fn convert_read_languages(
  conn: &mut PgConnection,
  language_ids: Vec<LanguageId>,
) -> Result<Vec<LanguageId>, Error> {
  static ALL_LANGUAGES_COUNT: OnceCell<usize> = OnceCell::new();
  let count = ALL_LANGUAGES_COUNT.get_or_init(|| {
    use crate::schema::language::dsl::*;
    let count: i64 = language
      .select(count(id))
      .first(conn)
      .expect("read number of languages");
    count as usize
  });

  if &language_ids.len() == count {
    Ok(vec![])
  } else {
    Ok(language_ids)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    impls::actor_language::*,
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_site::{LocalSite, LocalSiteInsertForm},
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      site::{Site, SiteInsertForm},
    },
    traits::Crud,
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  fn test_langs1(conn: &mut PgConnection) -> Vec<LanguageId> {
    vec![
      Language::read_id_from_code(conn, "en").unwrap(),
      Language::read_id_from_code(conn, "fr").unwrap(),
      Language::read_id_from_code(conn, "ru").unwrap(),
    ]
  }
  fn test_langs2(conn: &mut PgConnection) -> Vec<LanguageId> {
    vec![
      Language::read_id_from_code(conn, "fi").unwrap(),
      Language::read_id_from_code(conn, "se").unwrap(),
    ]
  }

  fn create_test_site(conn: &mut PgConnection) -> (Site, Instance) {
    let inserted_instance = Instance::create(conn, "my_domain.tld").unwrap();

    let site_form = SiteInsertForm::builder()
      .name("test site".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let site = Site::create(conn, &site_form).unwrap();

    // Create a local site, since this is necessary for local languages
    let local_site_form = LocalSiteInsertForm::builder().site_id(site.id).build();
    LocalSite::create(conn, &local_site_form).unwrap();

    (site, inserted_instance)
  }

  #[test]
  #[serial]
  fn test_convert_update_languages() {
    let conn = &mut establish_unpooled_connection();

    // call with empty vec, returns all languages
    let converted1 = convert_update_languages(conn, vec![]).unwrap();
    assert_eq!(184, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(conn);
    let converted2 = convert_update_languages(conn, test_langs.clone()).unwrap();
    assert_eq!(test_langs, converted2);
  }
  #[test]
  #[serial]
  fn test_convert_read_languages() {
    let conn = &mut establish_unpooled_connection();

    // call with all languages, returns empty vec
    use crate::schema::language::dsl::*;
    let all_langs = language.select(id).get_results(conn).unwrap();
    let converted1: Vec<LanguageId> = convert_read_languages(conn, all_langs).unwrap();
    assert_eq!(0, converted1.len());

    // call with nonempty vec, returns same vec
    let test_langs = test_langs1(conn);
    let converted2 = convert_read_languages(conn, test_langs.clone()).unwrap();
    assert_eq!(test_langs, converted2);
  }

  #[test]
  #[serial]
  fn test_site_languages() {
    let conn = &mut establish_unpooled_connection();

    let (site, instance) = create_test_site(conn);
    let site_languages1 = SiteLanguage::read_local(conn).unwrap();
    // site is created with all languages
    assert_eq!(184, site_languages1.len());

    let test_langs = test_langs1(conn);
    SiteLanguage::update(conn, test_langs.clone(), &site).unwrap();

    let site_languages2 = SiteLanguage::read_local(conn).unwrap();
    // after update, site only has new languages
    assert_eq!(test_langs, site_languages2);

    Site::delete(conn, site.id).unwrap();
    Instance::delete(conn, instance.id).unwrap();
    LocalSite::delete(conn).unwrap();
  }

  #[test]
  #[serial]
  fn test_user_languages() {
    let conn = &mut establish_unpooled_connection();

    let (site, instance) = create_test_site(conn);
    let test_langs = test_langs1(conn);
    SiteLanguage::update(conn, test_langs.clone(), &site).unwrap();

    let person_form = PersonInsertForm::builder()
      .name("my test person".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let person = Person::create(conn, &person_form).unwrap();
    let local_user_form = LocalUserInsertForm::builder()
      .person_id(person.id)
      .password_encrypted("my_pw".to_string())
      .build();

    let local_user = LocalUser::create(conn, &local_user_form).unwrap();
    let local_user_langs1 = LocalUserLanguage::read(conn, local_user.id).unwrap();

    // new user should be initialized with site languages
    assert_eq!(test_langs, local_user_langs1);

    // update user languages
    let test_langs2 = test_langs2(conn);
    LocalUserLanguage::update(conn, test_langs2, local_user.id).unwrap();
    let local_user_langs2 = LocalUserLanguage::read(conn, local_user.id).unwrap();
    assert_eq!(2, local_user_langs2.len());

    Person::delete(conn, person.id).unwrap();
    LocalUser::delete(conn, local_user.id).unwrap();
    Site::delete(conn, site.id).unwrap();
    LocalSite::delete(conn).unwrap();
    Instance::delete(conn, instance.id).unwrap();
  }

  #[test]
  #[serial]
  fn test_community_languages() {
    let conn = &mut establish_unpooled_connection();
    let (site, instance) = create_test_site(conn);
    let test_langs = test_langs1(conn);
    SiteLanguage::update(conn, test_langs.clone(), &site).unwrap();

    let read_site_langs = SiteLanguage::read(conn, site.id).unwrap();
    assert_eq!(test_langs, read_site_langs);

    // Test the local ones are the same
    let read_local_site_langs = SiteLanguage::read_local(conn).unwrap();
    assert_eq!(test_langs, read_local_site_langs);

    let community_form = CommunityInsertForm::builder()
      .name("test community".to_string())
      .title("test community".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let community = Community::create(conn, &community_form).unwrap();
    let community_langs1 = CommunityLanguage::read(conn, community.id).unwrap();

    // community is initialized with site languages
    assert_eq!(test_langs, community_langs1);

    let allowed_lang1 =
      CommunityLanguage::is_allowed_community_language(conn, Some(test_langs[0]), community.id);
    assert!(allowed_lang1.is_ok());

    let test_langs2 = test_langs2(conn);
    let allowed_lang2 =
      CommunityLanguage::is_allowed_community_language(conn, Some(test_langs2[0]), community.id);
    assert!(allowed_lang2.is_err());

    // limit site languages to en, fi. after this, community languages should be updated to
    // intersection of old languages (en, fr, ru) and (en, fi), which is only fi.
    SiteLanguage::update(conn, vec![test_langs[0], test_langs2[0]], &site).unwrap();
    let community_langs2 = CommunityLanguage::read(conn, community.id).unwrap();
    assert_eq!(vec![test_langs[0]], community_langs2);

    // update community languages to different ones
    CommunityLanguage::update(conn, test_langs2.clone(), community.id).unwrap();
    let community_langs3 = CommunityLanguage::read(conn, community.id).unwrap();
    assert_eq!(test_langs2, community_langs3);

    Community::delete(conn, community.id).unwrap();
    Site::delete(conn, site.id).unwrap();
    LocalSite::delete(conn).unwrap();
    Instance::delete(conn, instance.id).unwrap();
  }

  #[test]
  #[serial]
  fn test_default_post_language() {
    let conn = &mut establish_unpooled_connection();
    let (site, instance) = create_test_site(conn);
    let test_langs = test_langs1(conn);
    let test_langs2 = test_langs2(conn);

    let community_form = CommunityInsertForm::builder()
      .name("test community".to_string())
      .title("test community".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let community = Community::create(conn, &community_form).unwrap();
    CommunityLanguage::update(conn, test_langs, community.id).unwrap();

    let person_form = PersonInsertForm::builder()
      .name("my test person".to_string())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();
    let person = Person::create(conn, &person_form).unwrap();
    let local_user_form = LocalUserInsertForm::builder()
      .person_id(person.id)
      .password_encrypted("my_pw".to_string())
      .build();
    let local_user = LocalUser::create(conn, &local_user_form).unwrap();
    LocalUserLanguage::update(conn, test_langs2, local_user.id).unwrap();

    // no overlap in user/community languages, so no default language for post
    let def1 = default_post_language(conn, community.id, local_user.id).unwrap();
    assert_eq!(None, def1);

    let ru = Language::read_id_from_code(conn, "ru").unwrap();
    let test_langs3 = vec![
      ru,
      Language::read_id_from_code(conn, "fi").unwrap(),
      Language::read_id_from_code(conn, "se").unwrap(),
    ];
    LocalUserLanguage::update(conn, test_langs3, local_user.id).unwrap();

    // this time, both have ru as common lang
    let def2 = default_post_language(conn, community.id, local_user.id).unwrap();
    assert_eq!(Some(ru), def2);

    Person::delete(conn, person.id).unwrap();
    Community::delete(conn, community.id).unwrap();
    LocalUser::delete(conn, local_user.id).unwrap();
    Site::delete(conn, site.id).unwrap();
    LocalSite::delete(conn).unwrap();
    Instance::delete(conn, instance.id).unwrap();
  }
}
