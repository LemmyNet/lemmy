use crate::structs::CommentView;
use diesel::{
  dsl::{exists, not},
  pg::Pg,
  result::Error,
  sql_types,
  BoolExpressionMethods,
  BoxableExpression,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{nlevel, subpath, Ltree, LtreeExtensions};
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommentId, CommunityId, LocalUserId, PersonId, PostId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_block,
    community_follower,
    community_moderator,
    community_person_ban,
    instance_block,
    local_user,
    local_user_language,
    person,
    person_block,
    post,
  },
  source::local_user::LocalUser,
  utils::{
    fuzzy_search,
    limit_and_offset,
    visible_communities_only,
    DbConn,
    DbPool,
    ListFn,
    Queries,
    ReadFn,
  },
  CommentSortType,
  ListingType,
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, CommentView, (CommentId, Option<PersonId>)>,
  impl ListFn<'a, CommentView, CommentQuery<'a>>,
> {
  let is_creator_banned_from_community = exists(
    community_person_ban::table.filter(
      community::id
        .eq(community_person_ban::community_id)
        .and(community_person_ban::person_id.eq(comment::creator_id)),
    ),
  );

  let is_local_user_banned_from_community = |person_id| {
    exists(
      community_person_ban::table.filter(
        community::id
          .eq(community_person_ban::community_id)
          .and(community_person_ban::person_id.eq(person_id)),
      ),
    )
  };

  let is_community_followed = |person_id| {
    community_follower::table
      .filter(
        post::community_id
          .eq(community_follower::community_id)
          .and(community_follower::person_id.eq(person_id)),
      )
      .select(community_follower::pending.nullable())
      .single_value()
  };

  let is_creator_blocked = |person_id| {
    exists(
      person_block::table.filter(
        comment::creator_id
          .eq(person_block::target_id)
          .and(person_block::person_id.eq(person_id)),
      ),
    )
  };

  let score = |person_id| {
    comment_like::table
      .filter(
        comment::id
          .eq(comment_like::comment_id)
          .and(comment_like::person_id.eq(person_id)),
      )
      .select(comment_like::score.nullable())
      .single_value()
  };

  let creator_is_moderator = exists(
    community_moderator::table.filter(
      community::id
        .eq(community_moderator::community_id)
        .and(community_moderator::person_id.eq(comment::creator_id)),
    ),
  );

  let creator_is_admin = exists(
    local_user::table.filter(
      comment::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    ),
  );

  let all_joins = move |query: comment::BoxedQuery<'a, Pg>, my_person_id: Option<PersonId>| {
    let is_local_user_banned_from_community_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>,
    > = if let Some(person_id) = my_person_id {
      Box::new(is_local_user_banned_from_community(person_id))
    } else {
      Box::new(false.into_sql::<sql_types::Bool>())
    };

    let score_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(score(person_id))
    } else {
      Box::new(None::<i16>.into_sql::<sql_types::Nullable<sql_types::SmallInt>>())
    };

    let subscribed_type_selection: Box<
      dyn BoxableExpression<_, Pg, SqlType = sql_types::Nullable<sql_types::Bool>>,
    > = if let Some(person_id) = my_person_id {
      Box::new(is_community_followed(person_id))
    } else {
      Box::new(None::<bool>.into_sql::<sql_types::Nullable<sql_types::Bool>>())
    };

    let is_creator_blocked_selection: Box<dyn BoxableExpression<_, Pg, SqlType = sql_types::Bool>> =
      if let Some(person_id) = my_person_id {
        Box::new(is_creator_blocked(person_id))
      } else {
        Box::new(false.into_sql::<sql_types::Bool>())
      };

    query
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(comment_aggregates::table)
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(my_person_id.unwrap_or(PersonId(-1)))),
        ),
      )
      .select((
        comment::all_columns,
        person::all_columns,
        post::all_columns,
        community::all_columns,
        comment_aggregates::all_columns,
        is_creator_banned_from_community,
        is_local_user_banned_from_community_selection,
        creator_is_moderator,
        creator_is_admin,
        subscribed_type_selection,
        comment_saved::person_id.nullable().is_not_null(),
        is_creator_blocked_selection,
        score_selection,
      ))
  };

  let read = move |mut conn: DbConn<'a>,
                   (comment_id, my_person_id): (CommentId, Option<PersonId>)| async move {
    let mut query = all_joins(comment::table.find(comment_id).into_boxed(), my_person_id);
    query = visible_communities_only(my_person_id, query);
    query.first(&mut conn).await
  };

  let list = move |mut conn: DbConn<'a>, options: CommentQuery<'a>| async move {
    // The left join below will return None in this case
    let person_id_join = options.local_user.person_id().unwrap_or(PersonId(-1));
    let local_user_id_join = options
      .local_user
      .local_user_id()
      .unwrap_or(LocalUserId(-1));

    let mut query = all_joins(comment::table.into_boxed(), options.local_user.person_id());

    if let Some(creator_id) = options.creator_id {
      query = query.filter(comment::creator_id.eq(creator_id));
    };

    if let Some(post_id) = options.post_id {
      query = query.filter(comment::post_id.eq(post_id));
    };

    if let Some(parent_path) = options.parent_path.as_ref() {
      query = query.filter(comment::path.contained_by(parent_path));
    };
    //filtering out removed and deleted comments from search
    if let Some(search_term) = options.search_term {
      query = query.filter(
        comment::content
          .ilike(fuzzy_search(&search_term))
          .and(not(comment::removed.or(comment::deleted))),
      );
    };

    if let Some(community_id) = options.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if let Some(listing_type) = options.listing_type {
      let is_subscribed = exists(
        community_follower::table.filter(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      );

      match listing_type {
        ListingType::Subscribed => query = query.filter(is_subscribed), /* TODO could be this: and(community_follower::person_id.eq(person_id_join)), */
        ListingType::Local => {
          query = query
            .filter(community::local.eq(true))
            .filter(community::hidden.eq(false).or(is_subscribed))
        }
        ListingType::All => query = query.filter(community::hidden.eq(false).or(is_subscribed)),
        ListingType::ModeratorView => {
          query = query.filter(exists(
            community_moderator::table.filter(
              post::community_id
                .eq(community_moderator::community_id)
                .and(community_moderator::person_id.eq(person_id_join)),
            ),
          ));
        }
      }
    }

    // If its saved only, then filter, and order by the saved time, not the comment creation time.
    if options.saved_only.unwrap_or_default() {
      query = query
        .filter(comment_saved::person_id.is_not_null())
        .then_order_by(comment_saved::published.desc());
    }

    if let Some(my_id) = options.local_user.person_id() {
      let not_creator_filter = comment::creator_id.ne(my_id);
      if options.liked_only.unwrap_or_default() {
        query = query.filter(not_creator_filter).filter(score(my_id).eq(1));
      } else if options.disliked_only.unwrap_or_default() {
        query = query.filter(not_creator_filter).filter(score(my_id).eq(-1));
      }
    }

    if !options.local_user.show_bot_accounts() {
      query = query.filter(person::bot_account.eq(false));
    };

    if options.local_user.is_some()
      && options.listing_type.unwrap_or_default() != ListingType::ModeratorView
    {
      // Filter out the rows with missing languages
      query = query.filter(exists(
        local_user_language::table.filter(
          comment::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      ));

      // Don't show blocked communities or persons
      query = query.filter(not(exists(
        instance_block::table.filter(
          community::instance_id
            .eq(instance_block::instance_id)
            .and(instance_block::person_id.eq(person_id_join)),
        ),
      )));
      query = query.filter(not(exists(
        community_block::table.filter(
          community::id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )));
      query = query.filter(not(is_creator_blocked(person_id_join)));
    };

    query = visible_communities_only(options.local_user.person_id(), query);

    // A Max depth given means its a tree fetch
    let (limit, offset) = if let Some(max_depth) = options.max_depth {
      let depth_limit = if let Some(parent_path) = options.parent_path.as_ref() {
        parent_path.0.split('.').count() as i32 + max_depth
        // Add one because of root "0"
      } else {
        max_depth + 1
      };

      query = query.filter(nlevel(comment::path).le(depth_limit));

      // only order if filtering by a post id, or parent_path. DOS potential otherwise and max_depth
      // + !post_id isn't used anyways (afaik)
      if options.post_id.is_some() || options.parent_path.is_some() {
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
      // limit_and_offset_unlimited(options.page, options.limit)
      limit_and_offset(options.page, options.limit)?
    };

    // distinguished comments should go first when viewing post
    if options.post_id.is_some() || options.parent_path.is_some() {
      query = query.then_order_by(comment::distinguished.desc());
    }

    query = match options.sort.unwrap_or(CommentSortType::Hot) {
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

    // Note: deleted and removed comments are done on the front side
    query
      .limit(limit)
      .offset(offset)
      .load::<CommentView>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl CommentView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    my_person_id: Option<PersonId>,
  ) -> Result<Option<Self>, Error> {
    // If a person is given, then my_vote (res.9), if None, should be 0, not null
    // Necessary to differentiate between other person's votes
    if let Ok(Some(res)) = queries().read(pool, (comment_id, my_person_id)).await {
      let mut new_view = res.clone();
      if my_person_id.is_some() && res.my_vote.is_none() {
        new_view.my_vote = Some(0);
      }
      if res.comment.deleted || res.comment.removed {
        new_view.comment.content = String::new();
      }
      Ok(Some(new_view))
    } else {
      Ok(None)
    }
  }
}

#[derive(Default)]
pub struct CommentQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CommentSortType>,
  pub community_id: Option<CommunityId>,
  pub post_id: Option<PostId>,
  pub parent_path: Option<Ltree>,
  pub creator_id: Option<PersonId>,
  pub local_user: Option<&'a LocalUser>,
  pub search_term: Option<String>,
  pub saved_only: Option<bool>,
  pub liked_only: Option<bool>,
  pub disliked_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub max_depth: Option<i32>,
}

impl<'a> CommentQuery<'a> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<CommentView>, Error> {
    Ok(
      queries()
        .list(pool, self)
        .await?
        .into_iter()
        .map(|mut c| {
          if c.comment.deleted || c.comment.removed {
            c.comment.content = String::new();
          }
          c
        })
        .collect(),
    )
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{
    comment_view::{CommentQuery, CommentSortType, CommentView, DbPool},
    structs::LocalUserView,
  };
  use lemmy_db_schema::{
    aggregates::structs::CommentAggregates,
    assert_length,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::LanguageId,
    source::{
      actor_language::LocalUserLanguage,
      comment::{
        Comment,
        CommentInsertForm,
        CommentLike,
        CommentLikeForm,
        CommentSaved,
        CommentSavedForm,
        CommentUpdateForm,
      },
      community::{
        Community,
        CommunityInsertForm,
        CommunityModerator,
        CommunityModeratorForm,
        CommunityPersonBan,
        CommunityPersonBanForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm},
      local_user_vote_display_mode::LocalUserVoteDisplayMode,
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      post::{Post, PostInsertForm},
    },
    traits::{Bannable, Blockable, Crud, Joinable, Likeable, Saveable},
    utils::{build_db_pool_for_tests, RANK_DEFAULT},
    CommunityVisibility,
    SubscribedType,
  };
  use lemmy_utils::{error::LemmyResult, LemmyErrorType};
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
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let timmy_person_form = PersonInsertForm::test_form(inserted_instance.id, "timmy");
    let inserted_timmy_person = Person::create(pool, &timmy_person_form).await?;
    let timmy_local_user_form = LocalUserInsertForm::test_form_admin(inserted_timmy_person.id);

    let inserted_timmy_local_user = LocalUser::create(pool, &timmy_local_user_form, vec![]).await?;

    let sara_person_form = PersonInsertForm::test_form(inserted_instance.id, "sara");
    let inserted_sara_person = Person::create(pool, &sara_person_form).await?;

    let new_community = CommunityInsertForm::builder()
      .name("test community 5".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::builder()
      .name("A test post 2".into())
      .creator_id(inserted_timmy_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await?;
    let english_id = Language::read_id_from_code(pool, Some("en")).await?;

    // Create a comment tree with this hierarchy
    //       0
    //     \     \
    //    1      2
    //    \
    //  3  4
    //     \
    //     5
    let comment_form_0 = CommentInsertForm::builder()
      .content("Comment 0".into())
      .creator_id(inserted_timmy_person.id)
      .post_id(inserted_post.id)
      .language_id(english_id)
      .build();

    let inserted_comment_0 = Comment::create(pool, &comment_form_0, None).await?;

    let comment_form_1 = CommentInsertForm::builder()
      .content("Comment 1, A test blocked comment".into())
      .creator_id(inserted_sara_person.id)
      .post_id(inserted_post.id)
      .language_id(english_id)
      .build();

    let inserted_comment_1 =
      Comment::create(pool, &comment_form_1, Some(&inserted_comment_0.path)).await?;

    let finnish_id = Language::read_id_from_code(pool, Some("fi")).await?;
    let comment_form_2 = CommentInsertForm::builder()
      .content("Comment 2".into())
      .creator_id(inserted_timmy_person.id)
      .post_id(inserted_post.id)
      .language_id(finnish_id)
      .build();

    let inserted_comment_2 =
      Comment::create(pool, &comment_form_2, Some(&inserted_comment_0.path)).await?;

    let comment_form_3 = CommentInsertForm::builder()
      .content("Comment 3".into())
      .creator_id(inserted_timmy_person.id)
      .post_id(inserted_post.id)
      .language_id(english_id)
      .build();

    let _inserted_comment_3 =
      Comment::create(pool, &comment_form_3, Some(&inserted_comment_1.path)).await?;

    let polish_id = Language::read_id_from_code(pool, Some("pl"))
      .await?
      .ok_or(LemmyErrorType::LanguageNotAllowed)?;
    let comment_form_4 = CommentInsertForm::builder()
      .content("Comment 4".into())
      .creator_id(inserted_timmy_person.id)
      .post_id(inserted_post.id)
      .language_id(Some(polish_id))
      .build();

    let inserted_comment_4 =
      Comment::create(pool, &comment_form_4, Some(&inserted_comment_1.path)).await?;

    let comment_form_5 = CommentInsertForm::builder()
      .content("Comment 5".into())
      .creator_id(inserted_timmy_person.id)
      .post_id(inserted_post.id)
      .build();

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
      post_id: inserted_post.id,
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
    Ok(Data {
      inserted_instance,
      inserted_comment_0,
      inserted_comment_1,
      inserted_comment_2,
      inserted_post,
      timmy_local_user_view,
      inserted_sara_person,
      inserted_community,
    })
  }

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let expected_comment_view_no_person = expected_comment_view(&data, pool).await?;

    let mut expected_comment_view_with_person = expected_comment_view_no_person.clone();
    expected_comment_view_with_person.my_vote = Some(1);

    let read_comment_views_no_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.inserted_post.id)),
      ..Default::default()
    }
    .list(pool)
    .await?;

    assert_eq!(
      &expected_comment_view_no_person,
      read_comment_views_no_person
        .first()
        .ok_or(LemmyErrorType::CouldntFindComment)?
    );

    let read_comment_views_with_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.inserted_post.id)),
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(pool)
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
      Some(data.timmy_local_user_view.person.id),
    )
    .await?
    .ok_or(LemmyErrorType::CouldntFindComment)?;

    // Make sure block set the creator blocked
    assert!(read_comment_from_blocked_person.creator_blocked);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_liked_only() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
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
      post_id: data.inserted_post.id,
      person_id: data.timmy_local_user_view.person.id,
      score: 1,
    };
    CommentLike::like(pool, &comment_like_form).await?;

    let read_liked_comment_views = CommentQuery {
      local_user: Some(&data.timmy_local_user_view.local_user),
      liked_only: Some(true),
      ..Default::default()
    }
    .list(pool)
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
    .list(pool)
    .await?;

    assert!(read_disliked_comment_views.is_empty());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_comment_tree() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let top_path = data.inserted_comment_0.path.clone();
    let read_comment_views_top_path = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(top_path)),
      ..Default::default()
    }
    .list(pool)
    .await?;

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_child_path = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(child_path)),
      ..Default::default()
    }
    .list(pool)
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
    .list(pool)
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
    .list(pool)
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
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // by default, user has all languages enabled and should see all comments
    // (except from blocked user)
    let all_languages = CommentQuery {
      local_user: (Some(&data.timmy_local_user_view.local_user)),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_length!(5, all_languages);

    // change user lang to finnish, should only show one post in finnish and one undetermined
    let finnish_id = Language::read_id_from_code(pool, Some("fi"))
      .await?
      .ok_or(LemmyErrorType::LanguageNotAllowed)?;
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
    .list(pool)
    .await?;
    assert_length!(2, finnish_comments);
    let finnish_comment = finnish_comments
      .iter()
      .find(|c| c.comment.language_id == finnish_id);
    assert!(finnish_comment.is_some());
    assert_eq!(
      data.inserted_comment_2.content,
      finnish_comment
        .ok_or(LemmyErrorType::CouldntFindComment)?
        .comment
        .content
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
    .list(pool)
    .await?;
    assert_length!(1, undetermined_comment);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_distinguished_first() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
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
    .list(pool)
    .await?;
    assert_eq!(comments[0].comment.id, data.inserted_comment_2.id);
    assert!(comments[0].comment.distinguished);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_creator_is_moderator() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
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
    .list(pool)
    .await?;

    assert_eq!(comments[1].creator.name, "sara");
    assert!(comments[1].creator_is_moderator);
    assert!(!comments[0].creator_is_moderator);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_creator_is_admin() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let comments = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // Timmy is an admin, and make sure that field is true
    assert_eq!(comments[0].creator.name, "timmy");
    assert!(comments[0].creator_is_admin);

    // Sara isn't, make sure its false
    assert_eq!(comments[1].creator.name, "sara");
    assert!(!comments[1].creator_is_admin);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_saved_order() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Save two comments
    let save_comment_0_form = CommentSavedForm {
      person_id: data.timmy_local_user_view.person.id,
      comment_id: data.inserted_comment_0.id,
    };
    CommentSaved::save(pool, &save_comment_0_form).await?;

    let save_comment_2_form = CommentSavedForm {
      person_id: data.timmy_local_user_view.person.id,
      comment_id: data.inserted_comment_2.id,
    };
    CommentSaved::save(pool, &save_comment_2_form).await?;

    // Fetch the saved comments
    let comments = CommentQuery {
      local_user: Some(&data.timmy_local_user_view.local_user),
      saved_only: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;

    // There should only be two comments
    assert_eq!(2, comments.len());

    // The first comment, should be the last one saved (descending order)
    assert_eq!(comments[0].comment.id, data.inserted_comment_2.id);

    // The second comment, should be the first one saved
    assert_eq!(comments[1].comment.id, data.inserted_comment_0.id);

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

    Ok(())
  }

  async fn expected_comment_view(data: &Data, pool: &mut DbPool<'_>) -> LemmyResult<CommentView> {
    let agg = CommentAggregates::read(pool, data.inserted_comment_0.id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindComment)?;
    Ok(CommentView {
      creator_banned_from_community: false,
      banned_from_community: false,
      creator_is_moderator: false,
      creator_is_admin: true,
      my_vote: None,
      subscribed: SubscribedType::NotSubscribed,
      saved: false,
      creator_blocked: false,
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
        actor_id: data.timmy_local_user_view.person.actor_id.clone(),
        local: true,
        banned: false,
        deleted: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: data.timmy_local_user_view.person.inbox_url.clone(),
        shared_inbox_url: None,
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
      },
      community: Community {
        id: data.inserted_community.id,
        name: "test community 5".to_string(),
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: data.inserted_community.actor_id.clone(),
        local: true,
        title: "nada".to_owned(),
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
        shared_inbox_url: data.inserted_community.shared_inbox_url.clone(),
        moderators_url: data.inserted_community.moderators_url.clone(),
        featured_url: data.inserted_community.featured_url.clone(),
        visibility: CommunityVisibility::Public,
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
      },
    })
  }

  #[tokio::test]
  #[serial]
  async fn local_only_instance() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
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
    .list(pool)
    .await?;
    assert_eq!(0, unauthenticated_query.len());

    let authenticated_query = CommentQuery {
      local_user: Some(&data.timmy_local_user_view.local_user),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(5, authenticated_query.len());

    let unauthenticated_comment = CommentView::read(pool, data.inserted_comment_0.id, None).await?;
    assert!(unauthenticated_comment.is_none());

    let authenticated_comment = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(data.timmy_local_user_view.person.id),
    )
    .await;
    assert!(authenticated_comment.is_ok());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_local_user_banned_from_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
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
      Some(inserted_banned_from_comm_local_user.person_id),
    )
    .await?
    .ok_or(LemmyErrorType::CouldntFindComment)?;

    assert!(comment_view.banned_from_community);

    Person::delete(pool, inserted_banned_from_comm_person.id).await?;
    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn comment_listing_local_user_not_banned_from_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let comment_view = CommentView::read(
      pool,
      data.inserted_comment_0.id,
      Some(data.timmy_local_user_view.person.id),
    )
    .await?
    .ok_or(LemmyErrorType::CouldntFindComment)?;

    assert!(!comment_view.banned_from_community);

    cleanup(data, pool).await
  }
}
