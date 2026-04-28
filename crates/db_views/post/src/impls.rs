use crate::PostView;
use diesel::{
  self,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
  TextExpressionMethods,
  debug_query,
  dsl::not,
  pg::Pg,
  query_builder::AsQuery,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::{SortDirection, asc_if};
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, MultiCommunityId, PostId},
  source::{
    actor_language::LocalUserLanguage,
    community::CommunityActions,
    local_site::LocalSite,
    local_user::LocalUser,
    multi_community::MultiCommunityEntry,
    person::Person,
    post::{Post, PostActions, post_actions_keys as pa_key, post_keys as key},
    site::Site,
  },
  utils::{
    limit_fetch,
    queries::filters::{filter_blocked, filter_not_unlisted},
  },
};
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  enums::{CommunityFollowerState, CommunityVisibility, ListingType, PostSortType},
  joins::{
    creator_community_actions_join,
    creator_community_instance_actions_join,
    creator_home_instance_actions_join,
    creator_local_instance_actions_join,
    image_details_join,
    my_community_actions_join,
    my_instance_communities_actions_join,
    my_instance_persons_actions_join_1,
    my_local_user_admin_join,
    my_person_actions_join,
    my_post_actions_join,
  },
  schema::{community, community_actions, person, post, post_actions},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
  traits::Crud,
  utils::{CoalesceKey, Commented, fuzzy_search, now, seconds_to_pg_interval},
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::validation::clean_url,
};
use tracing::debug;
use url::Url;

impl PaginationCursorConversion for PostView {
  type PaginatedType = Post;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.post.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    Post::read(pool, PostId(cursor.id()?)).await
  }
}

/// This dummy struct is necessary to allow pagination using PostAction keys
struct PostViewDummy(PostActions);
impl PaginationCursorConversion for PostViewDummy {
  type PaginatedType = PostActions;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_multi([self.0.post_id.0, self.0.person_id.0])
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let [post_id, person_id] = cursor.multi()?;
    PostActions::read(pool, PostId(post_id), PersonId(person_id)).await
  }
}

