use crate::structs::{PaginationCursor, PostView};
use diesel::{
  debug_query,
  dsl::{exists, not},
  pg::Pg,
  query_builder::AsQuery,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  OptionalExtension,
  PgTextExpressionMethods,
  QueryDsl,
  TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::{post_aggregates_keys as key, PostAggregates},
  aliases::{creator_community_actions, creator_local_user},
  impls::{
    community::community_follower_select_subscribed_type,
    local_user::{local_user_can_mod, LocalUserOptionHelper},
  },
  newtypes::{CommunityId, PersonId, PostId},
  schema::{
    community,
    community_actions,
    image_details,
    instance_actions,
    local_user,
    local_user_language,
    person,
    person_actions,
    post,
    post_actions,
    post_aggregates,
    post_tag,
    tag,
  },
  source::{
    community::CommunityFollowerState,
    local_user::LocalUser,
    post::{post_actions_keys, PostActionsCursor},
    site::Site,
  },
  utils::{
    functions::coalesce,
    fuzzy_search,
    get_conn,
    limit_and_offset,
    now,
    paginate,
    seconds_to_pg_interval,
    Commented,
    DbPool,
    ReverseTimestampKey,
  },
  CommunityVisibility,
  ListingType,
  PostSortType,
};
use tracing::debug;
use PostSortType::*;

impl PostView {
  // TODO while we can abstract the joins into a function, the selects are currently impossible to
  // do, because they rely on a few types that aren't yet publicly exported in diesel:
  // https://github.com/diesel-rs/diesel/issues/4462

  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>) -> _ {
    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(post_aggregates::community_id)
        .and(community_actions::person_id.nullable().eq(my_person_id)),
    );

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(post_aggregates::creator_id)
        .and(person_actions::person_id.nullable().eq(my_person_id)),
    );

    let post_actions_join = post_actions::table.on(
      post_actions::post_id
        .eq(post_aggregates::post_id)
        .and(post_actions::person_id.nullable().eq(my_person_id)),
    );

    let instance_actions_join = instance_actions::table.on(
      instance_actions::instance_id
        .eq(post_aggregates::instance_id)
        .and(instance_actions::person_id.nullable().eq(my_person_id)),
    );

    let post_creator_community_actions_join = creator_community_actions.on(
      creator_community_actions
        .field(community_actions::community_id)
        .eq(post_aggregates::community_id)
        .and(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(post_aggregates::creator_id),
        ),
    );

    let image_details_join =
      image_details::table.on(post::thumbnail_url.eq(image_details::link.nullable()));

    let local_user_join = local_user::table.on(local_user::person_id.nullable().eq(my_person_id));

    post_aggregates::table
      .inner_join(person::table)
      .inner_join(community::table)
      .inner_join(post::table)
      .left_join(image_details_join)
      .left_join(community_actions_join)
      .left_join(person_actions_join)
      .left_join(post_actions_join)
      .left_join(instance_actions_join)
      .left_join(post_creator_community_actions_join)
      .left_join(local_user_join)
  }

  #[diesel::dsl::auto_type(no_type_alias)]
  fn creator_is_admin() -> _ {
    exists(
      creator_local_user.filter(
        post_aggregates::creator_id
          .eq(creator_local_user.field(local_user::person_id))
          .and(creator_local_user.field(local_user::admin).eq(true)),
      ),
    )
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    my_local_user: Option<&'_ LocalUser>,
    is_mod_or_admin: bool,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let my_person_id = my_local_user.person_id();

    let post_tags = post_tag::table
      .inner_join(tag::table)
      .select(diesel::dsl::sql::<diesel::sql_types::Json>(
        "json_agg(tag.*)",
      ))
      .filter(post_tag::post_id.eq(post_aggregates::post_id))
      .filter(tag::deleted.eq(false))
      .single_value();

    let mut query = Self::joins(my_person_id)
      .filter(post_aggregates::post_id.eq(post_id))
      .select((
        post::all_columns,
        person::all_columns,
        community::all_columns,
        image_details::all_columns.nullable(),
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        community_actions::received_ban.nullable().is_not_null(),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        Self::creator_is_admin(),
        post_aggregates::all_columns,
        community_follower_select_subscribed_type(),
        post_actions::saved.nullable(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
          post_aggregates::comments,
        ),
        post_tags,
        local_user_can_mod(),
      ))
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
        // users can see their own deleted posts
        .filter(
          community::deleted
            .eq(false)
            .or(post::creator_id.nullable().eq(my_person_id)),
        )
        .filter(
          post::deleted
            .eq(false)
            .or(post::creator_id.nullable().eq(my_person_id)),
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
  }
}

impl PaginationCursor {
  // get cursor for page that starts immediately after the given post
  pub fn after_post(view: &PostView) -> PaginationCursor {
    // hex encoding to prevent ossification
    PaginationCursor(format!("P{:x}", view.counts.post_id.0))
  }
  pub async fn read(
    &self,
    pool: &mut DbPool<'_>,
    local_user: Option<&LocalUser>,
  ) -> Result<PaginationCursorData, Error> {
    let err_msg = || Error::QueryBuilderError("Could not parse pagination token".into());
    let post_id = PostId(
      self
        .0
        .get(1..)
        .and_then(|e| i32::from_str_radix(e, 16).ok())
        .ok_or_else(err_msg)?,
    );
    let post_aggregates = PostAggregates::read(pool, post_id).await?;
    let post_actions = PostActionsCursor::read(pool, post_id, local_user.person_id()).await?;

    Ok(PaginationCursorData {
      post_aggregates,
      post_actions,
    })
  }
}

// currently we use aggregates or actions as the pagination token.
// we only use some of the properties, depending on which sort type we page by
#[derive(Clone)]
pub struct PaginationCursorData {
  post_aggregates: PostAggregates,
  post_actions: PostActionsCursor,
}

#[derive(Clone, Default)]
pub struct PostQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<PostSortType>,
  pub time_range_seconds: Option<i32>,
  pub creator_id: Option<PersonId>,
  pub community_id: Option<CommunityId>,
  // if true, the query should be handled as if community_id was not given except adding the
  // literal filter
  pub community_id_just_for_prefetch: bool,
  pub local_user: Option<&'a LocalUser>,
  pub search_term: Option<String>,
  pub url_only: Option<bool>,
  pub read_only: Option<bool>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  pub title_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub page_after: Option<PaginationCursorData>,
  pub page_before_or_equal: Option<PostAggregates>,
  pub page_back: Option<bool>,
  pub show_hidden: Option<bool>,
  pub show_read: Option<bool>,
  pub show_nsfw: Option<bool>,
  pub hide_media: Option<bool>,
  pub no_comments_only: Option<bool>,
}

