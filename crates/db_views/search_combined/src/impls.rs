use crate::{
  CommentView,
  CommunityView,
  LocalUserView,
  PersonView,
  PostView,
  SearchCombinedView,
  SearchCombinedViewInternal,
};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
  dsl::not,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::asc_if;
use lemmy_db_schema::{
  SearchSortType::{self, *},
  SearchType,
  impls::local_user::LocalUserOptionHelper,
  newtypes::CommunityId,
  source::{
    combined::search::{SearchCombined, search_combined_keys as key},
    site::Site,
  },
  traits::InternalToCombinedView,
  utils::{
    limit_fetch,
    queries::filters::{
      filter_is_subscribed,
      filter_not_unlisted_or_is_subscribed,
      filter_suggested_communities,
    },
  },
};
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  enums::{CommunityFollowerState, CommunityVisibility, ListingType},
  joins::{
    creator_community_actions_join,
    creator_home_instance_actions_join,
    creator_local_instance_actions_join,
    creator_local_user_admin_join,
    image_details_join,
    my_comment_actions_join,
    my_community_actions_join,
    my_local_user_admin_join,
    my_person_actions_join,
    my_post_actions_join,
  },
  schema::{
    comment,
    comment_actions,
    community,
    community_actions,
    multi_community,
    person,
    post,
    post_actions,
    search_combined,
  },
};
use lemmy_db_views_community::MultiCommunityView;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
  utils::{fuzzy_search, now, seconds_to_pg_interval},
};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::validation::clean_url,
};
use url::Url;

impl SearchCombinedViewInternal {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>, local_instance_id: InstanceId) -> _ {
    let item_creator = person::id;

    let item_creator_join = person::table.on(
      search_combined::person_id
        .eq(item_creator.nullable())
        .or(
          search_combined::comment_id
            .is_not_null()
            .and(comment::creator_id.eq(item_creator)),
        )
        .or(
          search_combined::post_id
            .is_not_null()
            .and(post::creator_id.eq(item_creator)),
        )
        .or(
          search_combined::multi_community_id
            .is_not_null()
            .and(multi_community::creator_id.eq(item_creator)),
        )
        .and(not(person::deleted)),
    );

    let comment_join = comment::table.on(
      search_combined::comment_id
        .eq(comment::id.nullable())
        .and(not(comment::removed))
        .and(not(comment::deleted)),
    );

    let post_join = post::table.on(
      search_combined::post_id
        .eq(post::id.nullable())
        .or(comment::post_id.eq(post::id))
        .and(not(post::removed))
        .and(not(post::deleted)),
    );

    let community_join = community::table.on(
      search_combined::community_id
        .eq(community::id.nullable())
        .or(post::community_id.eq(community::id))
        .and(not(community::removed))
        .and(not(community::local_removed))
        .and(not(community::deleted)),
    );

    let multi_community_join = multi_community::table.on(
      search_combined::multi_community_id
        .eq(multi_community::id.nullable())
        .and(not(multi_community::deleted)),
    );

    let my_community_actions_join: my_community_actions_join =
      my_community_actions_join(my_person_id);
    let my_post_actions_join: my_post_actions_join = my_post_actions_join(my_person_id);
    let my_comment_actions_join: my_comment_actions_join = my_comment_actions_join(my_person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(my_person_id);
    let my_person_actions_join: my_person_actions_join = my_person_actions_join(my_person_id);
    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    search_combined::table
      .left_join(comment_join)
      .left_join(post_join)
      .left_join(multi_community_join)
      .left_join(item_creator_join)
      .left_join(community_join)
      .left_join(image_details_join())
      .left_join(creator_community_actions_join())
      .left_join(creator_local_user_admin_join())
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .left_join(my_local_user_admin_join)
      .left_join(my_community_actions_join)
      .left_join(my_post_actions_join)
      .left_join(my_person_actions_join)
      .left_join(my_comment_actions_join)
  }
}

impl SearchCombinedView {
  /// Useful in combination with filter_map
  pub fn to_post_view(&self) -> Option<&PostView> {
    if let Self::Post(v) = self {
      Some(v)
    } else {
      None
    }
  }
}

impl PaginationCursorConversion for SearchCombinedView {
  type PaginatedType = SearchCombined;