impl PostView {
  // TODO while we can abstract the joins into a function, the selects are currently impossible to
  // do, because they rely on a few types that aren't yet publicly exported in diesel:
  // https://github.com/diesel-rs/diesel/issues/4462

  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(my_person_id);
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(my_person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(my_person_id);
    let my_instance_communities_actions_join: my_instance_communities_actions_join =
      my_instance_communities_actions_join(my_person_id);
    let my_instance_persons_actions_join_1: my_instance_persons_actions_join_1 =
      my_instance_persons_actions_join_1(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    post::table
      .inner_join(person::table)
      .inner_join(community::table)
      .left_join(image_details_join())
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_community_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(creator_community_actions_join())
      .left_join(my_community_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_instance_communities_actions_join)
      .left_join(my_instance_persons_actions_join_1)
      .left_join(my_local_user_admin_join)
  }

  #[diesel::dsl::auto_type(no_type_alias)]
  /// This uses the post_actions table as the base, for faster filtering for some queries
  fn post_action_joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let community_join = community::table.on(post::community_id.eq(community::id));
    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(my_person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(my_person_id);
    let my_instance_communities_actions_join: my_instance_communities_actions_join =
      my_instance_communities_actions_join(my_person_id);
    let my_instance_persons_actions_join_1: my_instance_persons_actions_join_1 =
      my_instance_persons_actions_join_1(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    post_actions::table
      .inner_join(post::table)
      .inner_join(person::table)
      .inner_join(community_join)
      .left_join(image_details_join())
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_community_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(creator_community_actions_join())
      .left_join(my_community_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_instance_communities_actions_join)
      .left_join(my_instance_persons_actions_join_1)
      .left_join(my_local_user_admin_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    my_local_user: Option<&'_ LocalUser>,
    local_instance_id: InstanceId,
    is_mod_or_admin: bool,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let my_person_id = my_local_user.person_id();

    let mut query = Self::joins(my_person_id, local_instance_id)
      .filter(post::id.eq(post_id))
      .select(Self::as_select())
      .into_boxed();

    // Hide deleted and removed for non-admins or mods
    if !is_mod_or_admin {
      query = query
        .filter(
          community::removed
            .eq(false)
            .or(post::creator_id.nullable().eq(my_person_id)),
        )
        .filter(
          post::removed
            .eq(false)
            .or(post::creator_id.nullable().eq(my_person_id)),
        )
        .filter(
          community::deleted
            .eq(false)
            .or(post::creator_id.nullable().eq(my_person_id)),
        )
        // Posts deleted by the creator are still visible if they have any comments. If there
        // are no comments only the creator can see it.
        .filter(
          post::deleted
            .eq(false)
            .or(post::creator_id.nullable().eq(my_person_id))
            .or(post::comments.gt(0)),
        )
        // private communities can only by browsed by accepted followers
        .filter(
          community::visibility
            .ne(CommunityVisibility::Private)
            .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
        );
    }

    query = my_local_user.visible_communities_only(query);

    Commented::new(query)
      .text("PostView::read")
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// List all the read posts for your person, ordered by the read date.
  pub async fn list_read(
    pool: &mut DbPool<'_>,
    my_person: &Person,
    page_cursor: Option<PaginationCursor>,
    limit: Option<i64>,
    no_limit: Option<bool>,
  ) -> LemmyResult<PagedResponse<PostView>> {
    let limit = limit_fetch(limit, no_limit)?;
    let query = PostView::post_action_joins(Some(my_person.id), my_person.instance_id)
      .filter(post_actions::person_id.eq(my_person.id))
      .filter(post_actions::read_at.is_not_null())
      .filter(filter_blocked())
      .limit(limit)
      .select(PostView::as_select())
      .into_boxed();

    // Sorting by the read date
    let paginated_query = PostViewDummy::paginate(query, &page_cursor, SortDirection::Desc, pool)
      .await?
      .then_order_by(pa_key::read_at)
      // Tie breaker
      .then_order_by(pa_key::post_id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, page_cursor)
  }

  /// List all the hidden posts for your person, ordered by the hide date.
  pub async fn list_hidden(
    pool: &mut DbPool<'_>,
    my_person: &Person,
    page_cursor: Option<PaginationCursor>,
    limit: Option<i64>,
    no_limit: Option<bool>,
  ) -> LemmyResult<PagedResponse<PostView>> {
    let limit = limit_fetch(limit, no_limit)?;
    let query = PostView::post_action_joins(Some(my_person.id), my_person.instance_id)
      .filter(post_actions::person_id.eq(my_person.id))
      .filter(post_actions::hidden_at.is_not_null())
      .filter(filter_blocked())
      .limit(limit)
      .select(PostView::as_select())
      .into_boxed();

    // Sorting by the hidden date
    let paginated_query = PostViewDummy::paginate(query, &page_cursor, SortDirection::Desc, pool)
      .await?
      .then_order_by(pa_key::hidden_at)
      // Tie breaker
      .then_order_by(pa_key::post_id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, page_cursor)
  }
}

#[derive(Clone, Default)]
pub struct PostQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<PostSortType>,
  pub time_range_seconds: Option<i32>,
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub multi_community_id: Option<MultiCommunityId>,
  pub local_user: Option<&'a LocalUser>,
  pub show_hidden: Option<bool>,
  pub show_read: Option<bool>,
  pub show_nsfw: Option<bool>,
  pub hide_media: Option<bool>,
  pub no_comments_only: Option<bool>,
  pub keyword_blocks: Option<Vec<String>>,
  pub search_term: Option<String>,
  pub search_title_only: Option<bool>,
  pub search_url_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  /// For backwards compat with API v3 (not available on API v4).
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PostQuery<'_> {
  /// A special function which pre-fetches a list of community_ids, that can be used for a sql
  /// `post::community_id.eq_any` query.
  ///
  /// Used for the following cases:
  /// - Subscribed
  /// - Multicommunities
  /// - Moderator view
  /// - Suggested
  ///
  ///  A return value of None means ignore, empty vec means filter out everything (IE empty
  /// subscribed, moderated, suggested)
  async fn prefetch_community_ids(
    &self,
    pool: &mut DbPool<'_>,
    local_site: &LocalSite,
  ) -> LemmyResult<Option<Vec<CommunityId>>> {
    // First, check the given community or multi community id, then if both are none, check the
    // listing types
    let community_ids = match (self.community_id, self.multi_community_id) {
      (Some(id), None) => Some(vec![id]),
      (None, Some(id)) => Some(MultiCommunityEntry::list_community_ids(pool, id).await?),
      (Some(_), Some(_)) => {
        return Err(LemmyErrorType::CannotCombineCommunityIdAndMultiCommunityId.into());
      }
      (None, None) => {
        // If no community or multi_community is given, then parse the listing_types
        match self.listing_type.unwrap_or_default() {
          ListingType::Local | ListingType::All => None,
          ListingType::Subscribed => {
            if let Some(my_person_id) = self.local_user.person_id() {
              Some(CommunityActions::list_subscribed_community_ids(pool, my_person_id).await?)
            } else {
              // If you have no subscriptions, then return an empty list
              Some(vec![])
            }
          }
          ListingType::ModeratorView => {
            if let Some(my_person_id) = self.local_user.person_id() {
              Some(CommunityActions::get_person_moderated_communities(pool, my_person_id).await?)
            } else {
              // If you don't moderate anything, then return an empty list
              Some(vec![])
            }
          }
          ListingType::Suggested => {
            if let Some(suggested_multi_id) = local_site.suggested_multi_community_id {
              Some(MultiCommunityEntry::list_community_ids(pool, suggested_multi_id).await?)
            } else {
              Some(vec![])
            }
          }
        }
      }
    };

    Ok(community_ids)
  }

  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    site: &Site,
    local_site: &LocalSite,
  ) -> LemmyResult<PagedResponse<PostView>> {
    // Pre-fetching some important items, to prevent costly joins.
    let community_ids = self.prefetch_community_ids(pool, local_site).await?;
    let language_ids = LocalUserLanguage::read_opt(pool, self.local_user.map(|l| l.id)).await?;

    let limit = limit_fetch(self.limit, None)?;
    let my_person_id = self.local_user.person_id();

    let mut query = PostView::joins(my_person_id, site.instance_id)
      .select(PostView::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(page) = self.page {
      query = query.offset(limit * (page - 1));
    }

    // hide posts from deleted communities
    query = query.filter(community::deleted.eq(false));

    // only creator can see deleted posts and unpublished scheduled posts
    if let Some(person_id) = my_person_id {
      query = query.filter(post::deleted.eq(false).or(post::creator_id.eq(person_id)));
      query = query.filter(
        post::scheduled_publish_time_at
          .is_null()
          .or(post::creator_id.eq(person_id)),
      );
    } else {
      query = query
        .filter(post::deleted.eq(false))
        .filter(post::scheduled_publish_time_at.is_null());
    }

    //  Filter by the given community ids, prefetched above
    if let Some(community_ids) = &community_ids {
      query = query.filter(post::community_id.eq_any(community_ids));
    }

    // Filter by the creator id
    if let Some(creator_id) = self.creator_id {
      query = query.filter(post::creator_id.eq(creator_id));
    }

    // Although the other listing types pre-fetched the communities, you still need to filter by
    // local if necessary.
    if self.listing_type.unwrap_or_default() == ListingType::Local {
      query = query.filter(community::local.eq(true));
    }

    // The search term
    if let Some(search_term) = self.search_term {
      let searcher = fuzzy_search(&search_term);

      let name_or_title_filter = post::name.ilike(searcher.clone());

      // A url / cross-post search
      query = if self.search_url_only.unwrap_or_default() {
        // Parse and normalize the url, removing tracking parameters (same logic which is used
        // when creating a new post).
        let normalized_url = Url::parse(&search_term).map(|u| clean_url(&u).to_string())?;

        query.filter(post::url.eq(normalized_url))
      } else if self.search_title_only.unwrap_or_default() {
        query.filter(name_or_title_filter)
      } else {
        let body_or_description_filter = post::body.ilike(searcher.clone());
        query.filter(name_or_title_filter.or(body_or_description_filter))
      }
    }

    // Hide the unlisted communities for the general types. Subscribed will still show them
    if [ListingType::Local, ListingType::All].contains(&self.listing_type.unwrap_or_default()) {
      query = query.filter(filter_not_unlisted());
    }

    if !self.show_nsfw.unwrap_or(self.local_user.show_nsfw(site)) {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    if !self.local_user.show_bot_accounts() {
      query = query.filter(person::bot_account.eq(false));
    };

    // Filter to show only posts with no comments
    if self.no_comments_only.unwrap_or_default() {
      query = query.filter(post::comments.eq(0));
    };

    if !self.show_read.unwrap_or(self.local_user.show_read_posts()) {
      query = query.filter(post_actions::read_at.is_null());
    }

    // Hide the hidden posts
    if !self.show_hidden.unwrap_or_default() {
      query = query.filter(post_actions::hidden_at.is_null());
    }

    if self.hide_media.unwrap_or(self.local_user.hide_media()) {
      query = query.filter(not(
        post::url_content_type.is_not_null().and(
          post::url_content_type
            .like("image/%")
            .or(post::url_content_type.like("video/%")),
        ),
      ));
    }

    query = self.local_user.visible_communities_only(query);
    query = query.filter(
      post::federation_pending
        .eq(false)
        .or(post::creator_id.nullable().eq(my_person_id)),
    );

    if !self.local_user.is_admin() {
      query = query
        .filter(
          community::visibility
            .ne(CommunityVisibility::Private)
            .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
        )
        // only show removed posts to admin
        .filter(community::removed.eq(false))
        .filter(community::local_removed.eq(false))
        .filter(post::removed.eq(false));
    }

    // Dont filter blocks or missing languages for moderator view type
    if self.listing_type.unwrap_or_default() != ListingType::ModeratorView {
      // Filter out the rows with missing languages if user is logged in
      if let Some(language_ids) = language_ids {
        query = query.filter(post::language_id.eq_any(language_ids));
      }

      query = query.filter(filter_blocked());

      if let Some(keyword_blocks) = self.keyword_blocks {
        for keyword in keyword_blocks {
          let pattern = format!("%{}%", keyword);
          query = query.filter(post::name.not_ilike(pattern.clone()));
          query = query.filter(post::url.is_null().or(post::url.not_ilike(pattern.clone())));
          query = query.filter(
            post::body
              .is_null()
              .or(post::body.not_ilike(pattern.clone())),
          );
        }
      }
    }

    // Filter by the time range
    if let Some(time_range_seconds) = self.time_range_seconds {
      query =
        query.filter(post::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)));
    }

    // Only sort by ascending for Old
    let sort = self.sort.unwrap_or(PostSortType::Hot);
    let sort_direction = asc_if(sort == PostSortType::Old);

    let mut pq = PostView::paginate(query, &self.page_cursor, sort_direction, pool).await?;

    // featured posts first
    // Don't do for new / old sorts
    if sort != PostSortType::New && sort != PostSortType::Old {
      pq = if community_ids.is_none() {
        pq.then_order_by(key::featured_local)
      } else {
        pq.then_order_by(key::featured_community)
      };
    }

    // then use the main sort
    pq = match sort {
      PostSortType::Active => pq.then_order_by(key::hot_rank_active),
      PostSortType::Hot => pq.then_order_by(key::hot_rank),
      PostSortType::Scaled => pq.then_order_by(key::scaled_rank),
      PostSortType::Controversial => pq.then_order_by(key::controversy_rank),
      PostSortType::New | PostSortType::Old => pq.then_order_by(key::published_at),
      PostSortType::NewComments => {
        pq.then_order_by(CoalesceKey(key::newest_comment_time_at, key::published_at))
      }
      PostSortType::MostComments => pq.then_order_by(key::comments),
      PostSortType::Top => pq.then_order_by(key::score),
    };

    // use publish as fallback. especially useful for hot rank which reaches zero after some days.
    // necessary because old posts can be fetched over federation and inserted with high post id
    pq = match sort {
      // A second time-based sort would not be very useful
      PostSortType::New | PostSortType::Old | PostSortType::NewComments => pq,
      _ => pq.then_order_by(key::published_at),
    };

    // finally use unique post id as tie breaker
    pq = pq.then_order_by(key::id);

    // Convert to as_query to be able to use in commented.
    let query = pq.as_query();

    debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));
    let conn = &mut get_conn(pool).await?;
    let res = Commented::new(query)
      .text("PostQuery::list")
      .load::<PostView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, self.page_cursor)
  }
}