impl<'a> PostQuery<'a> {
  #[allow(clippy::expect_used)]
  async fn prefetch_upper_bound_for_page_before(
    &self,
    site: &Site,
    pool: &mut DbPool<'_>,
  ) -> Result<Option<PostQuery<'a>>, Error> {
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
    use lemmy_db_schema::schema::community_aggregates::dsl::{
      community_aggregates,
      community_id,
      users_active_month,
    };
    let (limit, offset) = limit_and_offset(self.page, self.limit)?;
    if offset != 0 && self.page_after.is_some() {
      return Err(Error::QueryBuilderError(
        "legacy pagination cannot be combined with v2 pagination".into(),
      ));
    }
    let self_person_id = self.local_user.expect("part of the above if").person_id;
    let largest_subscribed = {
      let conn = &mut get_conn(pool).await?;
      community_actions::table
        .filter(community_actions::followed.is_not_null())
        .filter(community_actions::person_id.eq(self_person_id))
        .inner_join(community_aggregates.on(community_id.eq(community_actions::community_id)))
        .order_by(users_active_month.desc())
        .select(community_id)
        .limit(1)
        .get_result::<CommunityId>(conn)
        .await
        .optional()?
    };
    let Some(largest_subscribed) = largest_subscribed else {
      // nothing subscribed to? no posts
      return Ok(None);
    };

    let mut v = Box::pin(
      PostQuery {
        community_id: Some(largest_subscribed),
        community_id_just_for_prefetch: true,
        ..self.clone()
      }
      .list(site, pool),
    )
    .await?;

    // take last element of array. if this query returned less than LIMIT elements,
    // the heuristic is invalid since we can't guarantee the full query will return >= LIMIT results
    // (return original query)
    if (v.len() as i64) < limit {
      Ok(Some(self.clone()))
    } else {
      let item = if self.page_back.unwrap_or_default() {
        // for backward pagination, get first element instead
        v.into_iter().next()
      } else {
        v.pop()
      };
      let limit_cursor = Some(item.expect("else case").counts);
      Ok(Some(PostQuery {
        page_before_or_equal: limit_cursor,
        ..self.clone()
      }))
    }
  }

  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> Result<Vec<PostView>, Error> {
    let o = if self.listing_type == Some(ListingType::Subscribed)
      && self.community_id.is_none()
      && self.local_user.is_some()
      && self.page_before_or_equal.is_none()
    {
      if let Some(query) = self
        .prefetch_upper_bound_for_page_before(site, pool)
        .await?
      {
        query
      } else {
        self
      }
    } else {
      self
    };

    let conn = &mut get_conn(pool).await?;

    let my_person_id = o.local_user.person_id();
    let my_local_user_id = o.local_user.local_user_id();

    let post_tags = post_tag::table
      .inner_join(tag::table)
      .select(diesel::dsl::sql::<diesel::sql_types::Json>(
        "json_agg(tag.*)",
      ))
      .filter(post_tag::post_id.eq(post_aggregates::post_id))
      .filter(tag::deleted.eq(false))
      .single_value();

    let mut query = PostView::joins(my_person_id)
      .select((
        post::all_columns,
        person::all_columns,
        community::all_columns,
        image_details::all_columns.nullable(),
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        community_actions::received_ban.nullable().is_not_null(),
        creator_community_actions
          .field(community_actions::became_moderator)
          .nullable()
          .is_not_null(),
        PostView::creator_is_admin(),
        post_aggregates::all_columns,
        community_follower_select_subscribed_type(),
        post_actions::saved.nullable(),
        post_actions::read.nullable().is_not_null(),
        post_actions::hidden.nullable().is_not_null(),
        person_actions::blocked.nullable().is_not_null(),
        post_actions::like_score.nullable(),
        coalesce(
          post_aggregates::comments.nullable() - post_actions::read_comments_amount.nullable(),
          post_aggregates::comments,
        ),
        post_tags,
        local_user_can_mod(),
      ))
      .into_boxed();

    // hide posts from deleted communities
    query = query.filter(community::deleted.eq(false));

    // only creator can see deleted posts and unpublished scheduled posts
    if let Some(person_id) = o.local_user.person_id() {
      query = query.filter(post::deleted.eq(false).or(post::creator_id.eq(person_id)));
      query = query.filter(
        post::scheduled_publish_time
          .is_null()
          .or(post::creator_id.eq(person_id)),
      );
    } else {
      query = query
        .filter(post::deleted.eq(false))
        .filter(post::scheduled_publish_time.is_null());
    }

    // only show removed posts to admin when viewing user profile
    if !(o.creator_id.is_some() && o.local_user.is_admin()) {
      query = query
        .filter(community::removed.eq(false))
        .filter(post::removed.eq(false));
    }
    if let Some(community_id) = o.community_id {
      query = query.filter(post_aggregates::community_id.eq(community_id));
    }

    if let Some(creator_id) = o.creator_id {
      query = query.filter(post_aggregates::creator_id.eq(creator_id));
    }

    let is_subscribed = community_actions::followed.is_not_null();
    match o.listing_type.unwrap_or_default() {
      ListingType::Subscribed => query = query.filter(is_subscribed),
      ListingType::Local => {
        query = query
          .filter(community::local.eq(true))
          .filter(community::hidden.eq(false).or(is_subscribed));
      }
      ListingType::All => query = query.filter(community::hidden.eq(false).or(is_subscribed)),
      ListingType::ModeratorView => {
        query = query.filter(community_actions::became_moderator.is_not_null());
      }
    }

    if let Some(search_term) = &o.search_term {
      if o.url_only.unwrap_or_default() {
        query = query.filter(post::url.eq(search_term));
      } else {
        let searcher = fuzzy_search(search_term);
        let name_filter = post::name.ilike(searcher.clone());
        let body_filter = post::body.ilike(searcher.clone());
        query = if o.title_only.unwrap_or_default() {
          query.filter(name_filter)
        } else {
          query.filter(name_filter.or(body_filter))
        }
        .filter(not(post::removed.or(post::deleted)));
      }
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
      query = query.filter(post_aggregates::comments.eq(0));
    };

    if !o.show_read.unwrap_or(o.local_user.show_read_posts()) {
      // Do not hide read posts when it is a user profile view
      // Or, only hide read posts on non-profile views
      if o.creator_id.is_none() {
        query = query.filter(post_actions::read.is_null());
      }
    }

    // If a creator id isn't given (IE its on home or community pages), hide the hidden posts
    if !o.show_hidden.unwrap_or_default() && o.creator_id.is_none() {
      query = query.filter(post_actions::hidden.is_null());
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

    if let Some(my_id) = o.local_user.person_id() {
      let not_creator_filter = post_aggregates::creator_id.ne(my_id);
      if o.liked_only.unwrap_or_default() {
        query = query
          .filter(not_creator_filter)
          .filter(post_actions::like_score.eq(1));
      } else if o.disliked_only.unwrap_or_default() {
        query = query
          .filter(not_creator_filter)
          .filter(post_actions::like_score.eq(-1));
      }
    };

    query = o.local_user.visible_communities_only(query);

    if !o.local_user.is_admin() {
      query = query.filter(
        community::visibility
          .ne(CommunityVisibility::Private)
          .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
      );
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

      // Don't show blocked instances, communities or persons
      query = query.filter(community_actions::blocked.is_null());
      query = query.filter(instance_actions::blocked.is_null());
      query = query.filter(person_actions::blocked.is_null());
    }

    let (limit, offset) = limit_and_offset(o.page, o.limit)?;
    query = query.limit(limit).offset(offset);

    let query = if o.read_only.unwrap_or_default() {
      paginate(
        query,
        o.page_after.map(|c| c.post_actions),
        None,
        o.page_back.unwrap_or_default(),
      )
      .filter(post_actions::read.is_not_null())
      .then_desc(post_actions_keys::read)
      .as_query()
    } else {
      let mut query = paginate(
        query,
        o.page_after.map(|c| c.post_aggregates),
        o.page_before_or_equal,
        o.page_back.unwrap_or_default(),
      );

      // featured posts first
      query = if o.community_id.is_none() || o.community_id_just_for_prefetch {
        query.then_desc(key::featured_local)
      } else {
        query.then_desc(key::featured_community)
      };

      // then use the main sort
      query = match o.sort.unwrap_or(Hot) {
        Active => query.then_desc(key::hot_rank_active),
        Hot => query.then_desc(key::hot_rank),
        Scaled => query.then_desc(key::scaled_rank),
        Controversial => query.then_desc(key::controversy_rank),
        New => query.then_desc(key::published),
        Old => query.then_desc(ReverseTimestampKey(key::published)),
        NewComments => query.then_desc(key::newest_comment_time),
        MostComments => query.then_desc(key::comments),
        Top => query.then_desc(key::score),
      };

      // Filter by the time range
      if let Some(time_range_seconds) = o.time_range_seconds {
        query = query.filter(
          post_aggregates::published.gt(now() - seconds_to_pg_interval(time_range_seconds)),
        );
      }

      // use publish as fallback. especially useful for hot rank which reaches zero after some days.
      // necessary because old posts can be fetched over federation and inserted with high post id
      query = match o.sort.unwrap_or(Hot) {
        // A second time-based sort would not be very useful
        New | Old | NewComments => query,
        _ => query.then_desc(key::published),
      };

      // finally use unique post id as tie breaker
      query = query.then_desc(key::post_id);

      query.as_query()
    };

    debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));

    Commented::new(query)
      .text("PostQuery::list")
      .text_if(
        "getting upper bound for next query",
        o.community_id_just_for_prefetch,
      )
      .load::<PostView>(conn)
      .await
  }
}

