use crate::{api::UserSettingsBackup, SiteView};
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::local_user::UserBackupLists,
  source::person::Person,
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
use std::sync::{Arc, LazyLock};

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

  pub async fn read_multicomm_follower(pool: &mut DbPool<'_>) -> LemmyResult<Person> {
    let site_view = SiteView::read_local(pool).await?;
    Person::read(pool, site_view.local_site.multi_comm_follower).await
  }
}

pub fn user_backup_list_to_user_settings_backup(
  local_user_view: LocalUserView,
  lists: UserBackupLists,
) -> UserSettingsBackup {
  let vec_into = |vec: Vec<_>| vec.into_iter().map(Into::into).collect();

  UserSettingsBackup {
    display_name: local_user_view.person.display_name,
    bio: local_user_view.person.bio,
    avatar: local_user_view.person.avatar.map(Into::into),
    banner: local_user_view.person.banner.map(Into::into),
    matrix_id: local_user_view.person.matrix_user_id,
    bot_account: local_user_view.person.bot_account.into(),
    settings: Some(local_user_view.local_user),
    followed_communities: vec_into(lists.followed_communities),
    blocked_communities: vec_into(lists.blocked_communities),
    blocked_instances: lists.blocked_instances,
    blocked_users: lists.blocked_users.into_iter().map(Into::into).collect(),
    saved_posts: lists.saved_posts.into_iter().map(Into::into).collect(),
    saved_comments: lists.saved_comments.into_iter().map(Into::into).collect(),
  }
}
