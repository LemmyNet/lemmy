use crate::PostView;
use diesel::{
  self,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
  TextExpressionMethods,
  debug_query,
  dsl::{exists, not},
  pg::Pg,
  query_builder::AsQuery,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::{SortDirection, asc_if};
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, MultiCommunityId, PostId},
  source::{
    community::CommunityActions,
    local_user::LocalUser,
    person::Person,
    post::{Post, PostActions, post_actions_keys as pa_key, post_keys as key},
    site::Site,
  },
  utils::{
    limit_fetch,
    queries::filters::{
      filter_blocked,
      filter_is_subscribed,
      filter_not_unlisted_or_is_subscribed,
      filter_suggested_communities,
    },
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
  schema::{
    community,
    community_actions,
    local_user_language,
    multi_community_entry,
    person,
    post,
    post_actions,
  },
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
  utils::{CoalesceKey, Commented, now, seconds_to_pg_interval},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use tracing::debug;

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
    let query = PostView::joins(Some(my_person.id), my_person.instance_id)
      .filter(post_actions::person_id.eq(my_person.id))
      .filter(post_actions::read_at.is_not_null())
      .filter(filter_blocked())
      .limit(limit)
      .select(PostView::as_select())
      .into_boxed();

    // Sorting by the read date
    let paginated_query =
      PostViewDummy::paginate(query, &page_cursor, SortDirection::Desc, pool, None)
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
    let query = PostView::joins(Some(my_person.id), my_person.instance_id)
      .filter(post_actions::person_id.eq(my_person.id))
      .filter(post_actions::hidden_at.is_not_null())
      .filter(filter_blocked())
      .limit(limit)
      .select(PostView::as_select())
      .into_boxed();

    // Sorting by the hidden date
    let paginated_query =
      PostViewDummy::paginate(query, &page_cursor, SortDirection::Desc, pool, None)
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
  pub multi_community_id: Option<MultiCommunityId>,
  pub local_user: Option<&'a LocalUser>,
  pub show_hidden: Option<bool>,
  pub show_read: Option<bool>,
  pub show_nsfw: Option<bool>,
  pub hide_media: Option<bool>,
  pub no_comments_only: Option<bool>,
  pub keyword_blocks: Option<Vec<String>>,
  pub page_cursor: Option<PaginationCursor>,
  /// For backwards compat with API v3 (not available on API v4).
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PostQuery<'_> {
  async fn prefetch_cursor_before_data(
    &self,
    site: &Site,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Option<Post>> {
    // first get one page for the most popular community to get an upper bound for the page end for
    // the real query. the reason this is needed is that when fetching posts for a single
    // community PostgreSQL can optimize the query to use an index on e.g. (=, >=, >=, >=) and
    // fetch only LIMIT rows but for the followed-communities query it has to query the index on
    // (IN, >=, >=, >=) which it currently can't do at all (as of PG 16). see the discussion
    // here: https://github.com/LemmyNet/lemmy/issues/2877#issuecomment-1673597190
    //
    // the results are correct no matter which community we fetch these for, since it basically
    // covers the "worst case" of the whole page consisting of posts from one community
    // but using the largest community decreases the pagination-frame so make the real query more
    // efficient.

    // If its a subscribed type, you need to prefetch both the largest community, and the upper
    // bound post for the cursor.
    Ok(if self.listing_type == Some(ListingType::Subscribed) {
      if let Some(person_id) = self.local_user.person_id() {
        let largest_subscribed =
          CommunityActions::fetch_largest_subscribed_community(pool, person_id).await?;

        let upper_bound_results: Vec<PostView> = self
          .clone()
          .list_inner(site, None, largest_subscribed, pool)
          .await?
          .items;

        let limit = limit_fetch(self.limit, None)?;

        // take last element of array. if this query returned less than LIMIT elements,
        // the heuristic is invalid since we can't guarantee the full query will return >= LIMIT
        // results (return original query)
        let len: i64 = upper_bound_results.len().try_into()?;
        if len < limit {
          None
        } else {
          if self
            .page_cursor
            .clone()
            .and_then(|c| c.is_back().ok())
            .unwrap_or_default()
          {
            // for backward pagination, get first element instead
            upper_bound_results.into_iter().next()
          } else {
            upper_bound_results.into_iter().next_back()
          }
          .map(|pv| pv.post)
        }
      } else {
        None
      }
    } else {
      None
    })
  }

  async fn list_inner(
    self,
    site: &Site,
    cursor_before_data: Option<Post>,
    largest_subscribed_for_prefetch: Option<CommunityId>,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<PagedResponse<PostView>> {
    let o = self;
    let limit = limit_fetch(o.limit, None)?;

    let my_person_id = o.local_user.person_id();
    let my_local_user_id = o.local_user.local_user_id();

    let mut query = PostView::joins(my_person_id, site.instance_id)
      .select(PostView::as_select())
      .limit(limit)
      .into_boxed();

    if let Some(page) = o.page {
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

    match (o.community_id, o.multi_community_id) {
      (Some(id), None) => {
        query = query.filter(post::community_id.eq(id));
      }
      (None, Some(id)) => {
        let communities = multi_community_entry::table
          .filter(multi_community_entry::multi_community_id.eq(id))
          .select(multi_community_entry::community_id);
        query = query.filter(post::community_id.eq_any(communities))
      }
      (Some(_), Some(_)) => {
        return Err(LemmyErrorType::CannotCombineCommunityIdAndMultiCommunityId.into());
      }
      (None, None) => {
        if let (Some(ListingType::Subscribed), Some(id)) =
          (o.listing_type, largest_subscribed_for_prefetch)
        {
          query = query.filter(post::community_id.eq(id));
        }
      }
    }

    match o.listing_type.unwrap_or_default() {
      // TODO we might have much better performance by using post::community_id.eq_any()
      ListingType::Subscribed => query = query.filter(filter_is_subscribed()),
      ListingType::Local => {
        query = query
          .filter(community::local.eq(true))
          .filter(filter_not_unlisted_or_is_subscribed());
      }
      ListingType::All => query = query.filter(filter_not_unlisted_or_is_subscribed()),
      ListingType::ModeratorView => {
        query = query.filter(community_actions::became_moderator_at.is_not_null());
      }
      ListingType::Suggested => query = query.filter(filter_suggested_communities()),
    }

    if !o.show_nsfw.unwrap_or(o.local_user.show_nsfw(site)) {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    if !o.local_user.show_bot_accounts() {
      query = query.filter(person::bot_account.eq(false));
    };

    // Filter to show only posts with no comments
    if o.no_comments_only.unwrap_or_default() {
      query = query.filter(post::comments.eq(0));
    };

    if !o.show_read.unwrap_or(o.local_user.show_read_posts()) {
      query = query.filter(post_actions::read_at.is_null());
    }

    // Hide the hidden posts
    if !o.show_hidden.unwrap_or_default() {
      query = query.filter(post_actions::hidden_at.is_null());
    }

    if o.hide_media.unwrap_or(o.local_user.hide_media()) {
      query = query.filter(not(
        post::url_content_type.is_not_null().and(
          post::url_content_type
            .like("image/%")
            .or(post::url_content_type.like("video/%")),
        ),
      ));
    }

    query = o.local_user.visible_communities_only(query);
    query = query.filter(
      post::federation_pending
        .eq(false)
        .or(post::creator_id.nullable().eq(my_person_id)),
    );

    if !o.local_user.is_admin() {
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
    if o.listing_type.unwrap_or_default() != ListingType::ModeratorView {
      // Filter out the rows with missing languages if user is logged in
      if o.local_user.is_some() {
        query = query.filter(exists(
          local_user_language::table.filter(
            post::language_id.eq(local_user_language::language_id).and(
              local_user_language::local_user_id
                .nullable()
                .eq(my_local_user_id),
            ),
          ),
        ));
      }

      query = query.filter(filter_blocked());

      if let Some(keyword_blocks) = o.keyword_blocks {
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
    if let Some(time_range_seconds) = o.time_range_seconds {
      query =
        query.filter(post::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)));
    }

    // Only sort by ascending for Old
    let sort = o.sort.unwrap_or(PostSortType::Hot);
    let sort_direction = asc_if(sort == PostSortType::Old);

    let mut pq = PostView::paginate(
      query,
      &o.page_cursor,
      sort_direction,
      pool,
      cursor_before_data,
    )
    .await?;

    // featured posts first
    // Don't do for new / old sorts
    if sort != PostSortType::New && sort != PostSortType::Old {
      pq = if o.community_id.is_none() || largest_subscribed_for_prefetch.is_some() {
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
    paginate_response(res, limit, o.page_cursor)
  }

  pub async fn list(
    &self,
    site: &Site,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<PagedResponse<PostView>> {
    let cursor_before_data = self.prefetch_cursor_before_data(site, pool).await?;

    self
      .clone()
      .list_inner(site, cursor_before_data, None, pool)
      .await
  }
}