#[allow(clippy::indexing_slicing)]
#[expect(clippy::expect_used)]
#[cfg(test)]
mod tests {
  use crate::{
    post::post_view::{PaginationCursorData, PostQuery, PostView},
    structs::{LocalUserView, PostTags},
  };
  use chrono::Utc;
  use diesel_async::SimpleAsyncConnection;
  use lemmy_db_schema::{
    aggregates::structs::PostAggregates,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::LanguageId,
    source::{
      actor_language::LocalUserLanguage,
      comment::{Comment, CommentInsertForm},
      community::{
        Community,
        CommunityFollower,
        CommunityFollowerForm,
        CommunityFollowerState,
        CommunityInsertForm,
        CommunityModerator,
        CommunityModeratorForm,
        CommunityPersonBan,
        CommunityPersonBanForm,
        CommunityUpdateForm,
      },
      community_block::{CommunityBlock, CommunityBlockForm},
      instance::Instance,
      instance_block::{InstanceBlock, InstanceBlockForm},
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      local_user_vote_display_mode::LocalUserVoteDisplayMode,
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      post::{
        Post,
        PostHide,
        PostInsertForm,
        PostLike,
        PostLikeForm,
        PostRead,
        PostReadForm,
        PostUpdateForm,
      },
      site::Site,
      tag::{PostTagInsertForm, Tag, TagInsertForm},
    },
    traits::{Bannable, Blockable, Crud, Followable, Joinable, Likeable},
    utils::{build_db_pool, get_conn, uplete, ActualDbPool, DbPool, RANK_DEFAULT},
    CommunityVisibility,
    PostSortType,
    SubscribedType,
  };
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use std::time::{Duration, Instant};
  use test_context::{test_context, AsyncTestContext};
  use url::Url;

  const POST_WITH_ANOTHER_TITLE: &str = "Another title";
  const POST_BY_BLOCKED_PERSON: &str = "post by blocked person";
  const POST_BY_BOT: &str = "post by bot";
  const POST: &str = "post";
  const POST_WITH_TAGS: &str = "post with tags";

  fn names(post_views: &[PostView]) -> Vec<&str> {
    post_views.iter().map(|i| i.post.name.as_str()).collect()
  }

  struct Data {
    pool: ActualDbPool,
    instance: Instance,
    tegan_local_user_view: LocalUserView,
    john_local_user_view: LocalUserView,
    bot_local_user_view: LocalUserView,
    community: Community,
    post: Post,
    bot_post: Post,
    post_with_tags: Post,
    tag_1: Tag,
    tag_2: Tag,
    site: Site,
  }

