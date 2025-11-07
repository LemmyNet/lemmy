use crate::{
  FederatedInstanceView,
  ReadableFederationState,
  SiteView,
  api::{GetFederatedInstances, GetFederatedInstancesKind, UserSettingsBackup},
};
use diesel::{
  ExpressionMethods,
  JoinOnDsl,
  OptionalExtension,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  newtypes::{InstanceId, PaginationCursor},
  source::{
    actor_language::LocalUserLanguage,
    federation_queue_state::FederationQueueState,
    instance::{Instance, instance_keys as key},
    keyword_block::LocalUserKeywordBlock,
    language::Language,
    local_user::LocalUser,
    person::Person,
  },
  traits::PaginationCursorBuilder,
  utils::limit_fetch,
};
use lemmy_db_schema_file::schema::{
  federation_allowlist,
  federation_blocklist,
  federation_queue_state,
  instance,
  local_site,
  local_site_rate_limit,
  site,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  traits::Crud,
  utils::{fuzzy_search, paginate},
};
use lemmy_utils::{
  CacheLock,
  build_cache,
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  federate_retry_sleep_duration,
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

impl FederatedInstanceView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    instance::table
      // omit instance representing the local site
      .left_join(site::table.left_join(local_site::table))
      .filter(local_site::id.is_null())
      .left_join(federation_blocklist::table)
      .left_join(federation_allowlist::table)
      .left_join(federation_queue_state::table)
  }

  pub async fn list(pool: &mut DbPool<'_>, data: GetFederatedInstances) -> LemmyResult<Vec<Self>> {
    let cursor_data = if let Some(cursor) = &data.page_cursor {
      Some(FederatedInstanceView::from_cursor(cursor, pool).await?)
    } else {
      None
    };
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(data.limit)?;
    let mut query = Self::joins()
      .select(Self::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(domain_filter) = &data.domain_filter {
      query = query.filter(instance::domain.ilike(fuzzy_search(domain_filter)))
    }

    query = match data.kind {
      GetFederatedInstancesKind::All => query,
      GetFederatedInstancesKind::Linked => {
        query.filter(federation_blocklist::instance_id.is_null())
      }
      GetFederatedInstancesKind::Allowed => {
        query.filter(federation_allowlist::instance_id.is_not_null())
      }
      GetFederatedInstancesKind::Blocked => {
        query.filter(federation_blocklist::instance_id.is_not_null())
      }
    };

    let mut pq = paginate(
      query,
      SortDirection::Desc,
      cursor_data,
      None,
      data.page_back,
    );

    // Show recently updated instances and those with valid metadata first
    pq = pq
      .then_order_by(key::updated_at)
      .then_order_by(key::software)
      .then_order_by(key::id);

    pq.get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read(pool: &mut DbPool<'_>, instance_id: InstanceId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(instance::id.eq(instance_id))
      .select(Self::as_select())
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorBuilder for FederatedInstanceView {
  type CursorData = Instance;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('I', self.instance.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let [(_, id)] = cursor.prefixes_and_ids()?;
    Instance::read(pool, InstanceId(id)).await
  }
}

#[allow(clippy::expect_used)]
impl From<FederationQueueState> for ReadableFederationState {
  fn from(internal_state: FederationQueueState) -> Self {
    ReadableFederationState {
      next_retry_at: internal_state.last_retry_at.map(|r| {
        r + chrono::Duration::from_std(federate_retry_sleep_duration(internal_state.fail_count))
          .expect("sleep duration longer than 2**63 ms (262 million years)")
      }),
      internal_state,
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use crate::{
    FederatedInstanceView,
    api::{GetFederatedInstances, GetFederatedInstancesKind},
  };
  use lemmy_db_schema::{
    assert_length,
    source::{
      federation_allowlist::{FederationAllowList, FederationAllowListForm},
      federation_queue_state::FederationQueueState,
      instance::Instance,
      site::{Site, SiteInsertForm},
    },
  };
  use lemmy_diesel_utils::{connection::build_db_pool_for_tests, traits::Crud};
  use lemmy_utils::error::LemmyResult;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_instance_list() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    // insert test data
    let instance0 = Instance::read_or_create(pool, "example0.com").await?;
    let instance1 = Instance::read_or_create(pool, "example1.com").await?;
    let site_form = SiteInsertForm::new("Example".to_string(), instance0.id);
    let site = Site::create(pool, &site_form).await?;
    let form = FederationAllowListForm::new(instance0.id);
    let allow = FederationAllowList::allow(pool, &form).await?;
    let queue_state = FederationQueueState {
      instance_id: instance0.id,
      fail_count: 5,
      last_successful_id: None,
      last_successful_published_time_at: None,
      last_retry_at: None,
    };
    FederationQueueState::upsert(pool, &queue_state).await?;

    // run the query
    let data = GetFederatedInstances {
      domain_filter: None,
      kind: GetFederatedInstancesKind::Linked,
      page_cursor: None,
      page_back: None,
      limit: None,
    };
    let list = FederatedInstanceView::list(pool, data).await?;
    assert_length!(2, list);

    // compare first result
    let list0 = &list[1];
    assert_eq!(instance0.domain, list0.instance.domain);
    assert_eq!(Some(site), list0.site.clone());
    assert_eq!(
      Some(queue_state.fail_count),
      list0.queue_state.clone().map(|q| q.fail_count)
    );
    assert_eq!(Some(allow), list0.allowed);
    assert!(list0.blocked.is_none());

    // compare second result
    let list1 = &list[0];
    assert_eq!(instance1.domain, list1.instance.domain);
    assert!(list1.site.is_none());
    assert!(list1.queue_state.is_none());
    assert!(list1.allowed.is_none());
    assert!(list1.blocked.is_none());

    Instance::delete_all(pool).await?;
    Ok(())
  }
}
