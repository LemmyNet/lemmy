use crate::structs::{CommentSlimView, CommentView};
use diesel::{
  dsl::exists,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{nlevel, subpath, Ltree, LtreeExtensions};
use lemmy_db_schema::{
  aliases::creator_community_actions,
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommentId, CommunityId, PersonId, PostId},
  schema::{
    comment,
    comment_actions,
    comment_aggregates,
    community,
    community_actions,
    instance_actions,
    local_user,
    local_user_language,
    person,
    person_actions,
    post,
  },
  source::{community::CommunityFollowerState, local_user::LocalUser, site::Site},
  utils::{get_conn, limit_and_offset, now, seconds_to_pg_interval, DbPool},
  CommentSortType,
  CommunityVisibility,
  ListingType,
};

impl CommentView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: Option<PersonId>) -> _ {
    let community_join = community::table.on(post::community_id.eq(community::id));

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(post::community_id)
        .and(community_actions::person_id.nullable().eq(my_person_id)),
    );

    let comment_actions_join = comment_actions::table.on(
      comment_actions::comment_id
        .eq(comment_aggregates::comment_id)
        .and(comment_actions::person_id.nullable().eq(my_person_id)),
    );

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(comment::creator_id)
        .and(person_actions::person_id.nullable().eq(my_person_id)),
    );

    let instance_actions_join = instance_actions::table.on(
      instance_actions::instance_id
        .eq(community::instance_id)
        .and(instance_actions::person_id.nullable().eq(my_person_id)),
    );

    let comment_creator_community_actions_join = creator_community_actions.on(
      creator_community_actions
        .field(community_actions::community_id)
        .eq(post::community_id)
        .and(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(comment::creator_id),
        ),
    );

    let local_user_join = local_user::table.on(local_user::person_id.nullable().eq(my_person_id));

    comment::table
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community_join)
      .inner_join(comment_aggregates::table)
      .left_join(community_actions_join)
      .left_join(comment_actions_join)
      .left_join(person_actions_join)
      .left_join(instance_actions_join)
      .left_join(comment_creator_community_actions_join)
      .left_join(local_user_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    my_local_user: Option<&'_ LocalUser>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    let mut query = Self::joins(my_local_user.person_id())
      .filter(comment::id.eq(comment_id))
      .select(Self::as_select())
      .into_boxed();

    query = my_local_user.visible_communities_only(query);

    // Check permissions to view private community content.
    // Specifically, if the community is private then only accepted followers may view its
    // content, otherwise it is filtered out. Admins can view private community content
    // without restriction.
    if !my_local_user.is_admin() {
      query = query.filter(
        community::visibility
          .ne(CommunityVisibility::Private)
          .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
      );
    }

    let mut res = query.first::<Self>(conn).await?;

    // If a person is given, then my_vote (res.9), if None, should be 0, not null
    // Necessary to differentiate between other person's votes
    if my_local_user.is_some() && res.my_vote.is_none() {
      res.my_vote = Some(0);
    }

    Ok(res)
  }

  pub fn map_to_slim(self) -> CommentSlimView {
    CommentSlimView {
      comment: self.comment,
      creator: self.creator,
      counts: self.counts,
      creator_banned_from_community: self.creator_banned_from_community,
      banned_from_community: self.banned_from_community,
      creator_is_moderator: self.creator_is_moderator,
      creator_is_admin: self.creator_is_admin,
      subscribed: self.subscribed,
      saved: self.saved,
      creator_blocked: self.creator_blocked,
      my_vote: self.my_vote,
      can_mod: self.can_mod,
    }
  }
}

#[derive(Default)]
pub struct CommentQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CommentSortType>,
  pub time_range_seconds: Option<i32>,
  pub community_id: Option<CommunityId>,
  pub post_id: Option<PostId>,
  pub parent_path: Option<Ltree>,
  pub creator_id: Option<PersonId>,
  pub local_user: Option<&'a LocalUser>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub max_depth: Option<i32>,
}