  fn to_cursor(&self) -> CursorData {
    let (prefix, id) = match &self {
      SearchCombinedView::Post(v) => ('P', v.post.id.0),
      SearchCombinedView::Comment(v) => ('C', v.comment.id.0),
      SearchCombinedView::Community(v) => ('O', v.community.id.0),
      SearchCombinedView::Person(v) => ('E', v.person.id.0),
      SearchCombinedView::MultiCommunity(v) => ('M', v.multi.id.0),
    };
    CursorData::new_with_prefix(prefix, id)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let conn = &mut get_conn(pool).await?;
    let (prefix, id) = cursor.id_and_prefix()?;

    let mut query = search_combined::table
      .select(Self::PaginatedType::as_select())
      .into_boxed();

    query = match prefix {
      'P' => query.filter(search_combined::post_id.eq(id)),
      'C' => query.filter(search_combined::comment_id.eq(id)),
      'O' => query.filter(search_combined::community_id.eq(id)),
      'E' => query.filter(search_combined::person_id.eq(id)),
      'M' => query.filter(search_combined::multi_community_id.eq(id)),
      _ => return Err(LemmyErrorType::CouldntParsePaginationToken.into()),
    };
    let token = query.first(conn).await?;

    Ok(token)
  }
}

#[derive(Default)]
pub struct SearchCombinedQuery {
  pub search_term: Option<String>,
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub type_: Option<SearchType>,
  pub sort: Option<SearchSortType>,
  pub time_range_seconds: Option<i32>,
  pub listing_type: Option<ListingType>,
  pub title_only: Option<bool>,
  pub post_url_only: Option<bool>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  pub show_nsfw: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

impl SearchCombinedQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &Option<LocalUserView>,
    site_local: &Site,
  ) -> LemmyResult<PagedResponse<SearchCombinedView>> {
    let my_local_user = user.as_ref().map(|u| &u.local_user);
    let my_person_id = my_local_user.person_id();
    let item_creator = person::id;

    let limit = limit_fetch(self.limit, None)?;

    let mut query = SearchCombinedViewInternal::joins(my_person_id, site_local.instance_id)
      .select(SearchCombinedViewInternal::as_select())
      .limit(limit)
      .into_boxed();

    // The filters

    // Some helpers
    let is_post = search_combined::post_id.is_not_null();
    let is_comment = search_combined::comment_id.is_not_null();
    let is_community = search_combined::community_id.is_not_null();
    let is_person = search_combined::person_id.is_not_null();
    let is_multi_community = search_combined::multi_community_id.is_not_null();

    // The search term
    if let Some(search_term) = self.search_term {
      if self.post_url_only.unwrap_or_default() {
        // Parse and normalize the url, removing tracking parameters (same logic which is used
        // when creating a new post).
        let normalized_url = Url::parse(&search_term).map(|u| clean_url(&u).to_string());
        // If any of the normalization steps above failed, use the search term directly
        // (this can happen when searching part of an url).
        let url_searcher = fuzzy_search(&normalized_url.unwrap_or(search_term));
        query = query.filter(is_post.and(post::url.ilike(url_searcher)));
      } else {
        let searcher = fuzzy_search(&search_term);

        // These need to also filter by the type, otherwise they may return children
        let name_or_title_filter = is_post
          .and(post::name.ilike(searcher.clone()))
          .or(is_comment.and(comment::content.ilike(searcher.clone())))
          .or(is_community.and(community::name.ilike(searcher.clone())))
          .or(is_community.and(community::title.ilike(searcher.clone())))
          .or(is_person.and(person::name.ilike(searcher.clone())))
          .or(is_person.and(person::display_name.ilike(searcher.clone())))
          .or(is_multi_community.and(multi_community::title.ilike(searcher.clone())))
          .or(is_multi_community.and(multi_community::name.ilike(searcher.clone())));

        query = if self.title_only.unwrap_or_default() {
          query.filter(name_or_title_filter)
        } else {
          let body_or_description_filter = is_post
            .and(post::body.ilike(searcher.clone()))
            .or(is_community.and(community::summary.ilike(searcher.clone())))
            .or(is_multi_community.and(multi_community::summary.ilike(searcher.clone())))
            .or(is_person.and(person::bio.ilike(searcher.clone())));
          query.filter(name_or_title_filter.or(body_or_description_filter))
        }
      }
    }

    // Community id
    if let Some(community_id) = self.community_id {
      query = query.filter(community::id.eq(community_id));
    }

    // Creator id
    if let Some(creator_id) = self.creator_id {
      query = query.filter(item_creator.eq(creator_id));
    }

    // Liked / disliked filter
    if let Some(my_id) = my_person_id {
      let not_creator_filter = item_creator.ne(my_id);
      let liked_disliked_filter = |should_be_upvote: bool| {
        is_post
          .and(post_actions::vote_is_upvote.eq(should_be_upvote))
          .or(is_comment.and(comment_actions::vote_is_upvote.eq(should_be_upvote)))
      };

      if self.liked_only.unwrap_or_default() {
        query = query
          .filter(not_creator_filter)
          .filter(liked_disliked_filter(true));
      } else if self.disliked_only.unwrap_or_default() {
        query = query
          .filter(not_creator_filter)
          .filter(liked_disliked_filter(false));
      }
    };

    // Type
    query = match self.type_.unwrap_or_default() {
      SearchType::All => query,
      SearchType::Posts => query.filter(is_post),
      SearchType::Comments => query.filter(is_comment),
      SearchType::Communities => query.filter(is_community),
      SearchType::Users => query.filter(is_person),
      SearchType::MultiCommunities => query.filter(is_multi_community),
    };

    // Listing type
    query = match self.listing_type.unwrap_or_default() {
      ListingType::Subscribed => query.filter(filter_is_subscribed()),
      ListingType::Local => query.filter(
        community::local
          .eq(true)
          .and(filter_not_unlisted_or_is_subscribed())
          .or(is_person.and(person::local))
          .or(multi_community::local),
      ),
      ListingType::All => query.filter(
        filter_not_unlisted_or_is_subscribed()
          .or(is_person)
          .or(is_multi_community),
      ),
      ListingType::ModeratorView => {
        query.filter(community_actions::became_moderator_at.is_not_null())
      }
      ListingType::Suggested => query.filter(filter_suggested_communities()),
    };

    // Filter by the time range
    if let Some(time_range_seconds) = self.time_range_seconds {
      query = query.filter(
        search_combined::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)),
      );
    }

    // NSFW
    let user_and_site_nsfw = my_local_user.show_nsfw(site_local);
    if !self.show_nsfw.unwrap_or(user_and_site_nsfw) {
      let safe_community = community::nsfw.eq(false);
      let safe_post_and_community = post::nsfw.eq(false).and(safe_community);

      query = query.filter(
        is_community
          .and(safe_community)
          .or(is_post.and(safe_post_and_community))
          .or(is_comment.and(safe_post_and_community))
          .or(is_person)
          .or(is_multi_community),
      );
    };

    // Check permissions to view private community content.
    // Specifically, if the community is private then only accepted followers may view its
    // content, otherwise it is filtered out. Admins can view private community content
    // without restriction.
    if !my_local_user.is_admin() {
      let view_private_community = community::visibility
        .ne(CommunityVisibility::Private)
        .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted));

      // Only filter for communities, posts, and comments
      query = query.filter(
        is_community
          .and(view_private_community.clone())
          .or(is_post.and(view_private_community.clone()))
          .or(is_comment.and(view_private_community.clone()))
          .or(is_person)
          .or(is_multi_community),
      );
    };

    // Only sort by asc if old
    let sort = self.sort.unwrap_or_default();
    let sort_direction = asc_if(sort == Old);

    let mut paginated_query =
      SearchCombinedView::paginate(query, &self.page_cursor, sort_direction, pool, None).await?;

    paginated_query = match sort {
      New | Old => paginated_query.then_order_by(key::published_at),
      Top => paginated_query.then_order_by(key::score),
    }
    // finally use unique id as tie breaker
    .then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<SearchCombinedViewInternal>(conn)
      .await?;

    // Map the query results to the enum
    let out = res
      .into_iter()
      .filter_map(InternalToCombinedView::map_to_enum)
      .collect();

    paginate_response(out, limit, self.page_cursor)
  }
}