  impl Data {
    fn pool(&self) -> ActualDbPool {
      self.pool.clone()
    }
    pub fn pool2(&self) -> DbPool<'_> {
      DbPool::Pool(&self.pool)
    }
    fn default_post_query(&self) -> PostQuery<'_> {
      PostQuery {
        sort: Some(PostSortType::New),
        local_user: Some(&self.tegan_local_user_view.local_user),
        ..Default::default()
      }
    }

    async fn setup() -> LemmyResult<Data> {
      let actual_pool = build_db_pool()?;
      let pool = &mut (&actual_pool).into();
      let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

      let tegan_person_form = PersonInsertForm::test_form(instance.id, "tegan");
      let inserted_tegan_person = Person::create(pool, &tegan_person_form).await?;
      let tegan_local_user_form = LocalUserInsertForm {
        admin: Some(true),
        ..LocalUserInsertForm::test_form(inserted_tegan_person.id)
      };
      let inserted_tegan_local_user =
        LocalUser::create(pool, &tegan_local_user_form, vec![]).await?;

      let bot_person_form = PersonInsertForm {
        bot_account: Some(true),
        ..PersonInsertForm::test_form(instance.id, "mybot")
      };
      let inserted_bot_person = Person::create(pool, &bot_person_form).await?;
      let inserted_bot_local_user = LocalUser::create(
        pool,
        &LocalUserInsertForm::test_form(inserted_bot_person.id),
        vec![],
      )
      .await?;

      let new_community = CommunityInsertForm::new(
        instance.id,
        "test_community_3".to_string(),
        "nada".to_owned(),
        "pubkey".to_string(),
      );
      let community = Community::create(pool, &new_community).await?;

      // Test a person block, make sure the post query doesn't include their post
      let john_person_form = PersonInsertForm::test_form(instance.id, "john");
      let inserted_john_person = Person::create(pool, &john_person_form).await?;
      let inserted_john_local_user = LocalUser::create(
        pool,
        &LocalUserInsertForm::test_form(inserted_john_person.id),
        vec![],
      )
      .await?;

      let post_from_blocked_person = PostInsertForm {
        language_id: Some(LanguageId(1)),
        ..PostInsertForm::new(
          POST_BY_BLOCKED_PERSON.to_string(),
          inserted_john_person.id,
          community.id,
        )
      };
      Post::create(pool, &post_from_blocked_person).await?;

      // block that person
      let person_block = PersonBlockForm {
        person_id: inserted_tegan_person.id,
        target_id: inserted_john_person.id,
      };

      PersonBlock::block(pool, &person_block).await?;

      // Two community post tags
      let tag_1 = Tag::create(
        pool,
        &TagInsertForm {
          ap_id: Url::parse(&format!("{}/tags/test_tag1", community.ap_id))?.into(),
          name: "Test Tag 1".into(),
          community_id: community.id,
          published: None,
          updated: None,
          deleted: false,
        },
      )
      .await?;
      let tag_2 = Tag::create(
        pool,
        &TagInsertForm {
          ap_id: Url::parse(&format!("{}/tags/test_tag2", community.ap_id))?.into(),
          name: "Test Tag 2".into(),
          community_id: community.id,
          published: None,
          updated: None,
          deleted: false,
        },
      )
      .await?;

      // A sample post
      let new_post = PostInsertForm {
        language_id: Some(LanguageId(47)),
        ..PostInsertForm::new(POST.to_string(), inserted_tegan_person.id, community.id)
      };

      let post = Post::create(pool, &new_post).await?;

      let new_bot_post = PostInsertForm::new(
        POST_BY_BOT.to_string(),
        inserted_bot_person.id,
        community.id,
      );
      let bot_post = Post::create(pool, &new_bot_post).await?;

      // A sample post with tags
      let new_post = PostInsertForm {
        language_id: Some(LanguageId(47)),
        ..PostInsertForm::new(
          POST_WITH_TAGS.to_string(),
          inserted_tegan_person.id,
          community.id,
        )
      };

      let post_with_tags = Post::create(pool, &new_post).await?;
      let inserted_tags = vec![
        PostTagInsertForm {
          post_id: post_with_tags.id,
          tag_id: tag_1.id,
        },
        PostTagInsertForm {
          post_id: post_with_tags.id,
          tag_id: tag_2.id,
        },
      ];
      PostTagInsertForm::insert_tag_associations(pool, &inserted_tags).await?;

      let tegan_local_user_view = LocalUserView {
        local_user: inserted_tegan_local_user,
        local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
        person: inserted_tegan_person,
        counts: Default::default(),
      };
      let john_local_user_view = LocalUserView {
        local_user: inserted_john_local_user,
        local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
        person: inserted_john_person,
        counts: Default::default(),
      };

      let bot_local_user_view = LocalUserView {
        local_user: inserted_bot_local_user,
        local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
        person: inserted_bot_person,
        counts: Default::default(),
      };

      let site = Site {
        id: Default::default(),
        name: String::new(),
        sidebar: None,
        published: Default::default(),
        updated: None,
        icon: None,
        banner: None,
        description: None,
        ap_id: Url::parse("http://example.com")?.into(),
        last_refreshed_at: Default::default(),
        inbox_url: Url::parse("http://example.com")?.into(),
        private_key: None,
        public_key: String::new(),
        instance_id: Default::default(),
        content_warning: None,
      };

      Ok(Data {
        pool: actual_pool,
        instance,
        tegan_local_user_view,
        john_local_user_view,
        bot_local_user_view,
        community,
        post,
        bot_post,
        post_with_tags,
        tag_1,
        tag_2,
        site,
      })
    }
    async fn teardown(data: Data) -> LemmyResult<()> {
      let pool = &mut data.pool2();
      // let pool = &mut (&pool).into();
      let num_deleted = Post::delete(pool, data.post.id).await?;
      Community::delete(pool, data.community.id).await?;
      Person::delete(pool, data.tegan_local_user_view.person.id).await?;
      Person::delete(pool, data.bot_local_user_view.person.id).await?;
      Person::delete(pool, data.john_local_user_view.person.id).await?;
      Instance::delete(pool, data.instance.id).await?;
      assert_eq!(1, num_deleted);

      Ok(())
    }
  }
  impl AsyncTestContext for Data {
    async fn setup() -> Self {
      Data::setup().await.expect("setup failed")
    }
    async fn teardown(self) {
      Data::teardown(self).await.expect("teardown failed")
    }
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_with_person(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    LocalUser::update(
      pool,
      data.tegan_local_user_view.local_user.id,
      &local_user_form,
    )
    .await?;
    data.tegan_local_user_view.local_user.show_bot_accounts = false;

    let mut read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    // remove tags post
    read_post_listing.remove(0);

    let post_listing_single_with_person = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      false,
    )
    .await?;

    let expected_post_listing_with_user = expected_post_view(data, pool).await?;

    // Should be only one person, IE the bot post, and blocked should be missing
    assert_eq!(
      vec![post_listing_single_with_person.clone()],
      read_post_listing
    );
    assert_eq!(
      expected_post_listing_with_user,
      post_listing_single_with_person
    );

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(true),
      ..Default::default()
    };
    LocalUser::update(
      pool,
      data.tegan_local_user_view.local_user.id,
      &local_user_form,
    )
    .await?;
    data.tegan_local_user_view.local_user.show_bot_accounts = true;

    let post_listings_with_bots = PostQuery {
      community_id: Some(data.community.id),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    // should include bot post which has "undetermined" language
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_with_bots)
    );
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_no_person(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let read_post_listing_multiple_no_person = PostQuery {
      community_id: Some(data.community.id),
      local_user: None,
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;

    let read_post_listing_single_no_person =
      PostView::read(pool, data.post.id, None, false).await?;

    let mut expected_post_listing_no_person = expected_post_view(data, pool).await?;
    expected_post_listing_no_person.can_mod = false;

    // Should be 2 posts, with the bot post, and the blocked
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST, POST_BY_BLOCKED_PERSON],
      names(&read_post_listing_multiple_no_person)
    );

    assert_eq!(
      Some(&expected_post_listing_no_person),
      read_post_listing_multiple_no_person.get(2)
    );
    assert_eq!(
      expected_post_listing_no_person,
      read_post_listing_single_no_person
    );
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_title_only(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // A post which contains the search them 'Post' not in the title (but in the body)
    let new_post = PostInsertForm {
      language_id: Some(LanguageId(47)),
      body: Some("Post".to_string()),
      ..PostInsertForm::new(
        POST_WITH_ANOTHER_TITLE.to_string(),
        data.tegan_local_user_view.person.id,
        data.community.id,
      )
    };

    let inserted_post = Post::create(pool, &new_post).await?;

    let read_post_listing_by_title_only = PostQuery {
      community_id: Some(data.community.id),
      local_user: None,
      search_term: Some("Post".to_string()),
      title_only: Some(true),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;

    let read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      local_user: None,
      search_term: Some("Post".to_string()),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;

    // Should be 4 posts when we do not search for title only
    assert_eq!(
      vec![
        POST_WITH_ANOTHER_TITLE,
        POST_WITH_TAGS,
        POST_BY_BOT,
        POST,
        POST_BY_BLOCKED_PERSON
      ],
      names(&read_post_listing)
    );

    // Should be 3 posts when we search for title only
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST, POST_BY_BLOCKED_PERSON],
      names(&read_post_listing_by_title_only)
    );
    Post::delete(pool, inserted_post.id).await?;
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_block_community(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let community_block = CommunityBlockForm {
      person_id: data.tegan_local_user_view.person.id,
      community_id: data.community.id,
    };
    CommunityBlock::block(pool, &community_block).await?;

    let read_post_listings_with_person_after_block = PostQuery {
      community_id: Some(data.community.id),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    // Should be 0 posts after the community block
    assert_eq!(read_post_listings_with_person_after_block, vec![]);

    CommunityBlock::unblock(pool, &community_block).await?;
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_like(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let post_like_form = PostLikeForm::new(data.post.id, data.tegan_local_user_view.person.id, 1);

    let inserted_post_like = PostLike::like(pool, &post_like_form).await?;

    let expected_post_like = PostLike {
      post_id: data.post.id,
      person_id: data.tegan_local_user_view.person.id,
      published: inserted_post_like.published,
      score: 1,
    };
    assert_eq!(expected_post_like, inserted_post_like);

    let post_listing_single_with_person = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      false,
    )
    .await?;

    let mut expected_post_with_upvote = expected_post_view(data, pool).await?;
    expected_post_with_upvote.my_vote = Some(1);
    expected_post_with_upvote.counts.score = 1;
    expected_post_with_upvote.counts.upvotes = 1;
    assert_eq!(expected_post_with_upvote, post_listing_single_with_person);

    let local_user_form = LocalUserUpdateForm {
      show_bot_accounts: Some(false),
      ..Default::default()
    };
    LocalUser::update(
      pool,
      data.tegan_local_user_view.local_user.id,
      &local_user_form,
    )
    .await?;
    data.tegan_local_user_view.local_user.show_bot_accounts = false;

    let mut read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    read_post_listing.remove(0);
    assert_eq!(vec![expected_post_with_upvote], read_post_listing);

    let like_removed =
      PostLike::remove(pool, data.tegan_local_user_view.person.id, data.post.id).await?;
    assert_eq!(uplete::Count::only_deleted(1), like_removed);
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_liked_only(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Like both the bot post, and your own
    // The liked_only should not show your own post
    let post_like_form = PostLikeForm::new(data.post.id, data.tegan_local_user_view.person.id, 1);
    PostLike::like(pool, &post_like_form).await?;

    let bot_post_like_form =
      PostLikeForm::new(data.bot_post.id, data.tegan_local_user_view.person.id, 1);
    PostLike::like(pool, &bot_post_like_form).await?;

    // Read the liked only
    let read_liked_post_listing = PostQuery {
      community_id: Some(data.community.id),
      liked_only: Some(true),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;

    // This should only include the bot post, not the one you created
    assert_eq!(vec![POST_BY_BOT], names(&read_liked_post_listing));

    let read_disliked_post_listing = PostQuery {
      community_id: Some(data.community.id),
      disliked_only: Some(true),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;

    // Should be no posts
    assert_eq!(read_disliked_post_listing, vec![]);

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_read_only(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Only mark the bot post as read
    // The read_only should only show the bot post
    let post_read_form = PostReadForm::new(data.bot_post.id, data.tegan_local_user_view.person.id);
    PostRead::mark_as_read(pool, &post_read_form).await?;

    // Only read the post marked as read
    let read_read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      read_only: Some(true),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;

    // This should only include the bot post, not the one you created
    assert_eq!(vec![POST_BY_BOT], names(&read_read_post_listing));

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn creator_info(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();
    let community_id = data.community.id;

    let tegan_listings = PostQuery {
      community_id: Some(community_id),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?
    .into_iter()
    .map(|p| {
      (
        p.creator.name,
        p.creator_is_moderator,
        p.creator_is_admin,
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    // Tegan is an admin, so can_mod should be always true
    let expected_post_listing = vec![
      ("tegan".to_owned(), false, true, true),
      ("mybot".to_owned(), false, false, true),
      ("tegan".to_owned(), false, true, true),
    ];
    assert_eq!(expected_post_listing, tegan_listings);

    // Have john become a moderator, then the bot
    let john_mod_form = CommunityModeratorForm {
      community_id,
      person_id: data.john_local_user_view.person.id,
    };
    CommunityModerator::join(pool, &john_mod_form).await?;

    let bot_mod_form = CommunityModeratorForm {
      community_id,
      person_id: data.bot_local_user_view.person.id,
    };
    CommunityModerator::join(pool, &bot_mod_form).await?;

    let john_listings = PostQuery {
      sort: Some(PostSortType::New),
      local_user: Some(&data.john_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .into_iter()
    .map(|p| {
      (
        p.creator.name,
        p.creator_is_moderator,
        p.creator_is_admin,
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    // John is a mod, so he can_mod the bots (and his own) posts, but not tegans.
    let expected_post_listing = vec![
      ("tegan".to_owned(), false, true, false),
      ("mybot".to_owned(), true, false, true),
      ("tegan".to_owned(), false, true, false),
      ("john".to_owned(), true, false, true),
    ];
    assert_eq!(expected_post_listing, john_listings);

    // Bot is also a mod, but was added after john, so can't mod anything
    let bot_listings = PostQuery {
      sort: Some(PostSortType::New),
      local_user: Some(&data.bot_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .into_iter()
    .map(|p| {
      (
        p.creator.name,
        p.creator_is_moderator,
        p.creator_is_admin,
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    let expected_post_listing = vec![
      ("tegan".to_owned(), false, true, false),
      ("mybot".to_owned(), true, false, true),
      ("tegan".to_owned(), false, true, false),
      ("john".to_owned(), true, false, false),
    ];
    assert_eq!(expected_post_listing, bot_listings);

    // Make the bot leave the mod team, and make sure it can_mod is false.
    CommunityModerator::leave(pool, &bot_mod_form).await?;

    let bot_listings = PostQuery {
      sort: Some(PostSortType::New),
      local_user: Some(&data.bot_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .into_iter()
    .map(|p| {
      (
        p.creator.name,
        p.creator_is_moderator,
        p.creator_is_admin,
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    let expected_post_listing = vec![
      ("tegan".to_owned(), false, true, false),
      ("mybot".to_owned(), false, false, false),
      ("tegan".to_owned(), false, true, false),
      ("john".to_owned(), true, false, false),
    ];
    assert_eq!(expected_post_listing, bot_listings);

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_person_language(data: &mut Data) -> LemmyResult<()> {
    const EL_POSTO: &str = "el posto";

    let pool = &data.pool();
    let pool = &mut pool.into();

    let spanish_id = Language::read_id_from_code(pool, "es").await?;

    let french_id = Language::read_id_from_code(pool, "fr").await?;

    let post_spanish = PostInsertForm {
      language_id: Some(spanish_id),
      ..PostInsertForm::new(
        EL_POSTO.to_string(),
        data.tegan_local_user_view.person.id,
        data.community.id,
      )
    };
    Post::create(pool, &post_spanish).await?;

    let post_listings_all = data.default_post_query().list(&data.site, pool).await?;

    // no language filters specified, all posts should be returned
    assert_eq!(
      vec![EL_POSTO, POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_all)
    );

    LocalUserLanguage::update(
      pool,
      vec![french_id],
      data.tegan_local_user_view.local_user.id,
    )
    .await?;

    let post_listing_french = data.default_post_query().list(&data.site, pool).await?;

    // only one post in french and one undetermined should be returned
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listing_french)
    );
    assert_eq!(
      Some(french_id),
      post_listing_french.get(2).map(|p| p.post.language_id)
    );

    LocalUserLanguage::update(
      pool,
      vec![french_id, UNDETERMINED_ID],
      data.tegan_local_user_view.local_user.id,
    )
    .await?;
    let post_listings_french_und = data
      .default_post_query()
      .list(&data.site, pool)
      .await?
      .into_iter()
      .map(|p| (p.post.name, p.post.language_id))
      .collect::<Vec<_>>();
    let expected_post_listings_french_und = vec![
      (POST_WITH_TAGS.to_owned(), french_id),
      (POST_BY_BOT.to_owned(), UNDETERMINED_ID),
      (POST.to_owned(), french_id),
    ];

    // french post and undetermined language post should be returned
    assert_eq!(expected_post_listings_french_und, post_listings_french_und);

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_removed(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Remove the post
    Post::update(
      pool,
      data.bot_post.id,
      &PostUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    // Make sure you don't see the removed post in the results
    let post_listings_no_admin = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(vec![POST_WITH_TAGS, POST], names(&post_listings_no_admin));

    // Removed bot post is shown to admins on its profile page
    data.tegan_local_user_view.local_user.admin = true;
    let post_listings_is_admin = PostQuery {
      creator_id: Some(data.bot_local_user_view.person.id),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(vec![POST_BY_BOT], names(&post_listings_is_admin));

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_deleted(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Delete the post
    Post::update(
      pool,
      data.post.id,
      &PostUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    // Deleted post is only shown to creator
    for (local_user, expect_contains_deleted) in [
      (None, false),
      (Some(&data.john_local_user_view.local_user), false),
      (Some(&data.tegan_local_user_view.local_user), true),
    ] {
      let contains_deleted = PostQuery {
        local_user,
        ..data.default_post_query()
      }
      .list(&data.site, pool)
      .await?
      .iter()
      .any(|p| p.post.id == data.post.id);

      assert_eq!(expect_contains_deleted, contains_deleted);
    }

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_hidden_community(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    Community::update(
      pool,
      data.community.id,
      &CommunityUpdateForm {
        hidden: Some(true),
        ..Default::default()
      },
    )
    .await?;

    let posts = PostQuery::default().list(&data.site, pool).await?;
    assert!(posts.is_empty());

    let posts = data.default_post_query().list(&data.site, pool).await?;
    assert!(posts.is_empty());

    // Follow the community
    let form = CommunityFollowerForm {
      state: Some(CommunityFollowerState::Accepted),
      ..CommunityFollowerForm::new(data.community.id, data.tegan_local_user_view.person.id)
    };
    CommunityFollower::follow(pool, &form).await?;

    let posts = data.default_post_query().list(&data.site, pool).await?;
    assert!(!posts.is_empty());

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_instance_block(data: &mut Data) -> LemmyResult<()> {
    const POST_FROM_BLOCKED_INSTANCE: &str = "post on blocked instance";

    let pool = &data.pool();
    let pool = &mut pool.into();

    let blocked_instance = Instance::read_or_create(pool, "another_domain.tld".to_string()).await?;

    let community_form = CommunityInsertForm::new(
      blocked_instance.id,
      "test_community_4".to_string(),
      "none".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &community_form).await?;

    let post_form = PostInsertForm {
      language_id: Some(LanguageId(1)),
      ..PostInsertForm::new(
        POST_FROM_BLOCKED_INSTANCE.to_string(),
        data.bot_local_user_view.person.id,
        inserted_community.id,
      )
    };
    let post_from_blocked_instance = Post::create(pool, &post_form).await?;

    // no instance block, should return all posts
    let post_listings_all = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(
      vec![
        POST_FROM_BLOCKED_INSTANCE,
        POST_WITH_TAGS,
        POST_BY_BOT,
        POST
      ],
      names(&post_listings_all)
    );

    // block the instance
    let block_form = InstanceBlockForm {
      person_id: data.tegan_local_user_view.person.id,
      instance_id: blocked_instance.id,
    };
    InstanceBlock::block(pool, &block_form).await?;

    // now posts from communities on that instance should be hidden
    let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_blocked)
    );
    assert!(post_listings_blocked
      .iter()
      .all(|p| p.post.id != post_from_blocked_instance.id));

    // after unblocking it should return all posts again
    InstanceBlock::unblock(pool, &block_form).await?;
    let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(
      vec![
        POST_FROM_BLOCKED_INSTANCE,
        POST_WITH_TAGS,
        POST_BY_BOT,
        POST
      ],
      names(&post_listings_blocked)
    );

    Instance::delete(pool, blocked_instance.id).await?;
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn pagination_includes_each_post_once(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let community_form = CommunityInsertForm::new(
      data.instance.id,
      "yes".to_string(),
      "yes".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &community_form).await?;

    let mut inserted_post_ids = vec![];
    let mut inserted_comment_ids = vec![];

    // Create 150 posts with varying non-correlating values for publish date, number of comments,
    // and featured
    for comments in 0..10 {
      for _ in 0..15 {
        let post_form = PostInsertForm {
          featured_local: Some((comments % 2) == 0),
          featured_community: Some((comments % 2) == 0),
          published: Some(Utc::now() - Duration::from_secs(comments % 3)),
          ..PostInsertForm::new(
            "keep Christ in Christmas".to_owned(),
            data.tegan_local_user_view.person.id,
            inserted_community.id,
          )
        };
        let inserted_post = Post::create(pool, &post_form).await?;
        inserted_post_ids.push(inserted_post.id);

        for _ in 0..comments {
          let comment_form = CommentInsertForm::new(
            data.tegan_local_user_view.person.id,
            inserted_post.id,
            "yes".to_owned(),
          );
          let inserted_comment = Comment::create(pool, &comment_form, None).await?;
          inserted_comment_ids.push(inserted_comment.id);
        }
      }
    }

    let options = PostQuery {
      community_id: Some(inserted_community.id),
      sort: Some(PostSortType::MostComments),
      limit: Some(10),
      ..Default::default()
    };

    let mut listed_post_ids = vec![];
    let mut page_after = None;
    loop {
      let post_listings = PostQuery {
        page_after: page_after.map(|p| PaginationCursorData {
          post_aggregates: p,
          post_actions: Default::default(),
        }),
        ..options.clone()
      }
      .list(&data.site, pool)
      .await?;

      listed_post_ids.extend(post_listings.iter().map(|p| p.post.id));

      if let Some(p) = post_listings.into_iter().next_back() {
        page_after = Some(p.counts);
      } else {
        break;
      }
    }

    // Check that backward pagination matches forward pagination
    let mut listed_post_ids_forward = listed_post_ids.clone();
    let mut page_before = None;
    loop {
      let post_listings = PostQuery {
        page_after: page_before.map(|p| PaginationCursorData {
          post_aggregates: p,
          post_actions: Default::default(),
        }),
        page_back: Some(true),
        ..options.clone()
      }
      .list(&data.site, pool)
      .await?;

      let listed_post_ids = post_listings.iter().map(|p| p.post.id).collect::<Vec<_>>();

      let index = listed_post_ids_forward.len() - listed_post_ids.len();
      assert_eq!(
        listed_post_ids_forward.get(index..),
        listed_post_ids.get(..)
      );
      listed_post_ids_forward.truncate(index);

      if let Some(p) = post_listings.into_iter().next() {
        page_before = Some(p.counts);
      } else {
        break;
      }
    }

    inserted_post_ids.sort_unstable_by_key(|id| id.0);
    listed_post_ids.sort_unstable_by_key(|id| id.0);

    assert_eq!(inserted_post_ids, listed_post_ids);

    Community::delete(pool, inserted_community.id).await?;
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_hide_read(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Make sure local user hides read posts
    let local_user_form = LocalUserUpdateForm {
      show_read_posts: Some(false),
      ..Default::default()
    };
    LocalUser::update(
      pool,
      data.tegan_local_user_view.local_user.id,
      &local_user_form,
    )
    .await?;
    data.tegan_local_user_view.local_user.show_read_posts = false;

    // Mark a post as read
    let read_form = PostReadForm::new(data.bot_post.id, data.tegan_local_user_view.person.id);
    PostRead::mark_as_read(pool, &read_form).await?;

    // Make sure you don't see the read post in the results
    let post_listings_hide_read = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(vec![POST_WITH_TAGS, POST], names(&post_listings_hide_read));

    // Test with the show_read override as true
    let post_listings_show_read_true = PostQuery {
      show_read: Some(true),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_show_read_true)
    );

    // Test with the show_read override as false
    let post_listings_show_read_false = PostQuery {
      show_read: Some(false),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST],
      names(&post_listings_show_read_false)
    );
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_hide_hidden(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Mark a post as hidden
    PostHide::hide(pool, data.bot_post.id, data.tegan_local_user_view.person.id).await?;

    // Make sure you don't see the hidden post in the results
    let post_listings_hide_hidden = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST],
      names(&post_listings_hide_hidden)
    );

    // Make sure it does come back with the show_hidden option
    let post_listings_show_hidden = PostQuery {
      sort: Some(PostSortType::New),
      local_user: Some(&data.tegan_local_user_view.local_user),
      show_hidden: Some(true),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_show_hidden)
    );

    // Make sure that hidden field is true.
    assert!(&post_listings_show_hidden.get(1).is_some_and(|p| p.hidden));

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_hide_nsfw(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Mark a post as nsfw
    let update_form = PostUpdateForm {
      nsfw: Some(true),
      ..Default::default()
    };

    Post::update(pool, data.post_with_tags.id, &update_form).await?;

    // Make sure you don't see the nsfw post in the regular results
    let post_listings_hide_nsfw = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(vec![POST_BY_BOT, POST], names(&post_listings_hide_nsfw));

    // Make sure it does come back with the show_nsfw option
    let post_listings_show_nsfw = PostQuery {
      sort: Some(PostSortType::New),
      show_nsfw: Some(true),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_show_nsfw)
    );

    // Make sure that nsfw field is true.
    assert!(
      &post_listings_show_nsfw
        .first()
        .ok_or(LemmyErrorType::NotFound)?
        .post
        .nsfw
    );

    Ok(())
  }

  async fn expected_post_view(data: &Data, pool: &mut DbPool<'_>) -> LemmyResult<PostView> {
    let (inserted_person, inserted_community, inserted_post) = (
      &data.tegan_local_user_view.person,
      &data.community,
      &data.post,
    );
    let agg = PostAggregates::read(pool, inserted_post.id).await?;

    Ok(PostView {
      post: Post {
        id: inserted_post.id,
        name: inserted_post.name.clone(),
        creator_id: inserted_person.id,
        url: None,
        body: None,
        alt_text: None,
        published: inserted_post.published,
        updated: None,
        community_id: inserted_community.id,
        removed: false,
        deleted: false,
        locked: false,
        nsfw: false,
        embed_title: None,
        embed_description: None,
        embed_video_url: None,
        thumbnail_url: None,
        ap_id: inserted_post.ap_id.clone(),
        local: true,
        language_id: LanguageId(47),
        featured_community: false,
        featured_local: false,
        url_content_type: None,
        scheduled_publish_time: None,
      },
      my_vote: None,
      unread_comments: 0,
      creator: Person {
        id: inserted_person.id,
        name: inserted_person.name.clone(),
        display_name: None,
        published: inserted_person.published,
        avatar: None,
        ap_id: inserted_person.ap_id.clone(),
        local: true,
        bot_account: false,
        banned: false,
        deleted: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_person.inbox_url.clone(),
        matrix_user_id: None,
        ban_expires: None,
        instance_id: data.instance.id,
        private_key: inserted_person.private_key.clone(),
        public_key: inserted_person.public_key.clone(),
        last_refreshed_at: inserted_person.last_refreshed_at,
      },
      image_details: None,
      creator_banned_from_community: false,
      banned_from_community: false,
      creator_is_moderator: false,
      creator_is_admin: true,
      can_mod: true,
      community: Community {
        id: inserted_community.id,
        name: inserted_community.name.clone(),
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        ap_id: inserted_community.ap_id.clone(),
        local: true,
        title: "nada".to_owned(),
        sidebar: None,
        description: None,
        updated: None,
        banner: None,
        hidden: false,
        posting_restricted_to_mods: false,
        published: inserted_community.published,
        instance_id: data.instance.id,
        private_key: inserted_community.private_key.clone(),
        public_key: inserted_community.public_key.clone(),
        last_refreshed_at: inserted_community.last_refreshed_at,
        followers_url: inserted_community.followers_url.clone(),
        inbox_url: inserted_community.inbox_url.clone(),
        moderators_url: inserted_community.moderators_url.clone(),
        featured_url: inserted_community.featured_url.clone(),
        visibility: CommunityVisibility::Public,
        random_number: inserted_community.random_number,
      },
      counts: PostAggregates {
        post_id: inserted_post.id,
        comments: 0,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        published: agg.published,
        newest_comment_time_necro: inserted_post.published,
        newest_comment_time: inserted_post.published,
        featured_community: false,
        featured_local: false,
        hot_rank: RANK_DEFAULT,
        hot_rank_active: RANK_DEFAULT,
        controversy_rank: 0.0,
        scaled_rank: RANK_DEFAULT,
        community_id: inserted_post.community_id,
        creator_id: inserted_post.creator_id,
        instance_id: data.instance.id,
        report_count: 0,
        unresolved_report_count: 0,
      },
      subscribed: SubscribedType::NotSubscribed,
      read: false,
      hidden: false,
      saved: None,
      creator_blocked: false,
      tags: PostTags::default(),
    })
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn local_only_instance(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    Community::update(
      pool,
      data.community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::LocalOnly),
        ..Default::default()
      },
    )
    .await?;

    let unauthenticated_query = PostQuery {
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(0, unauthenticated_query.len());

    let authenticated_query = PostQuery {
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(3, authenticated_query.len());

    let unauthenticated_post = PostView::read(pool, data.post.id, None, false).await;
    assert!(unauthenticated_post.is_err());

    let authenticated_post = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      false,
    )
    .await;
    assert!(authenticated_post.is_ok());

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_local_user_banned_from_community(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Test that post view shows if local user is blocked from community
    let banned_from_comm_person = PersonInsertForm::test_form(data.instance.id, "jill");

    let inserted_banned_from_comm_person = Person::create(pool, &banned_from_comm_person).await?;

    let inserted_banned_from_comm_local_user = LocalUser::create(
      pool,
      &LocalUserInsertForm::test_form(inserted_banned_from_comm_person.id),
      vec![],
    )
    .await?;

    CommunityPersonBan::ban(
      pool,
      &CommunityPersonBanForm {
        community_id: data.community.id,
        person_id: inserted_banned_from_comm_person.id,
        expires: None,
      },
    )
    .await?;

    let post_view = PostView::read(
      pool,
      data.post.id,
      Some(&inserted_banned_from_comm_local_user),
      false,
    )
    .await?;

    assert!(post_view.banned_from_community);

    Person::delete(pool, inserted_banned_from_comm_person.id).await?;
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_local_user_not_banned_from_community(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let post_view = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      false,
    )
    .await?;

    assert!(!post_view.banned_from_community);

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn speed_check(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Make sure the post_view query is less than this time
    let duration_max = Duration::from_millis(80);

    // Create some dummy posts
    let num_posts = 1000;
    for x in 1..num_posts {
      let name = format!("post_{x}");
      let url = Some(Url::parse(&format!("https://google.com/{name}"))?.into());

      let post_form = PostInsertForm {
        url,
        ..PostInsertForm::new(
          name,
          data.tegan_local_user_view.person.id,
          data.community.id,
        )
      };
      Post::create(pool, &post_form).await?;
    }

    // Manually trigger and wait for a statistics update to ensure consistent and high amount of
    // accuracy in the statistics used for query planning
    println!("🧮 updating database statistics");
    let conn = &mut get_conn(pool).await?;
    conn.batch_execute("ANALYZE;").await?;

    // Time how fast the query took
    let now = Instant::now();
    PostQuery {
      sort: Some(PostSortType::Active),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    let elapsed = now.elapsed();
    println!("Elapsed: {:.0?}", elapsed);

    assert!(
      elapsed.lt(&duration_max),
      "Query took {:.0?}, longer than the max of {:.0?}",
      elapsed,
      duration_max
    );

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_no_comments_only(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Create a comment for a post
    let comment_form = CommentInsertForm::new(
      data.tegan_local_user_view.person.id,
      data.post.id,
      "a comment".to_owned(),
    );
    Comment::create(pool, &comment_form, None).await?;

    // Make sure it doesnt come back with the no_comments option
    let post_listings_no_comments = PostQuery {
      sort: Some(PostSortType::New),
      no_comments_only: Some(true),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT],
      names(&post_listings_no_comments)
    );

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_private_community(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Mark community as private
    Community::update(
      pool,
      data.community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::Private),
        ..Default::default()
      },
    )
    .await?;

    // No posts returned without auth
    let read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(0, read_post_listing.len());
    let post_view = PostView::read(pool, data.post.id, None, false).await;
    assert!(post_view.is_err());

    // No posts returned for non-follower who is not admin
    data.tegan_local_user_view.local_user.admin = false;
    let read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(0, read_post_listing.len());
    let post_view = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      false,
    )
    .await;
    assert!(post_view.is_err());

    // Admin can view content without following
    data.tegan_local_user_view.local_user.admin = true;
    let read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(3, read_post_listing.len());
    let post_view = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      true,
    )
    .await;
    assert!(post_view.is_ok());
    data.tegan_local_user_view.local_user.admin = false;

    // User can view after following
    CommunityFollower::follow(
      pool,
      &CommunityFollowerForm {
        state: Some(CommunityFollowerState::Accepted),
        ..CommunityFollowerForm::new(data.community.id, data.tegan_local_user_view.person.id)
      },
    )
    .await?;
    let read_post_listing = PostQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(3, read_post_listing.len());
    let post_view = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      true,
    )
    .await;
    assert!(post_view.is_ok());

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listings_hide_media(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Make one post an image post
    Post::update(
      pool,
      data.bot_post.id,
      &PostUpdateForm {
        url_content_type: Some(Some(String::from("image/png"))),
        ..Default::default()
      },
    )
    .await?;

    // Make sure all the posts are returned when `hide_media` is unset
    let hide_media_listing = PostQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(3, hide_media_listing.len());

    // Ensure the `hide_media` user setting is set
    let local_user_form = LocalUserUpdateForm {
      hide_media: Some(true),
      ..Default::default()
    };
    LocalUser::update(
      pool,
      data.tegan_local_user_view.local_user.id,
      &local_user_form,
    )
    .await?;
    data.tegan_local_user_view.local_user.hide_media = true;

    // Ensure you don't see the image post
    let hide_media_listing = PostQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.tegan_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(2, hide_media_listing.len());

    // Make sure the `hide_media` override works
    let hide_media_listing = PostQuery {
      community_id: Some(data.community.id),
      local_user: Some(&data.tegan_local_user_view.local_user),
      hide_media: Some(false),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(3, hide_media_listing.len());

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_tags_present(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let post_view = PostView::read(
      pool,
      data.post_with_tags.id,
      Some(&data.tegan_local_user_view.local_user),
      false,
    )
    .await?;

    assert_eq!(2, post_view.tags.tags.len());
    assert_eq!(data.tag_1.name, post_view.tags.tags[0].name);
    assert_eq!(data.tag_2.name, post_view.tags.tags[1].name);

    let all_posts = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(2, all_posts[0].tags.tags.len()); // post with tags
    assert_eq!(0, all_posts[1].tags.tags.len()); // bot post
    assert_eq!(0, all_posts[2].tags.tags.len()); // normal post

    Ok(())
  }
}