impl CommentQuery<'_> {
  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> Result<Vec<CommentView>, Error> {
    let conn = &mut get_conn(pool).await?;
    let o = self;

    // The left join below will return None in this case
    let my_person_id = o.local_user.person_id();
    let local_user_id = o.local_user.local_user_id();

    let mut query = CommentView::joins(my_person_id)
      .select(CommentView::as_select())
      .into_boxed();

    if let Some(creator_id) = o.creator_id {
      query = query.filter(comment::creator_id.eq(creator_id));
    };

    if let Some(post_id) = o.post_id {
      query = query.filter(comment::post_id.eq(post_id));
    };

    if let Some(parent_path) = o.parent_path.as_ref() {
      query = query.filter(comment::path.contained_by(parent_path));
    };

    if let Some(community_id) = o.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    let is_subscribed = community_actions::followed.is_not_null();

    // For posts, we only show hidden if its subscribed, but for comments,
    // we ignore hidden.
    query = match o.listing_type.unwrap_or_default() {
      ListingType::Subscribed => query.filter(is_subscribed),
      ListingType::Local => query.filter(community::local.eq(true)),
      ListingType::All => query,
      ListingType::ModeratorView => query.filter(community_actions::became_moderator.is_not_null()),
    };

    if let Some(my_id) = my_person_id {
      let not_creator_filter = comment::creator_id.ne(my_id);
      if o.liked_only.unwrap_or_default() {
        query = query
          .filter(not_creator_filter)
          .filter(comment_actions::like_score.eq(1));
      } else if o.disliked_only.unwrap_or_default() {
        query = query
          .filter(not_creator_filter)
          .filter(comment_actions::like_score.eq(-1));
      }
    }

    if !o.local_user.show_bot_accounts() {
      query = query.filter(person::bot_account.eq(false));
    };

    if o.local_user.is_some() && o.listing_type.unwrap_or_default() != ListingType::ModeratorView {
      // Filter out the rows with missing languages
      query = query.filter(exists(
        local_user_language::table.filter(
          comment::language_id
            .eq(local_user_language::language_id)
            .and(
              local_user_language::local_user_id
                .nullable()
                .eq(local_user_id),
            ),
        ),
      ));

      // Don't show blocked communities or persons
      query = query
        .filter(instance_actions::blocked.is_null())
        .filter(community_actions::blocked.is_null())
        .filter(person_actions::blocked.is_null());
    };

    if !o.local_user.show_nsfw(site) {
      query = query
        .filter(post::nsfw.eq(false))
        .filter(community::nsfw.eq(false));
    };

    query = o.local_user.visible_communities_only(query);

    if !o.local_user.is_admin() {
      query = query.filter(
        community::visibility
          .ne(CommunityVisibility::Private)
          .or(community_actions::follow_state.eq(CommunityFollowerState::Accepted)),
      );
    }

    // A Max depth given means its a tree fetch
    let (limit, offset) = if let Some(max_depth) = o.max_depth {
      let depth_limit = if let Some(parent_path) = o.parent_path.as_ref() {
        parent_path.0.split('.').count() as i32 + max_depth
        // Add one because of root "0"
      } else {
        max_depth + 1
      };

      query = query.filter(nlevel(comment::path).le(depth_limit));

      // only order if filtering by a post id, or parent_path. DOS potential otherwise and max_depth
      // + !post_id isn't used anyways (afaik)
      if o.post_id.is_some() || o.parent_path.is_some() {
        // Always order by the parent path first
        query = query.then_order_by(subpath(comment::path, 0, -1));
      }

      // TODO limit question. Limiting does not work for comment threads ATM, only max_depth
      // For now, don't do any limiting for tree fetches
      // https://stackoverflow.com/questions/72983614/postgres-ltree-how-to-limit-the-max-number-of-children-at-any-given-level

      // Don't use the regular error-checking one, many more comments must ofter be fetched.
      // This does not work for comment trees, and the limit should be manually set to a high number
      //
      // If a max depth is given, then you know its a tree fetch, and limits should be ignored
      // TODO a kludge to prevent attacks. Limit comments to 300 for now.
      // (i64::MAX, 0)
      (300, 0)
    } else {
      // limit_and_offset_unlimited(o.page, o.limit)
      limit_and_offset(o.page, o.limit)?
    };

    // distinguished comments should go first when viewing post
    if o.post_id.is_some() || o.parent_path.is_some() {
      query = query.then_order_by(comment::distinguished.desc());
    }

    query = match o.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query
        .then_order_by(comment_aggregates::hot_rank.desc())
        .then_order_by(comment_aggregates::score.desc()),
      CommentSortType::Controversial => {
        query.then_order_by(comment_aggregates::controversy_rank.desc())
      }
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.then_order_by(comment_aggregates::score.desc()),
    };

    // Filter by the time range
    if let Some(time_range_seconds) = o.time_range_seconds {
      query = query.filter(
        comment_aggregates::published.gt(now() - seconds_to_pg_interval(time_range_seconds)),
      );
    }

    let res = query
      .limit(limit)
      .offset(offset)
      .load::<CommentView>(conn)
      .await?;

    Ok(res)
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::{
    comment::comment_view::{CommentQuery, CommentSortType, CommentView, DbPool},
    structs::LocalUserView,
  };
  use lemmy_db_schema::{
    aggregates::structs::CommentAggregates,
    assert_length,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::LanguageId,
    source::{
      actor_language::LocalUserLanguage,
      comment::{Comment, CommentInsertForm, CommentLike, CommentLikeForm, CommentUpdateForm},
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
      instance::Instance,
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
      local_user_vote_display_mode::LocalUserVoteDisplayMode,
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      post::{Post, PostInsertForm, PostUpdateForm},
      site::{Site, SiteInsertForm},
    },
    traits::{Bannable, Blockable, Crud, Followable, Joinable, Likeable},
    utils::{build_db_pool_for_tests, RANK_DEFAULT},
    CommunityVisibility,
    SubscribedType,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    inserted_instance: Instance,
    inserted_comment_0: Comment,
    inserted_comment_1: Comment,
    inserted_comment_2: Comment,
    inserted_post: Post,
    timmy_local_user_view: LocalUserView,
    inserted_sara_person: Person,
    inserted_community: Community,
    site: Site,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_person_form = PersonInsertForm::test_form(inserted_instance.id, "timmy");
    let inserted_timmy_person = Person::create(pool, &timmy_person_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form_admin(inserted_timmy_person.id);

    let inserted_timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;

    let sara_person_form = PersonInsertForm::test_form(inserted_instance.id, "sara");
    let inserted_sara_person = Person::create(pool, &sara_person_form).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "test community 5".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "A test post 2".into(),
      inserted_timmy_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;
    let english_id = Language::read_id_from_code(pool, "en").await?;

    // Create a comment tree with this hierarchy
    //       0
    //     \     \
    //    1      2
    //    \
    //  3  4
    //     \
    //     5
    let comment_form_0 = CommentInsertForm {
      language_id: Some(english_id),
      ..CommentInsertForm::new(
        inserted_timmy_person.id,
        inserted_post.id,
        "Comment 0".into(),
      )
    };

    let inserted_comment_0 = Comment::create(pool, &comment_form_0, None).await?;

    let comment_form_1 = CommentInsertForm {
      language_id: Some(english_id),
      ..CommentInsertForm::new(
        inserted_sara_person.id,
        inserted_post.id,
        "Comment 1, A test blocked comment".into(),
      )
    };
    let inserted_comment_1 =
      Comment::create(pool, &comment_form_1, Some(&inserted_comment_0.path)).await?;

    let finnish_id = Language::read_id_from_code(pool, "fi").await?;
    let comment_form_2 = CommentInsertForm {
      language_id: Some(finnish_id),
      ..CommentInsertForm::new(
        inserted_timmy_person.id,
        inserted_post.id,
        "Comment 2".into(),
      )
    };

    let inserted_comment_2 =
      Comment::create(pool, &comment_form_2, Some(&inserted_comment_0.path)).await?;

    let comment_form_3 = CommentInsertForm {
      language_id: Some(english_id),
      ..CommentInsertForm::new(
        inserted_timmy_person.id,
        inserted_post.id,
        "Comment 3".into(),
      )
    };
    let _inserted_comment_3 =
      Comment::create(pool, &comment_form_3, Some(&inserted_comment_1.path)).await?;

    let polish_id = Language::read_id_from_code(pool, "pl").await?;
    let comment_form_4 = CommentInsertForm {
      language_id: Some(polish_id),
      ..CommentInsertForm::new(
        inserted_timmy_person.id,
        inserted_post.id,
        "Comment 4".into(),
      )
    };

    let inserted_comment_4 =
      Comment::create(pool, &comment_form_4, Some(&inserted_comment_1.path)).await?;

    let comment_form_5 = CommentInsertForm::new(
      inserted_timmy_person.id,
      inserted_post.id,
      "Comment 5".into(),
    );
    let _inserted_comment_5 =
      Comment::create(pool, &comment_form_5, Some(&inserted_comment_4.path)).await?;

    let timmy_blocks_sara_form = PersonBlockForm {
      person_id: inserted_timmy_person.id,
      target_id: inserted_sara_person.id,
    };

    let inserted_block = PersonBlock::block(pool, &timmy_blocks_sara_form).await?;

    let expected_block = PersonBlock {
      person_id: inserted_timmy_person.id,
      target_id: inserted_sara_person.id,
      published: inserted_block.published,
    };
    assert_eq!(expected_block, inserted_block);

    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment_0.id,
      person_id: inserted_timmy_person.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(pool, &comment_like_form).await?;

    let timmy_local_user_view = LocalUserView {
      local_user: inserted_timmy_local_user.clone(),
      local_user_vote_display_mode: LocalUserVoteDisplayMode::default(),
      person: inserted_timmy_person.clone(),
      counts: Default::default(),
    };
    let site_form = SiteInsertForm::new("test site".to_string(), inserted_instance.id);
    let site = Site::create(pool, &site_form).await?;
    Ok(Data {
      inserted_instance,
      inserted_comment_0,
      inserted_comment_1,
      inserted_comment_2,
      inserted_post,
      timmy_local_user_view,
      inserted_sara_person,
      inserted_community,
      site,
    })
  }

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let expected_comment_view_no_person = expected_comment_view(&data, pool).await?;

    let mut expected_comment_view_with_person = expected_comment_view_no_person.clone();
    expected_comment_view_with_person.my_vote = Some(1);
    expected_comment_view_with_person.can_mod = true;

    let read_comment_views_no_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.inserted_post.id)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert_eq!(
      Some(&expected_comment_view_no_person),
      read_comment_views_no_person.first()
    );

    let read_comment_views_with_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.inserted_post.id)),
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert_eq!(
      expected_comment_view_with_person,
      read_comment_views_with_person[0]
    );

    // Make sure its 1, not showing the blocked comment
    assert_length!(5, read_comment_views_with_person);

    let read_comment_from_blocked_person = CommentView::read(
      pool,
      data.inserted_comment_1.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await?;

    // Make sure block set the creator blocked
    assert!(read_comment_from_blocked_person.creator_blocked);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_liked_only() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Unblock sara first
    let timmy_unblocks_sara_form = PersonBlockForm {
      person_id: data.timmy_local_user_view.person.id,
      target_id: data.inserted_sara_person.id,
    };
    PersonBlock::unblock(pool, &timmy_unblocks_sara_form).await?;

    // Like a new comment
    let comment_like_form = CommentLikeForm {
      comment_id: data.inserted_comment_1.id,
      person_id: data.timmy_local_user_view.person.id,
      score: 1,
    };
    CommentLike::like(pool, &comment_like_form).await?;

    let read_liked_comment_views = CommentQuery {
      local_user: Some(&data.timmy_local_user_view.local_user),
      liked_only: Some(true),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .into_iter()
    .map(|c| c.comment.content)
    .collect::<Vec<String>>();

    // Shouldn't include your own post, only other peoples
    assert_eq!(data.inserted_comment_1.content, read_liked_comment_views[0]);

    assert_length!(1, read_liked_comment_views);

    let read_disliked_comment_views: Vec<CommentView> = CommentQuery {
      local_user: Some(&data.timmy_local_user_view.local_user),
      disliked_only: Some(true),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert!(read_disliked_comment_views.is_empty());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_comment_tree() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let top_path = data.inserted_comment_0.path.clone();
    let read_comment_views_top_path = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(top_path)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_child_path = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(child_path)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Make sure the comment parent-limited fetch is correct
    assert_length!(6, read_comment_views_top_path);
    assert_length!(4, read_comment_views_child_path);

    // Make sure it contains the parent, but not the comment from the other tree
    let child_comments = read_comment_views_child_path
      .into_iter()
      .map(|c| c.comment)
      .collect::<Vec<Comment>>();
    assert!(child_comments.contains(&data.inserted_comment_1));
    assert!(!child_comments.contains(&data.inserted_comment_2));

    let read_comment_views_top_max_depth = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      max_depth: (Some(1)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Make sure a depth limited one only has the top comment
    assert_eq!(
      expected_comment_view(&data, pool).await?,
      read_comment_views_top_max_depth[0]
    );
    assert_length!(1, read_comment_views_top_max_depth);

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_parent_max_depth = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(child_path)),
      max_depth: (Some(1)),
      sort: (Some(CommentSortType::New)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Make sure a depth limited one, and given child comment 1, has 3
    assert!(read_comment_views_parent_max_depth[2]
      .comment
      .content
      .eq("Comment 3"));
    assert_length!(3, read_comment_views_parent_max_depth);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_languages() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // by default, user has all languages enabled and should see all comments
    // (except from blocked user)
    let all_languages = CommentQuery {
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_length!(5, all_languages);

    // change user lang to finnish, should only show one post in finnish and one undetermined
    let finnish_id = Language::read_id_from_code(pool, "fi").await?;
    LocalUserLanguage::update(
      pool,
      vec![finnish_id],
      data.timmy_local_user_view.local_user.id,
    )
    .await?;
    let finnish_comments = CommentQuery {
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_length!(2, finnish_comments);
    let finnish_comment = finnish_comments
      .iter()
      .find(|c| c.comment.language_id == finnish_id);
    assert!(finnish_comment.is_some());
    assert_eq!(
      Some(&data.inserted_comment_2.content),
      finnish_comment.map(|c| &c.comment.content)
    );

    // now show all comments with undetermined language (which is the default value)
    LocalUserLanguage::update(
      pool,
      vec![UNDETERMINED_ID],
      data.timmy_local_user_view.local_user.id,
    )
    .await?;
    let undetermined_comment = CommentQuery {
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_length!(1, undetermined_comment);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_distinguished_first() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let form = CommentUpdateForm {
      distinguished: Some(true),
      ..Default::default()
    };
    Comment::update(pool, data.inserted_comment_2.id, &form).await?;

    let comments = CommentQuery {
      post_id: Some(data.inserted_comment_2.post_id),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(comments[0].comment.id, data.inserted_comment_2.id);
    assert!(comments[0].comment.distinguished);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_creator_is_moderator() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Make one of the inserted persons a moderator
    let person_id = data.inserted_sara_person.id;
    let community_id = data.inserted_community.id;
    let form = CommunityModeratorForm {
      community_id,
      person_id,
    };
    CommunityModerator::join(pool, &form).await?;

    // Make sure that they come back as a mod in the list
    let comments = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    assert_eq!(comments[1].creator.name, "sara");
    assert!(comments[1].creator_is_moderator);
    assert!(!comments[0].creator_is_moderator);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_creator_is_admin() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let comments = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;

    // Timmy is an admin, and make sure that field is true
    assert_eq!(comments[0].creator.name, "timmy");
    assert!(comments[0].creator_is_admin);

    // Sara isn't, make sure its false
    assert_eq!(comments[1].creator.name, "sara");
    assert!(!comments[1].creator_is_admin);

    cleanup(data, pool).await
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    CommentLike::remove(
      pool,
      data.timmy_local_user_view.person.id,
      data.inserted_comment_0.id,
    )
    .await?;
    Comment::delete(pool, data.inserted_comment_0.id).await?;
    Comment::delete(pool, data.inserted_comment_1.id).await?;
    Post::delete(pool, data.inserted_post.id).await?;
    Community::delete(pool, data.inserted_community.id).await?;
    Person::delete(pool, data.timmy_local_user_view.person.id).await?;
    LocalUser::delete(pool, data.timmy_local_user_view.local_user.id).await?;
    Person::delete(pool, data.inserted_sara_person.id).await?;
    Instance::delete(pool, data.inserted_instance.id).await?;
    Site::delete(pool, data.site.id).await?;

    Ok(())
  }

  async fn expected_comment_view(data: &Data, pool: &mut DbPool<'_>) -> LemmyResult<CommentView> {
    let agg = CommentAggregates::read(pool, data.inserted_comment_0.id).await?;
    Ok(CommentView {
      creator_banned_from_community: false,
      banned_from_community: false,
      creator_is_moderator: false,
      creator_is_admin: true,
      my_vote: None,
      subscribed: SubscribedType::NotSubscribed,
      saved: None,
      creator_blocked: false,
      can_mod: false,
      comment: Comment {
        id: data.inserted_comment_0.id,
        content: "Comment 0".into(),
        creator_id: data.timmy_local_user_view.person.id,
        post_id: data.inserted_post.id,
        removed: false,
        deleted: false,
        published: data.inserted_comment_0.published,
        ap_id: data.inserted_comment_0.ap_id.clone(),
        updated: None,
        local: true,
        distinguished: false,
        path: data.inserted_comment_0.clone().path,
        language_id: LanguageId(37),
      },
      creator: Person {
        id: data.timmy_local_user_view.person.id,
        name: "timmy".into(),
        display_name: None,
        published: data.timmy_local_user_view.person.published,
        avatar: None,
        ap_id: data.timmy_local_user_view.person.ap_id.clone(),
        local: true,
        banned: false,
        deleted: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: data.timmy_local_user_view.person.inbox_url.clone(),
        matrix_user_id: None,
        ban_expires: None,
        instance_id: data.inserted_instance.id,
        private_key: data.timmy_local_user_view.person.private_key.clone(),
        public_key: data.timmy_local_user_view.person.public_key.clone(),
        last_refreshed_at: data.timmy_local_user_view.person.last_refreshed_at,
      },
      post: Post {
        id: data.inserted_post.id,
        name: data.inserted_post.name.clone(),
        creator_id: data.timmy_local_user_view.person.id,
        url: None,
        body: None,
        alt_text: None,
        published: data.inserted_post.published,
        updated: None,
        community_id: data.inserted_community.id,
        removed: false,
        deleted: false,
        locked: false,
        nsfw: false,
        embed_title: None,
        embed_description: None,
        embed_video_url: None,
        thumbnail_url: None,
        ap_id: data.inserted_post.ap_id.clone(),
        local: true,
        language_id: Default::default(),
        featured_community: false,
        featured_local: false,
        url_content_type: None,
        scheduled_publish_time: None,
      },
      community: Community {
        id: data.inserted_community.id,
        name: "test community 5".to_string(),
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        ap_id: data.inserted_community.ap_id.clone(),
        local: true,
        title: "nada".to_owned(),
        sidebar: None,
        description: None,
        updated: None,
        banner: None,
        hidden: false,
        posting_restricted_to_mods: false,
        published: data.inserted_community.published,
        instance_id: data.inserted_instance.id,
        private_key: data.inserted_community.private_key.clone(),
        public_key: data.inserted_community.public_key.clone(),
        last_refreshed_at: data.inserted_community.last_refreshed_at,
        followers_url: data.inserted_community.followers_url.clone(),
        inbox_url: data.inserted_community.inbox_url.clone(),
        moderators_url: data.inserted_community.moderators_url.clone(),
        featured_url: data.inserted_community.featured_url.clone(),
        visibility: CommunityVisibility::Public,
        random_number: data.inserted_community.random_number,
      },
      counts: CommentAggregates {
        comment_id: data.inserted_comment_0.id,
        score: 1,
        upvotes: 1,
        downvotes: 0,
        published: agg.published,
        child_count: 5,
        hot_rank: RANK_DEFAULT,
        controversy_rank: 0.0,
        report_count: 0,
        unresolved_report_count: 0,
      },
    })
  }

  #[tokio::test]
  #[serial]
  async fn local_only_instance() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    Community::update(
      pool,
      data.inserted_community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::LocalOnly),
        ..Default::default()
      },
    )
    .await?;

    let unauthenticated_query = CommentQuery {
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(0, unauthenticated_query.len());

    let authenticated_query = CommentQuery {
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(5, authenticated_query.len());

    let unauthenticated_comment = CommentView::read(pool, data.inserted_comment_0.id, None).await;
    assert!(unauthenticated_comment.is_err());

    let authenticated_comment = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await;
    assert!(authenticated_comment.is_ok());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_local_user_banned_from_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Test that comment view shows if local user is blocked from community
    let banned_from_comm_person = PersonInsertForm::test_form(data.inserted_instance.id, "jill");

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
        community_id: data.inserted_community.id,
        person_id: inserted_banned_from_comm_person.id,
        expires: None,
      },
    )
    .await?;

    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&inserted_banned_from_comm_local_user),
    )
    .await?;

    assert!(comment_view.banned_from_community);

    Person::delete(pool, inserted_banned_from_comm_person.id).await?;
    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_local_user_not_banned_from_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await?;

    assert!(!comment_view.banned_from_community);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listings_hide_nsfw() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Mark a post as nsfw
    let update_form = PostUpdateForm {
      nsfw: Some(true),
      ..Default::default()
    };
    Post::update(pool, data.inserted_post.id, &update_form).await?;

    // Make sure comments of this post are not returned
    let comments = CommentQuery::default().list(&data.site, pool).await?;
    assert_eq!(0, comments.len());

    // Mark site as nsfw
    let mut site = data.site.clone();
    site.content_warning = Some("nsfw".to_string());

    // Now comments of nsfw post are returned
    let comments = CommentQuery::default().list(&site, pool).await?;
    assert_eq!(6, comments.len());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_private_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let mut data = init_data(pool).await?;

    // Mark community as private
    Community::update(
      pool,
      data.inserted_community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::Private),
        ..Default::default()
      },
    )
    .await?;

    // No comments returned without auth
    let read_comment_listing = CommentQuery::default().list(&data.site, pool).await?;
    assert_eq!(0, read_comment_listing.len());
    let comment_view = CommentView::read(pool, data.inserted_comment_0.id, None).await;
    assert!(comment_view.is_err());

    // No comments returned for non-follower who is not admin
    data.timmy_local_user_view.local_user.admin = false;
    let read_comment_listing = CommentQuery {
      community_id: Some(data.inserted_community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(0, read_comment_listing.len());
    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await;
    assert!(comment_view.is_err());

    // Admin can view content without following
    data.timmy_local_user_view.local_user.admin = true;
    let read_comment_listing = CommentQuery {
      community_id: Some(data.inserted_community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(5, read_comment_listing.len());
    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await;
    assert!(comment_view.is_ok());
    data.timmy_local_user_view.local_user.admin = false;

    // User can view after following
    CommunityFollower::follow(
      pool,
      &CommunityFollowerForm {
        state: Some(CommunityFollowerState::Accepted),
        ..CommunityFollowerForm::new(
          data.inserted_community.id,
          data.timmy_local_user_view.person.id,
        )
      },
    )
    .await?;
    let read_comment_listing = CommentQuery {
      community_id: Some(data.inserted_community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(5, read_comment_listing.len());
    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await;
    assert!(comment_view.is_ok());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_removed() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let mut data = init_data(pool).await?;

    // Mark a comment as removed
    let form = CommentUpdateForm {
      removed: Some(true),
      ..Default::default()
    };
    Comment::update(pool, data.inserted_comment_0.id, &form).await?;

    // Read as normal user, content is cleared
    // Timmy leaves admin
    LocalUser::update(
      pool,
      data.timmy_local_user_view.local_user.id,
      &LocalUserUpdateForm {
        admin: Some(false),
        ..Default::default()
      },
    )
    .await?;
    data.timmy_local_user_view.local_user.admin = false;
    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await?;
    assert_eq!("", comment_view.comment.content);
    let comment_listing = CommentQuery {
      community_id: Some(data.inserted_community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      sort: Some(CommentSortType::Old),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!("", comment_listing[0].comment.content);

    // Read as admin, content is returned
    LocalUser::update(
      pool,
      data.timmy_local_user_view.local_user.id,
      &LocalUserUpdateForm {
        admin: Some(true),
        ..Default::default()
      },
    )
    .await?;
    data.timmy_local_user_view.local_user.admin = true;
    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(&data.timmy_local_user_view.local_user),
    )
    .await?;
    assert_eq!(
      data.inserted_comment_0.content,
      comment_view.comment.content
    );
    let comment_listing = CommentQuery {
      community_id: Some(data.inserted_community.id),
      local_user: Some(&data.timmy_local_user_view.local_user),
      sort: Some(CommentSortType::Old),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(
      data.inserted_comment_0.content,
      comment_listing[0].comment.content
    );

    cleanup(data, pool).await
  }
}