impl InternalToCombinedView for SearchCombinedViewInternal {
  type CombinedView = SearchCombinedView;

  fn map_to_enum(self) -> Option<Self::CombinedView> {
    // Use for a short alias
    let v = self;

    if let (Some(comment), Some(creator), Some(post), Some(community)) = (
      v.comment,
      v.item_creator.clone(),
      v.post.clone(),
      v.community.clone(),
    ) {
      Some(SearchCombinedView::Comment(CommentView {
        comment,
        post,
        community,
        creator,
        community_actions: v.community_actions,
        person_actions: v.person_actions,
        comment_actions: v.comment_actions,
        creator_is_admin: v.item_creator_is_admin,
        tags: v.tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_ban_expires_at: v.creator_ban_expires_at,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_community: v.creator_banned_from_community,
        creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      }))
    } else if let (Some(post), Some(creator), Some(community)) =
      (v.post, v.item_creator.clone(), v.community.clone())
    {
      Some(SearchCombinedView::Post(PostView {
        post,
        community,
        creator,
        creator_is_admin: v.item_creator_is_admin,
        image_details: v.image_details,
        community_actions: v.community_actions,
        person_actions: v.person_actions,
        post_actions: v.post_actions,
        tags: v.tags,
        can_mod: v.can_mod,
        creator_banned: v.creator_banned,
        creator_ban_expires_at: v.creator_ban_expires_at,
        creator_is_moderator: v.creator_is_moderator,
        creator_banned_from_community: v.creator_banned_from_community,
        creator_community_ban_expires_at: v.creator_community_ban_expires_at,
      }))
    } else if let Some(community) = v.community {
      Some(SearchCombinedView::Community(CommunityView {
        community,
        community_actions: v.community_actions,
        can_mod: v.can_mod,
        tags: v.tags,
      }))
    } else if let (Some(multi), Some(creator)) = (v.multi_community, &v.item_creator) {
      Some(SearchCombinedView::MultiCommunity(MultiCommunityView {
        multi,
        owner: creator.clone(),
        follow_state: None,
      }))
    } else if let Some(person) = v.item_creator {
      Some(SearchCombinedView::Person(PersonView {
        person,
        is_admin: v.item_creator_is_admin,
        person_actions: v.person_actions,
        banned: v.creator_banned,
        ban_expires_at: v.creator_ban_expires_at,
      }))
    } else {
      None
    }
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
  use crate::{LocalUserView, SearchCombinedView, impls::SearchCombinedQuery};
  use lemmy_db_schema::{
    SearchSortType,
    SearchType,
    assert_length,
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm, CommentUpdateForm},
      community::{Community, CommunityActions, CommunityFollowerForm, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      multi_community::{MultiCommunity, MultiCommunityInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm, PostUpdateForm},
      site::{Site, SiteInsertForm},
    },
    traits::{Followable, Likeable},
  };
  use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;

