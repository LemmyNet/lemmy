use crate::PostView;
use diesel::{
  self,
  debug_query,
  dsl::{exists, not},
  pg::Pg,
  query_builder::AsQuery,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
  TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::{asc_if, SortDirection};
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, InstanceId, PaginationCursor, PersonId, PostId},
  source::{
    community::CommunityActions,
    local_user::LocalUser,
    person::Person,
    post::{post_actions_keys as pa_key, post_keys as key, Post, PostActions},
    site::Site,
  },
  traits::{Crud, PaginationCursorBuilder},
  utils::{
    get_conn,
    limit_fetch,
    now,
    paginate,
    queries::{
      creator_community_actions_join,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
      filter_blocked,
      filter_is_subscribed,
      filter_not_unlisted_or_is_subscribed,
      image_details_join,
      my_community_actions_join,
      my_instance_actions_community_join,
      my_local_user_admin_join,
      my_person_actions_join,
      my_post_actions_join,
    },
    seconds_to_pg_interval,
    Commented,
    DbPool,
  },
};
use lemmy_db_schema_file::{
  enums::{
    CommunityFollowerState,
    CommunityVisibility,
    ListingType,
    PostSortType::{self, *},
  },
  schema::{community, community_actions, local_user_language, person, post, post_actions},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use tracing::debug;

impl PaginationCursorBuilder for PostView {
  type CursorData = Post;
  fn to_cursor(&self) -> PaginationCursor {
    PaginationCursor::new_single('P', self.post.id.0)
  }

  async fn from_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::CursorData> {
    let id = cursor.first_id()?;
    Post::read(pool, PostId(id)).await
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
    let my_instance_actions_community_join: my_instance_actions_community_join =
      my_instance_actions_community_join(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    post::table
      .inner_join(person::table)
      .inner_join(community::table)
      .left_join(image_details_join())
      .left_join(my_community_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_instance_actions_community_join)
      .left_join(my_local_user_admin_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(creator_community_actions_join())
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
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// List all the read posts for your person, ordered by the read date.
  pub async fn list_read(
    pool: &mut DbPool<'_>,
    my_person: &Person,
    cursor_data: Option<PostActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> LemmyResult<Vec<PostView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let query = PostView::joins(Some(my_person.id), my_person.instance_id)
      .filter(post_actions::person_id.eq(my_person.id))
      .filter(post_actions::read.is_not_null())
      .filter(filter_blocked())
      .select(PostView::as_select())
      .limit(limit)
      .into_boxed();

    // Sorting by the read date
    let paginated_query = paginate(query, SortDirection::Desc, cursor_data, None, page_back)
      .then_order_by(pa_key::read)
      // Tie breaker
      .then_order_by(pa_key::post_id);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// List all the hidden posts for your person, ordered by the hide date.
  pub async fn list_hidden(
    pool: &mut DbPool<'_>,
    my_person: &Person,
    cursor_data: Option<PostActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
  ) -> LemmyResult<Vec<PostView>> {
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let query = PostView::joins(Some(my_person.id), my_person.instance_id)
      .filter(post_actions::person_id.eq(my_person.id))
      .filter(post_actions::hidden.is_not_null())
      .filter(filter_blocked())
      .select(PostView::as_select())
      .limit(limit)
      .into_boxed();

    // Sorting by the hidden date
    let paginated_query = paginate(query, SortDirection::Desc, cursor_data, None, page_back)
      .then_order_by(pa_key::hidden)
      // Tie breaker
      .then_order_by(pa_key::post_id);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub fn to_post_actions_cursor(&self) -> PaginationCursor {
    // This needs a person and post
    let prefixes_and_ids = [('P', self.creator.id.0), ('O', self.post.id.0)];

    PaginationCursor::new(&prefixes_and_ids)
  }
}

#[derive(Clone, Default)]
pub struct PostQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<PostSortType>,
  pub time_range_seconds: Option<i32>,
  pub community_id: Option<CommunityId>,
  pub local_user: Option<&'a LocalUser>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  pub show_hidden: Option<bool>,
  pub show_read: Option<bool>,
  pub show_nsfw: Option<bool>,
  pub hide_media: Option<bool>,
  pub no_comments_only: Option<bool>,
  pub keyword_blocks: Option<Vec<String>>,
  pub cursor_data: Option<Post>,
  pub page_back: Option<bool>,
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

        let upper_bound_results = self
          .clone()
          .list_inner(site, None, largest_subscribed, pool)
          .await?;

        let limit = limit_fetch(self.limit)?;

        // take last element of array. if this query returned less than LIMIT elements,
        // the heuristic is invalid since we can't guarantee the full query will return >= LIMIT
        // results (return original query)
        if (upper_bound_results.len() as i64) < limit {
          None
        } else {
          if self.page_back.unwrap_or_default() {
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
  ) -> LemmyResult<Vec<PostView>> {
    let o = self;
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(o.limit)?;

    let my_person_id = o.local_user.person_id();
    let my_local_user_id = o.local_user.local_user_id();

    let mut query = PostView::joins(my_person_id, site.instance_id)
      .select(PostView::as_select())
      .limit(limit)
      .into_boxed();

    // hide posts from deleted communities
    query = query.filter(community::deleted.eq(false));

    // only creator can see deleted posts and unpublished scheduled posts
    if let Some(person_id) = my_person_id {
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

    if let Some(community_id) = o.community_id.or(largest_subscribed_for_prefetch) {
      query = query.filter(post::community_id.eq(community_id));
    }

    match o.listing_type.unwrap_or_default() {
      ListingType::Subscribed => query = query.filter(filter_is_subscribed()),
      ListingType::Local => {
        query = query
          .filter(community::local.eq(true))
          .filter(filter_not_unlisted_or_is_subscribed());
      }
      ListingType::All => query = query.filter(filter_not_unlisted_or_is_subscribed()),
      ListingType::ModeratorView => {
        query = query.filter(community_actions::became_moderator.is_not_null());
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
      query = query.filter(post::comments.eq(0));
    };

    if !o.show_read.unwrap_or(o.local_user.show_read_posts()) {
      query = query.filter(post_actions::read.is_null());
    }

    // Hide the hidden posts
    if !o.show_hidden.unwrap_or_default() {
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

    // TODO move liked only elsewhere
    if let Some(my_id) = my_person_id {
      let not_creator_filter = post::creator_id.ne(my_id);
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
      query = query.filter(post::published.gt(now() - seconds_to_pg_interval(time_range_seconds)));
    }

    // Only sort by ascending for Old
    let sort = o.sort.unwrap_or(Hot);
    let sort_direction = asc_if(sort == Old);

    let mut pq = paginate(
      query,
      sort_direction,
      o.cursor_data,
      cursor_before_data,
      o.page_back,
    );

    // featured posts first
    // Don't do for new / old sorts
    if sort != New && sort != Old {
      pq = if o.community_id.is_none() || largest_subscribed_for_prefetch.is_some() {
        pq.then_order_by(key::featured_local)
      } else {
        pq.then_order_by(key::featured_community)
      };
    }

    // then use the main sort
    pq = match sort {
      Active => pq.then_order_by(key::hot_rank_active),
      Hot => pq.then_order_by(key::hot_rank),
      Scaled => pq.then_order_by(key::scaled_rank),
      Controversial => pq.then_order_by(key::controversy_rank),
      New | Old => pq.then_order_by(key::published),
      NewComments => pq.then_order_by(key::newest_comment_time),
      MostComments => pq.then_order_by(key::comments),
      Top => pq.then_order_by(key::score),
    };

    // use publish as fallback. especially useful for hot rank which reaches zero after some days.
    // necessary because old posts can be fetched over federation and inserted with high post id
    pq = match sort {
      // A second time-based sort would not be very useful
      New | Old | NewComments => pq,
      _ => pq.then_order_by(key::published),
    };

    // finally use unique post id as tie breaker
    pq = pq.then_order_by(key::id);

    // Convert to as_query to be able to use in commented.
    let query = pq.as_query();

    debug!("Post View Query: {:?}", debug_query::<Pg, _>(&query));
    Commented::new(query)
      .text("PostQuery::list")
      .load::<PostView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn list(&self, site: &Site, pool: &mut DbPool<'_>) -> LemmyResult<Vec<PostView>> {
    let cursor_before_data = self.prefetch_cursor_before_data(site, pool).await?;

    self
      .clone()
      .list_inner(site, cursor_before_data, None, pool)
      .await
  }
}

#[allow(clippy::indexing_slicing)]
#[expect(clippy::expect_used)]
#[cfg(test)]
mod tests {
  use crate::{
    impls::{PostQuery, PostSortType},
    PostView,
  };
  use chrono::Utc;
  use diesel_async::SimpleAsyncConnection;
  use lemmy_db_schema::{
    impls::actor_language::UNDETERMINED_ID,
    newtypes::LanguageId,
    source::{
      actor_language::LocalUserLanguage,
      comment::{Comment, CommentInsertForm},
      community::{
        Community,
        CommunityActions,
        CommunityBlockForm,
        CommunityFollowerForm,
        CommunityInsertForm,
        CommunityModeratorForm,
        CommunityPersonBanForm,
        CommunityUpdateForm,
      },
      instance::{Instance, InstanceActions, InstanceBanForm, InstanceBlockForm},
      keyword_block::LocalUserKeywordBlock,
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      person::{Person, PersonActions, PersonBlockForm, PersonInsertForm},
      post::{
        Post,
        PostActions,
        PostHideForm,
        PostInsertForm,
        PostLikeForm,
        PostReadForm,
        PostUpdateForm,
      },
      post_tag::{PostTag, PostTagForm},
      site::Site,
      tag::{Tag, TagInsertForm},
    },
    traits::{Bannable, Blockable, Crud, Followable, Hideable, Joinable, Likeable, Readable},
    utils::{build_db_pool, get_conn, uplete, ActualDbPool, DbPool},
  };
  use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use std::time::{Duration, Instant};
  use test_context::{test_context, AsyncTestContext};
  use url::Url;

  const POST_BY_BLOCKED_PERSON: &str = "post by blocked person";
  const POST_BY_BOT: &str = "post by bot";
  const POST: &str = "post";
  const POST_WITH_TAGS: &str = "post with tags";
  const POST_KEYWORD_BLOCKED: &str = "blocked_keyword";

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
      let person_block = PersonBlockForm::new(inserted_tegan_person.id, inserted_john_person.id);
      PersonActions::block(pool, &person_block).await?;

      LocalUserKeywordBlock::update(
        pool,
        vec![POST_KEYWORD_BLOCKED.to_string()],
        inserted_tegan_local_user.id,
      )
      .await?;

      // Two community post tags
      let tag_1 = Tag::create(
        pool,
        &TagInsertForm {
          ap_id: Url::parse(&format!("{}/tags/test_tag1", community.ap_id))?.into(),
          display_name: "Test Tag 1".into(),
          community_id: community.id,
        },
      )
      .await?;
      let tag_2 = Tag::create(
        pool,
        &TagInsertForm {
          ap_id: Url::parse(&format!("{}/tags/test_tag2", community.ap_id))?.into(),
          display_name: "Test Tag 2".into(),
          community_id: community.id,
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
        PostTagForm {
          post_id: post_with_tags.id,
          tag_id: tag_1.id,
        },
        PostTagForm {
          post_id: post_with_tags.id,
          tag_id: tag_2.id,
        },
      ];
      PostTag::set(pool, &inserted_tags).await?;

      let tegan_local_user_view = LocalUserView {
        local_user: inserted_tegan_local_user,
        person: inserted_tegan_person,
        instance_actions: None,
      };
      let john_local_user_view = LocalUserView {
        local_user: inserted_john_local_user,
        person: inserted_john_person,
        instance_actions: None,
      };

      let bot_local_user_view = LocalUserView {
        local_user: inserted_bot_local_user,
        person: inserted_bot_person,
        instance_actions: None,
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
      data.instance.id,
      false,
    )
    .await?;

    assert_eq!(
      vec![post_listing_single_with_person.clone()],
      read_post_listing
    );
    assert_eq!(data.post.id, post_listing_single_with_person.post.id);

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
      PostView::read(pool, data.post.id, None, data.instance.id, false).await?;

    // Should be 2 posts, with the bot post, and the blocked
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST, POST_BY_BLOCKED_PERSON],
      names(&read_post_listing_multiple_no_person)
    );

    assert!(read_post_listing_multiple_no_person
      .get(2)
      .is_some_and(|x| x.post.id == data.post.id));
    assert_eq!(false, read_post_listing_single_no_person.can_mod);
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_block_community(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let community_block =
      CommunityBlockForm::new(data.community.id, data.tegan_local_user_view.person.id);
    CommunityActions::block(pool, &community_block).await?;

    let read_post_listings_with_person_after_block = PostQuery {
      community_id: Some(data.community.id),
      ..data.default_post_query()
    }
    .list(&data.site, pool)
    .await?;
    // Should be 0 posts after the community block
    assert_eq!(read_post_listings_with_person_after_block, vec![]);

    CommunityActions::unblock(pool, &community_block).await?;
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_like(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let post_like_form = PostLikeForm::new(data.post.id, data.tegan_local_user_view.person.id, 1);

    let inserted_post_like = PostActions::like(pool, &post_like_form).await?;

    assert_eq!(
      (data.post.id, data.tegan_local_user_view.person.id, Some(1)),
      (
        inserted_post_like.post_id,
        inserted_post_like.person_id,
        inserted_post_like.like_score,
      )
    );

    let post_listing_single_with_person = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      data.instance.id,
      false,
    )
    .await?;

    assert_eq!(
      (true, 1, 1, 1),
      (
        post_listing_single_with_person
          .post_actions
          .is_some_and(|t| t.like_score == Some(1)),
        post_listing_single_with_person.post.score,
        post_listing_single_with_person.post.upvotes,
        post_listing_single_with_person.creator.post_score,
      )
    );

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
    assert_eq!(
      post_listing_single_with_person.post.id,
      read_post_listing[0].post.id
    );

    let like_removed =
      PostActions::remove_like(pool, data.tegan_local_user_view.person.id, data.post.id).await?;
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
    PostActions::like(pool, &post_like_form).await?;

    let bot_post_like_form =
      PostLikeForm::new(data.bot_post.id, data.tegan_local_user_view.person.id, 1);
    PostActions::like(pool, &bot_post_like_form).await?;

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

    // Mark the bot post, then the tags post as read
    let bot_post_read_form =
      PostReadForm::new(data.bot_post.id, data.tegan_local_user_view.person.id);
    PostActions::mark_as_read(pool, &bot_post_read_form).await?;

    let tag_post_read_form =
      PostReadForm::new(data.post_with_tags.id, data.tegan_local_user_view.person.id);
    PostActions::mark_as_read(pool, &tag_post_read_form).await?;

    let read_read_post_listing =
      PostView::list_read(pool, &data.tegan_local_user_view.person, None, None, None).await?;

    // This should be ordered from most recently read
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT],
      names(&read_read_post_listing)
    );

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
        p.creator_community_actions
          .map(|x| x.became_moderator.is_some())
          .unwrap_or(false),
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    // Tegan is an admin, so can_mod should be always true
    let expected_post_listing = vec![
      ("tegan".to_owned(), false, true),
      ("mybot".to_owned(), false, true),
      ("tegan".to_owned(), false, true),
    ];
    assert_eq!(expected_post_listing, tegan_listings);

    // Have john become a moderator, then the bot
    let john_mod_form =
      CommunityModeratorForm::new(community_id, data.john_local_user_view.person.id);
    CommunityActions::join(pool, &john_mod_form).await?;

    let bot_mod_form =
      CommunityModeratorForm::new(community_id, data.bot_local_user_view.person.id);
    CommunityActions::join(pool, &bot_mod_form).await?;

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
        p.creator_community_actions
          .map(|x| x.became_moderator.is_some())
          .unwrap_or(false),
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    // John is a mod, so he can_mod the bots (and his own) posts, but not tegans.
    let expected_post_listing = vec![
      ("tegan".to_owned(), false, false),
      ("mybot".to_owned(), true, true),
      ("tegan".to_owned(), false, false),
      ("john".to_owned(), true, true),
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
        p.creator_community_actions
          .map(|x| x.became_moderator.is_some())
          .unwrap_or(false),
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    let expected_post_listing = vec![
      ("tegan".to_owned(), false, false),
      ("mybot".to_owned(), true, true),
      ("tegan".to_owned(), false, false),
      ("john".to_owned(), true, false),
    ];
    assert_eq!(expected_post_listing, bot_listings);

    // Make the bot leave the mod team, and make sure it can_mod is false.
    CommunityActions::leave(pool, &bot_mod_form).await?;

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
        p.creator_community_actions
          .map(|x| x.became_moderator.is_some())
          .unwrap_or(false),
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    let expected_post_listing = vec![
      ("tegan".to_owned(), false, false),
      ("mybot".to_owned(), false, false),
      ("tegan".to_owned(), false, false),
      ("john".to_owned(), true, false),
    ];
    assert_eq!(expected_post_listing, bot_listings);

    // Have tegan the administrator become a moderator
    let tegan_mod_form =
      CommunityModeratorForm::new(community_id, data.tegan_local_user_view.person.id);
    CommunityActions::join(pool, &tegan_mod_form).await?;

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
        p.creator_community_actions
          .map(|x| x.became_moderator.is_some())
          .unwrap_or(false),
        p.can_mod,
      )
    })
    .collect::<Vec<_>>();

    // John is a mod, so he still can_mod the bots (and his own) posts. Tegan is a lower mod and
    // admin, john can't mod their posts.
    let expected_post_listing = vec![
      ("tegan".to_owned(), true, false),
      ("mybot".to_owned(), false, true),
      ("tegan".to_owned(), true, false),
      ("john".to_owned(), true, true),
    ];
    assert_eq!(expected_post_listing, john_listings);

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
    assert_eq!(vec![POST_WITH_TAGS, POST], names(&post_listing_french));
    assert_eq!(
      Some(french_id),
      post_listing_french.get(1).map(|p| p.post.language_id)
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
    data.tegan_local_user_view.local_user.admin = false;
    let post_listings_no_admin = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(vec![POST_WITH_TAGS, POST], names(&post_listings_no_admin));

    // Removed bot post is shown to admins
    data.tegan_local_user_view.local_user.admin = true;
    let post_listings_is_admin = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_is_admin)
    );

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
        visibility: Some(CommunityVisibility::Unlisted),
        ..Default::default()
      },
    )
    .await?;

    let posts = PostQuery::default().list(&data.site, pool).await?;
    assert!(posts.is_empty());

    let posts = data.default_post_query().list(&data.site, pool).await?;
    assert!(posts.is_empty());

    // Follow the community
    let form = CommunityFollowerForm::new(
      data.community.id,
      data.tegan_local_user_view.person.id,
      CommunityFollowerState::Accepted,
    );
    CommunityActions::follow(pool, &form).await?;

    let posts = data.default_post_query().list(&data.site, pool).await?;
    assert!(!posts.is_empty());

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_instance_block(data: &mut Data) -> LemmyResult<()> {
    const POST_FROM_BLOCKED_INSTANCE: &str = "post on blocked instance";
    const POST_LISTING_WITH_BLOCKED: [&str; 4] = [
      POST_FROM_BLOCKED_INSTANCE,
      POST_WITH_TAGS,
      POST_BY_BOT,
      POST,
    ];

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
    assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_all));

    // block the instance
    let block_form =
      InstanceBlockForm::new(data.tegan_local_user_view.person.id, blocked_instance.id);
    InstanceActions::block(pool, &block_form).await?;

    // now posts from communities on that instance should be hidden
    let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(
      vec![POST_WITH_TAGS, POST_BY_BOT, POST],
      names(&post_listings_blocked)
    );
    assert!(post_listings_blocked
      .iter()
      .all(|p| p.post.id != post_from_blocked_instance.id));

    // Follow community from the blocked instance to see posts anyway
    let follow_form = CommunityFollowerForm::new(
      inserted_community.id,
      data.tegan_local_user_view.person.id,
      CommunityFollowerState::Accepted,
    );
    CommunityActions::follow(pool, &follow_form).await?;
    let post_listings_bypass = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_bypass));
    CommunityActions::unfollow(
      pool,
      data.tegan_local_user_view.person.id,
      inserted_community.id,
    )
    .await?;

    // after unblocking it should return all posts again
    InstanceActions::unblock(pool, &block_form).await?;
    let post_listings_blocked = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(POST_LISTING_WITH_BLOCKED, *names(&post_listings_blocked));

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
    let mut cursor_data = None;
    loop {
      let post_listings = PostQuery {
        cursor_data,
        ..options.clone()
      }
      .list(&data.site, pool)
      .await?;

      listed_post_ids.extend(post_listings.iter().map(|p| p.post.id));

      if let Some(p) = post_listings.into_iter().next_back() {
        cursor_data = Some(p.post);
      } else {
        break;
      }
    }

    // Check that backward pagination matches forward pagination
    let mut listed_post_ids_forward = listed_post_ids.clone();
    let mut cursor_data_before = None;
    loop {
      let post_listings = PostQuery {
        cursor_data: cursor_data_before,
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
        cursor_data_before = Some(p.post);
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
    PostActions::mark_as_read(pool, &read_form).await?;

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
    let hide_form = PostHideForm::new(data.bot_post.id, data.tegan_local_user_view.person.id);
    PostActions::hide(pool, &hide_form).await?;

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
    assert!(&post_listings_show_hidden
      .get(1)
      .is_some_and(|p| p.post_actions.as_ref().is_some_and(|a| a.hidden.is_some())));

    // Make sure only that one comes back for list_hidden
    let list_hidden =
      PostView::list_hidden(pool, &data.tegan_local_user_view.person, None, None, None).await?;
    assert_eq!(vec![POST_BY_BOT], names(&list_hidden));

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
        visibility: Some(CommunityVisibility::LocalOnlyPrivate),
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

    let unauthenticated_post =
      PostView::read(pool, data.post.id, None, data.instance.id, false).await;
    assert!(unauthenticated_post.is_err());

    let authenticated_post = PostView::read(
      pool,
      data.post.id,
      Some(&data.tegan_local_user_view.local_user),
      data.instance.id,
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

    CommunityActions::ban(
      pool,
      &CommunityPersonBanForm::new(data.community.id, inserted_banned_from_comm_person.id),
    )
    .await?;

    let post_view = PostView::read(
      pool,
      data.post.id,
      Some(&inserted_banned_from_comm_local_user),
      data.instance.id,
      false,
    )
    .await?;

    assert!(post_view
      .community_actions
      .is_some_and(|x| x.received_ban.is_some()));

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
      data.instance.id,
      false,
    )
    .await?;

    assert!(post_view.community_actions.is_none());

    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn post_listing_local_user_banned(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let banned_person_form = PersonInsertForm::test_form(data.instance.id, "jill");

    let banned_person = Person::create(pool, &banned_person_form).await?;

    let post_form = PostInsertForm {
      language_id: Some(LanguageId(1)),
      ..PostInsertForm::new(
        "banned person post".to_string(),
        banned_person.id,
        data.community.id,
      )
    };
    let banned_post = Post::create(pool, &post_form).await?;

    InstanceActions::ban(
      pool,
      &InstanceBanForm::new(banned_person.id, data.instance.id, None),
    )
    .await?;

    // Let john read their post
    let post_view = PostView::read(
      pool,
      banned_post.id,
      Some(&data.john_local_user_view.local_user),
      data.instance.id,
      false,
    )
    .await?;

    assert!(post_view
      .creator_local_instance_actions
      .is_some_and(|x| x.received_ban.is_some()));

    assert!(post_view
      .creator_home_instance_actions
      .is_some_and(|x| x.received_ban.is_some()));

    // This should be none, since john wasn't banned, only the creator.
    assert!(post_view.instance_actions.is_none());

    assert!(post_view.creator_banned);

    Person::delete(pool, banned_person.id).await?;
    Ok(())
  }

  #[test_context(Data)]
  #[tokio::test]
  #[serial]
  async fn speed_check(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    // Make sure the post_view query is less than this time
    let duration_max = Duration::from_millis(120);

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
    let post_view = PostView::read(pool, data.post.id, None, data.instance.id, false).await;
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
      data.instance.id,
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
      data.instance.id,
      true,
    )
    .await;
    assert!(post_view.is_ok());
    data.tegan_local_user_view.local_user.admin = false;

    // User can view after following
    let follow_form = CommunityFollowerForm::new(
      data.community.id,
      data.tegan_local_user_view.person.id,
      CommunityFollowerState::Accepted,
    );
    CommunityActions::follow(pool, &follow_form).await?;

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
      data.instance.id,
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
  async fn post_with_blocked_keywords(data: &mut Data) -> LemmyResult<()> {
    let pool = &data.pool();
    let pool = &mut pool.into();

    let name_blocked = format!("post_{POST_KEYWORD_BLOCKED}");
    let name_blocked2 = format!("post2_{POST_KEYWORD_BLOCKED}2");
    let url = Some(Url::parse(&format!("https://google.com/{POST_KEYWORD_BLOCKED}"))?.into());
    let body = format!("post body with {POST_KEYWORD_BLOCKED}");
    let name_not_blocked = "post_with_name_not_blocked".to_string();
    let name_not_blocked2 = "post_with_name_not_blocked2".to_string();

    let post_name_blocked = PostInsertForm::new(
      name_blocked.clone(),
      data.tegan_local_user_view.person.id,
      data.community.id,
    );

    let post_body_blocked = PostInsertForm {
      body: Some(body),
      ..PostInsertForm::new(
        name_not_blocked.clone(),
        data.tegan_local_user_view.person.id,
        data.community.id,
      )
    };

    let post_url_blocked = PostInsertForm {
      url,
      ..PostInsertForm::new(
        name_not_blocked2.clone(),
        data.tegan_local_user_view.person.id,
        data.community.id,
      )
    };

    let post_name_blocked_but_not_body_and_url = PostInsertForm {
      body: Some("Some body".to_string()),
      url: Some(Url::parse("https://google.com")?.into()),
      ..PostInsertForm::new(
        name_blocked2.clone(),
        data.tegan_local_user_view.person.id,
        data.community.id,
      )
    };
    Post::create(pool, &post_name_blocked).await?;
    Post::create(pool, &post_body_blocked).await?;
    Post::create(pool, &post_url_blocked).await?;
    Post::create(pool, &post_name_blocked_but_not_body_and_url).await?;

    let keyword_blocks =
      Some(LocalUserKeywordBlock::read(pool, data.tegan_local_user_view.local_user.id).await?);

    let post_listings = PostQuery {
      local_user: Some(&data.tegan_local_user_view.local_user),
      keyword_blocks,
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Should not have any of the posts
    assert!(!names(&post_listings).contains(&name_blocked.as_str()));
    assert!(!names(&post_listings).contains(&name_blocked2.as_str()));
    assert!(!names(&post_listings).contains(&name_not_blocked.as_str()));
    assert!(!names(&post_listings).contains(&name_not_blocked2.as_str()));

    // Should contain not blocked posts
    assert!(names(&post_listings).contains(&POST_BY_BOT));
    assert!(names(&post_listings).contains(&POST));
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
      data.instance.id,
      false,
    )
    .await?;

    assert_eq!(2, post_view.tags.0.len());
    assert_eq!(data.tag_1.display_name, post_view.tags.0[0].display_name);
    assert_eq!(data.tag_2.display_name, post_view.tags.0[1].display_name);

    let all_posts = data.default_post_query().list(&data.site, pool).await?;
    assert_eq!(2, all_posts[0].tags.0.len()); // post with tags
    assert_eq!(0, all_posts[1].tags.0.len()); // bot post
    assert_eq!(0, all_posts[2].tags.0.len()); // normal post

    Ok(())
  }
}
