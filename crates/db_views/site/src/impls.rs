use crate::{api::UserSettingsBackup, SiteView};
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    keyword_block::LocalUserKeywordBlock,
    language::Language,
    local_user::LocalUser,
    person::Person,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{instance, local_site, local_site_rate_limit, site};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  build_cache,
  error::{LemmyError, LemmyErrorType, LemmyResult},
  CacheLock,
};
use std::{
  collections::HashMap,
  sync::{Arc, LazyLock},
};

impl SiteView {
  pub async fn read_local(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    static CACHE: CacheLock<SiteView> = LazyLock::new(build_cache);
    CACHE
      .try_get_with((), async move {
        let conn = &mut get_conn(pool).await?;
        let local_site = site::table
          .inner_join(local_site::table)
          .inner_join(instance::table)
          .inner_join(
            local_site_rate_limit::table
              .on(local_site::id.eq(local_site_rate_limit::local_site_id)),
          )
          .select(Self::as_select())
          .first(conn)
          .await
          .optional()?
          .ok_or(LemmyErrorType::LocalSiteNotSetup)?;
        Ok(local_site)
      })
      .await
      .map_err(|e: Arc<LemmyError>| anyhow::anyhow!("err getting local site: {e:?}").into())
  }

  /// A special site bot user, solely made for following non-local communities for
  /// multi-communities.
  pub async fn read_multicomm_follower(pool: &mut DbPool<'_>) -> LemmyResult<Person> {
    let site_view = SiteView::read_local(pool).await?;
    Person::read(pool, site_view.local_site.multi_comm_follower).await
  }
}

pub async fn user_backup_list_to_user_settings_backup(
  local_user_view: LocalUserView,
  pool: &mut DbPool<'_>,
) -> LemmyResult<UserSettingsBackup> {
  let lists = LocalUser::export_backup(pool, local_user_view.person.id).await?;
  let blocking_keywords = LocalUserKeywordBlock::read(pool, local_user_view.local_user.id).await?;
  let discussion_languages = LocalUserLanguage::read(pool, local_user_view.local_user.id).await?;

  let all_languages: HashMap<_, _> = Language::read_all(pool)
    .await?
    .into_iter()
    .map(|l| (l.id, l.code))
    .collect();
  let discussion_languages = discussion_languages
    .iter()
    .flat_map(|d| all_languages.get(d).cloned())
    .collect();
  let vec_into = |vec: Vec<_>| vec.into_iter().map(Into::into).collect();
  Ok(UserSettingsBackup {
    display_name: local_user_view.person.display_name,
    bio: local_user_view.person.bio,
    avatar: local_user_view.person.avatar.map(Into::into),
    banner: local_user_view.person.banner.map(Into::into),
    matrix_id: local_user_view.person.matrix_user_id,
    bot_account: local_user_view.person.bot_account.into(),
    settings: Some(local_user_view.local_user),
    followed_communities: vec_into(lists.followed_communities),
    blocked_communities: vec_into(lists.blocked_communities),
    blocked_instances_communities: lists.blocked_instances_communities,
    blocked_instances_persons: lists.blocked_instances_persons,
    blocked_users: vec_into(lists.blocked_users),
    saved_posts: vec_into(lists.saved_posts),
    saved_comments: vec_into(lists.saved_comments),
    blocking_keywords,
    discussion_languages,
  })
}