  struct Data {
    instance: Instance,
    site: Site,
    timmy: Person,
    timmy_view: LocalUserView,
    sara: Person,
    community: Community,
    community_2: Community,
    private_community: Community,
    timmy_post: Post,
    timmy_post_2: Post,
    sara_post: Post,
    nsfw_post: Post,
    timmy_post_private_comm: Post,
    timmy_comment: Comment,
    sara_comment: Comment,
    sara_comment_2: Comment,
    comment_in_nsfw_post: Comment,
    timmy_comment_private_comm: Comment,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;
    let site_form = SiteInsertForm::new("test_site".to_string(), instance.id);
    let site = Site::create(pool, &site_form).await?;

    let sara_form = PersonInsertForm::test_form(instance.id, "sara_pcv");
    let sara = Person::create(pool, &sara_form).await?;

    let timmy_form = PersonInsertForm::test_form(instance.id, "timmy_pcv");
    let timmy = Person::create(pool, &timmy_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form(timmy.id);
    let timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;
    let timmy_view = LocalUserView {
      local_user: timmy_local_user,
      person: timmy.clone(),
      banned: false,
      ban_expires_at: None,
    };

    let community_form = CommunityInsertForm {
      summary: Some("ask lemmy things".into()),
      ..CommunityInsertForm::new(
        instance.id,
        "asklemmy".to_string(),
        "Ask Lemmy".to_owned(),
        "pubkey".to_string(),
      )
    };
    let community = Community::create(pool, &community_form).await?;

    let community_form_2 = CommunityInsertForm::new(
      instance.id,
      "startrek_ds9".to_string(),
      "Star Trek - Deep Space Nine".to_owned(),
      "pubkey".to_string(),
    );
    let community_2 = Community::create(pool, &community_form_2).await?;

    let private_community_form = CommunityInsertForm {
      visibility: Some(CommunityVisibility::Private),
      ..CommunityInsertForm::new(
        instance.id,
        "private_comm".to_string(),
        "This is a private comm".to_owned(),
        "pubkey".to_string(),
      )
    };
    let private_community = Community::create(pool, &private_community_form).await?;

    let timmy_post_form = PostInsertForm {
      body: Some("postbody inside here".into()),
      url: Some(Url::parse("https://google.com")?.into()),
      ..PostInsertForm::new("timmy post prv".into(), timmy.id, community.id)
    };
    let timmy_post = Post::create(pool, &timmy_post_form).await?;

    let timmy_post_form_2 = PostInsertForm::new("timmy post prv 2".into(), timmy.id, community.id);
    let timmy_post_2 = Post::create(pool, &timmy_post_form_2).await?;

    let sara_post_form = PostInsertForm::new("sara post prv".into(), sara.id, community_2.id);
    let sara_post = Post::create(pool, &sara_post_form).await?;

    let nsfw_post_form = PostInsertForm {
      body: Some("nsfw post inside here".into()),
      url: Some(Url::parse("https://google.com")?.into()),
      nsfw: Some(true),
      ..PostInsertForm::new("nsfw post prv".into(), timmy.id, community.id)
    };
    let nsfw_post = Post::create(pool, &nsfw_post_form).await?;

    let timmy_post_private_comm_form = PostInsertForm::new(
      "timmy post private comm".into(),
      timmy.id,
      private_community.id,
    );
    let timmy_post_private_comm = Post::create(pool, &timmy_post_private_comm_form).await?;

    let timmy_comment_form =
      CommentInsertForm::new(timmy.id, timmy_post.id, "timmy comment prv gold".into());
    let timmy_comment = Comment::create(pool, &timmy_comment_form, None).await?;

    let sara_comment_form =
      CommentInsertForm::new(sara.id, sara_post.id, "sara comment prv gold".into());
    let sara_comment = Comment::create(pool, &sara_comment_form, None).await?;

    let sara_comment_form_2 =
      CommentInsertForm::new(sara.id, timmy_post_2.id, "sara comment prv 2".into());
    let sara_comment_2 = Comment::create(pool, &sara_comment_form_2, None).await?;

    let comment_in_nsfw_post_form = CommentInsertForm::new(
      sara.id,
      nsfw_post.id,
      "sara comment in nsfw post prv 2".into(),
    );
    let comment_in_nsfw_post = Comment::create(pool, &comment_in_nsfw_post_form, None).await?;

    let timmy_comment_private_comm_form = CommentInsertForm::new(
      timmy.id,
      timmy_post_private_comm.id,
      "timmy comment private comm".into(),
    );
    let timmy_comment_private_comm =
      Comment::create(pool, &timmy_comment_private_comm_form, None).await?;

    // Timmy likes and dislikes a few things
    let timmy_like_post_form = PostLikeForm::new(timmy_post.id, timmy.id, Some(true));
    PostActions::like(pool, &timmy_like_post_form).await?;

    let timmy_like_sara_post_form = PostLikeForm::new(sara_post.id, timmy.id, Some(true));
    PostActions::like(pool, &timmy_like_sara_post_form).await?;

    let timmy_dislike_post_form = PostLikeForm::new(timmy_post_2.id, timmy.id, Some(false));
    PostActions::like(pool, &timmy_dislike_post_form).await?;

    let timmy_like_comment_form = CommentLikeForm::new(timmy_comment.id, timmy.id, Some(true));
    CommentActions::like(pool, &timmy_like_comment_form).await?;

    let timmy_like_sara_comment_form = CommentLikeForm::new(sara_comment.id, timmy.id, Some(true));
    CommentActions::like(pool, &timmy_like_sara_comment_form).await?;

    let timmy_dislike_sara_comment_form =
      CommentLikeForm::new(sara_comment_2.id, timmy.id, Some(false));
    CommentActions::like(pool, &timmy_dislike_sara_comment_form).await?;

    Ok(Data {
      instance,
      site,
      timmy,
      timmy_view,
      sara,
      community,
      community_2,
      private_community,
      timmy_post,
      timmy_post_2,
      sara_post,
      nsfw_post,
      timmy_post_private_comm,
      timmy_comment,
      sara_comment,
      sara_comment_2,
      comment_in_nsfw_post,
      timmy_comment_private_comm,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn combined() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // search
    let search = SearchCombinedQuery::default()
      .list(pool, &None, &data.site)
      .await?;
    assert_length!(10, search);

    // Make sure the types are correct
    if let SearchCombinedView::Comment(v) = &search[0] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Comment(v) = &search[1] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara_post.id, v.post.id);
      assert_eq!(data.community_2.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Comment(v) = &search[2] {
      assert_eq!(data.timmy_comment.id, v.comment.id);
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Post(v) = &search[3] {
      assert_eq!(data.sara_post.id, v.post.id);
      assert_eq!(data.community_2.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Post(v) = &search[4] {
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Post(v) = &search[5] {
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Community(v) = &search[6] {
      assert_eq!(data.community_2.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Community(v) = &search[7] {
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Person(v) = &search[8] {
      assert_eq!(data.timmy.id, v.person.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Person(v) = &search[9] {
      assert_eq!(data.sara.id, v.person.id);
    } else {
      panic!("wrong type");
    }

    // Filtered by community id
    let search_by_community = SearchCombinedQuery {
      community_id: Some(data.community.id),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(5, search_by_community);

    // Filtered by creator_id
    let search_by_creator = SearchCombinedQuery {
      creator_id: Some(data.timmy.id),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(4, search_by_creator);

    // Using a term
    let search_by_name = SearchCombinedQuery {
      search_term: Some("gold".into()),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(2, search_by_name);

    // Liked / disliked only
    let search_liked_only = SearchCombinedQuery {
      liked_only: Some(true),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    assert_length!(2, search_liked_only);

    let search_disliked_only = SearchCombinedQuery {
      disliked_only: Some(true),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    assert_length!(1, search_disliked_only);

    // Test sorts
    // Test Old sort
    let search_old_sort = SearchCombinedQuery {
      sort: Some(SearchSortType::Old),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;
    if let SearchCombinedView::Person(v) = &search_old_sort[0] {
      assert_eq!(data.sara.id, v.person.id);
    } else {
      panic!("wrong type");
    }
    assert_length!(10, search_old_sort);

    // Remove a post and delete a comment
    Post::update(
      pool,
      data.timmy_post_2.id,
      &PostUpdateForm {
        removed: Some(true),
        ..Default::default()
      },
    )
    .await?;

    Comment::update(
      pool,
      data.sara_comment.id,
      &CommentUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;

    // 2 things got removed, but the post also has another comment which got removed
    let search = SearchCombinedQuery::default()
      .list(pool, &None, &data.site)
      .await?;
    assert_length!(7, search);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Community search
    let community_search = SearchCombinedQuery {
      type_: Some(SearchType::Communities),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(2, community_search);

    // Make sure the types are correct
    if let SearchCombinedView::Community(v) = &community_search[0] {
      assert_eq!(data.community_2.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Community(v) = &community_search[1] {
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    // Filtered by id
    let community_search_by_id = SearchCombinedQuery {
      community_id: Some(data.community.id),
      type_: Some(SearchType::Communities),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(1, community_search_by_id);

    // Using a term
    let community_search_by_name = SearchCombinedQuery {
      search_term: Some("things".into()),
      type_: Some(SearchType::Communities),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(1, community_search_by_name);
    if let SearchCombinedView::Community(v) = &community_search_by_name[0] {
      // The asklemmy community
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    // Test title only search to make sure 'ask lemmy things' doesn't get returned
    // Using a term
    let community_search_title_only = SearchCombinedQuery {
      search_term: Some("things".into()),
      type_: Some(SearchType::Communities),
      title_only: Some(true),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert!(community_search_title_only.is_empty());

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn person() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Person search
    let person_search = SearchCombinedQuery {
      type_: Some(SearchType::Users),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(2, person_search);

    // Make sure the types are correct
    if let SearchCombinedView::Person(v) = &person_search[0] {
      assert_eq!(data.timmy.id, v.person.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Person(v) = &person_search[1] {
      assert_eq!(data.sara.id, v.person.id);
    } else {
      panic!("wrong type");
    }

    // Filtered by creator_id
    let person_search_by_id = SearchCombinedQuery {
      creator_id: Some(data.sara.id),
      type_: Some(SearchType::Users),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(1, person_search_by_id);
    if let SearchCombinedView::Person(v) = &person_search_by_id[0] {
      assert_eq!(data.sara.id, v.person.id);
    } else {
      panic!("wrong type");
    }

    // Using a term
    let person_search_by_name = SearchCombinedQuery {
      search_term: Some("tim".into()),
      type_: Some(SearchType::Users),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(1, person_search_by_name);
    if let SearchCombinedView::Person(v) = &person_search_by_name[0] {
      assert_eq!(data.timmy.id, v.person.id);
    } else {
      panic!("wrong type");
    }

    // Test Top sorting (uses post score)
    let person_search_sort_top = SearchCombinedQuery {
      type_: Some(SearchType::Users),
      sort: Some(SearchSortType::Top),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(2, person_search_sort_top);

    // Sara should be first, as she has a higher score
    if let SearchCombinedView::Person(v) = &person_search_sort_top[0] {
      assert_eq!(data.sara.id, v.person.id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn post() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // post search
    let post_search = SearchCombinedQuery {
      type_: Some(SearchType::Posts),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(3, post_search);

    // Make sure the types are correct
    if let SearchCombinedView::Post(v) = &post_search[0] {
      assert_eq!(data.sara_post.id, v.post.id);
      assert_eq!(data.community_2.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Post(v) = &post_search[1] {
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Post(v) = &post_search[2] {
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    // Filtered by id
    let post_search_by_community = SearchCombinedQuery {
      community_id: Some(data.community.id),
      type_: Some(SearchType::Posts),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(2, post_search_by_community);

    // Using a term
    let post_search_by_name = SearchCombinedQuery {
      search_term: Some("sara".into()),
      type_: Some(SearchType::Posts),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(1, post_search_by_name);

    // Test title only search to make sure 'postbody' doesn't show up
    // Using a term
    let post_search_title_only = SearchCombinedQuery {
      search_term: Some("postbody".into()),
      type_: Some(SearchType::Posts),
      title_only: Some(true),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert!(post_search_title_only.is_empty());

    // Test title only search to make sure 'postbody' doesn't show up
    // Using a term
    let post_search_url_only = SearchCombinedQuery {
      search_term: data.timmy_post.url.as_ref().map(ToString::to_string),
      post_url_only: Some(true),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(1, post_search_url_only);

    let post_search_partial_url = SearchCombinedQuery {
      search_term: Some("google.c".to_string()),
      post_url_only: Some(true),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(1, post_search_partial_url);

    // Liked / disliked only
    let post_search_liked_only = SearchCombinedQuery {
      type_: Some(SearchType::Posts),
      liked_only: Some(true),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    // Should only be 1 not 2, because liked only ignores your own content
    assert_length!(1, post_search_liked_only);

    let post_search_disliked_only = SearchCombinedQuery {
      type_: Some(SearchType::Posts),
      disliked_only: Some(true),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    // Should be zero because you disliked your own post
    assert_length!(0, post_search_disliked_only);

    // Test top sort
    let post_search_sort_top = SearchCombinedQuery {
      type_: Some(SearchType::Posts),
      sort: Some(SearchSortType::Top),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(3, post_search_sort_top);

    // Timmy_post_2 has a dislike, so it should be last
    if let SearchCombinedView::Post(v) = &post_search_sort_top[2] {
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  // Due to the joins which return children, double check to make sure the search term filters
  // aren't returning child content. IE a search for post title my_post won't return any comments.
  async fn no_children() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Post searches should not return the child comments
    let post_no_children = SearchCombinedQuery {
      search_term: Some("timmy post prv 2".into()),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(1, post_no_children);

    // Community searches should not return posts or comments
    let community_no_children = SearchCombinedQuery {
      search_term: Some("asklemmy".into()),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(1, community_no_children);

    // Person searches should not return communities, posts, or comments
    let person_no_children = SearchCombinedQuery {
      search_term: Some("timmy_pcv".into()),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(1, person_no_children);

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn nsfw_post() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let nsfw_post_search = SearchCombinedQuery {
      type_: Some(SearchType::Posts),
      show_nsfw: Some(true),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(4, nsfw_post_search);

    // Make sure the first is the nsfw
    if let SearchCombinedView::Post(v) = &nsfw_post_search[0] {
      assert_eq!(data.nsfw_post.id, v.post.id);
      assert!(v.post.nsfw);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn nsfw_comment() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let nsfw_comment_search = SearchCombinedQuery {
      type_: Some(SearchType::Comments),
      show_nsfw: Some(true),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(4, nsfw_comment_search);

    // Make sure the first is the nsfw
    if let SearchCombinedView::Comment(v) = &nsfw_comment_search[0] {
      assert_eq!(data.comment_in_nsfw_post.id, v.comment.id);
      assert_eq!(data.nsfw_post.id, v.post.id);
      assert!(v.post.nsfw);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn private_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let unsubbed_private_search = SearchCombinedQuery {
      community_id: Some(data.private_community.id),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    assert_length!(0, unsubbed_private_search);

    // Approve timmy to the community
    let follow_form = CommunityFollowerForm::new(
      data.private_community.id,
      data.timmy.id,
      CommunityFollowerState::ApprovalRequired,
    );

    CommunityActions::follow(pool, &follow_form).await?;
    CommunityActions::approve_private_community_follower(
      pool,
      data.private_community.id,
      data.timmy.id,
      data.sara.id,
      CommunityFollowerState::Accepted,
    )
    .await?;

    let subbed_private_search = SearchCombinedQuery {
      community_id: Some(data.private_community.id),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    // Timmy subscribes to the comm and its accepted
    // 1 community, 1 post, and 1 comment
    assert_length!(3, subbed_private_search);

    // Check the content
    if let SearchCombinedView::Comment(v) = &subbed_private_search[0] {
      assert_eq!(data.timmy_comment_private_comm.id, v.comment.id);
      assert_eq!(data.timmy_post_private_comm.id, v.post.id);
      assert_eq!(data.private_community.id, v.community.id);
    } else {
      panic!("wrong type");
    }
    if let SearchCombinedView::Post(v) = &subbed_private_search[1] {
      assert_eq!(data.timmy_post_private_comm.id, v.post.id);
      assert_eq!(data.private_community.id, v.community.id);
    } else {
      panic!("wrong type");
    }
    if let SearchCombinedView::Community(v) = &subbed_private_search[2] {
      assert_eq!(data.private_community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn comment() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // comment search
    let comment_search = SearchCombinedQuery {
      type_: Some(SearchType::Comments),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(3, comment_search);

    // Make sure the types are correct
    if let SearchCombinedView::Comment(v) = &comment_search[0] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Comment(v) = &comment_search[1] {
      assert_eq!(data.sara_comment.id, v.comment.id);
      assert_eq!(data.sara_post.id, v.post.id);
      assert_eq!(data.community_2.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    if let SearchCombinedView::Comment(v) = &comment_search[2] {
      assert_eq!(data.timmy_comment.id, v.comment.id);
      assert_eq!(data.timmy_post.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    // Filtered by id
    let comment_search_by_community = SearchCombinedQuery {
      community_id: Some(data.community.id),
      type_: Some(SearchType::Comments),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(2, comment_search_by_community);

    // Using a term
    let comment_search_by_name = SearchCombinedQuery {
      search_term: Some("gold".into()),
      type_: Some(SearchType::Comments),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(2, comment_search_by_name);

    // Liked / disliked only
    let comment_search_liked_only = SearchCombinedQuery {
      type_: Some(SearchType::Comments),
      liked_only: Some(true),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    assert_length!(1, comment_search_liked_only);

    let comment_search_disliked_only = SearchCombinedQuery {
      type_: Some(SearchType::Comments),
      disliked_only: Some(true),
      ..Default::default()
    }
    .list(pool, &Some(data.timmy_view.clone()), &data.site)
    .await?;

    assert_length!(1, comment_search_disliked_only);

    // Test top sort
    let comment_search_sort_top = SearchCombinedQuery {
      type_: Some(SearchType::Comments),
      sort: Some(SearchSortType::Top),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(3, comment_search_sort_top);

    // Sara comment 2 is disliked, so should be last
    if let SearchCombinedView::Comment(v) = &comment_search_sort_top[2] {
      assert_eq!(data.sara_comment_2.id, v.comment.id);
      assert_eq!(data.timmy_post_2.id, v.post.id);
      assert_eq!(data.community.id, v.community.id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn multi_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = MultiCommunityInsertForm::new(
      data.timmy_view.person.id,
      data.instance.id,
      "multi".to_string(),
      String::new(),
    );
    let multi = MultiCommunity::create(pool, &form).await?;

    // Multi-community search
    let search = SearchCombinedQuery {
      type_: Some(SearchType::MultiCommunities),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;
    assert_length!(1, search);

    // Make sure the types are correct
    if let SearchCombinedView::MultiCommunity(v) = &search[0] {
      assert_eq!(multi.id, v.multi.id);
    } else {
      panic!("wrong type");
    }

    // Using a term
    let search_by_name = SearchCombinedQuery {
      search_term: Some("multi".into()),
      type_: Some(SearchType::MultiCommunities),
      ..Default::default()
    }
    .list(pool, &None, &data.site)
    .await?;

    assert_length!(1, search_by_name);
    if let SearchCombinedView::MultiCommunity(v) = &search_by_name[0] {
      assert_eq!(multi.id, v.multi.id);
    } else {
      panic!("wrong type");
    }

    cleanup(data, pool).await?;

    Ok(())
  }
}
