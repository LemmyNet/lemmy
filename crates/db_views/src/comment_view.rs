use crate::structs::{CommentView, LocalUserView};
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{nlevel, subpath, Ltree, LtreeExtensions};
use lemmy_db_schema::{
  aggregates::structs::CommentAggregatesNotInComment,
  newtypes::{CommentId, CommunityId, LocalUserId, PersonId, PostId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_saved,
    community,
    community_block,
    community_follower,
    community_person_ban,
    local_user_language,
    person,
    person_block,
    post,
  },
  source::{
    comment::Comment,
    community::{CommunityFollower, CommunityWithoutId},
    person::PersonWithoutId,
    post::PostWithoutId,
  },
  traits::JoinView,
  utils::{fuzzy_search, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  CommentSortType,
  ListingType,
  SubscribedType,
};

type CommentViewTuple = (
  Comment,
  PersonWithoutId,
  PostWithoutId,
  CommunityWithoutId,
  CommentAggregatesNotInComment,
  bool,
  SubscribedType,
  bool,
  bool,
  Option<i16>,
);

fn queries<'a>() -> Queries<
  impl ReadFn<'a, CommentView, (CommentId, Option<PersonId>)>,
  impl ListFn<'a, CommentView, CommentQuery<'a>>,
> {
  let all_joins = |query: comment::BoxedQuery<'a, Pg>, my_person_id: Option<PersonId>| {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));
    query
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(comment_aggregates::table)
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id)),
        ),
      )
      .left_join(
        community_follower::table.on(
          post::community_id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_saved::table.on(
          comment::id
            .eq(comment_saved::comment_id)
            .and(comment_saved::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        person_block::table.on(
          comment::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
  };

  let selection = (
    comment::all_columns,
    PersonWithoutId::as_select(),
    PostWithoutId::as_select(),
    CommunityWithoutId::as_select(),
    CommentAggregatesNotInComment::as_select(),
    community_person_ban::id.nullable().is_not_null(),
    CommunityFollower::select_subscribed_type(),
    comment_saved::id.nullable().is_not_null(),
    person_block::id.nullable().is_not_null(),
    comment_like::score.nullable(),
  );

  let read = move |mut conn: DbConn<'a>,
                   (comment_id, my_person_id): (CommentId, Option<PersonId>)| async move {
    all_joins(comment::table.find(comment_id).into_boxed(), my_person_id)
      .select(selection)
      .first::<CommentViewTuple>(&mut conn)
      .await
  };

  let list = move |mut conn: DbConn<'a>, options: CommentQuery<'a>| async move {
    let person_id = options.local_user.map(|l| l.person.id);
    let local_user_id = options.local_user.map(|l| l.local_user.id);

    // The left join below will return None in this case
    let person_id_join = person_id.unwrap_or(PersonId(-1));
    let local_user_id_join = local_user_id.unwrap_or(LocalUserId(-1));

    let mut query = all_joins(comment::table.into_boxed(), person_id)
      .left_join(
        community_block::table.on(
          community::id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        local_user_language::table.on(
          comment::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      )
      .select(selection);

    if let Some(creator_id) = options.creator_id {
      query = query.filter(comment::creator_id.eq(creator_id));
    };

    if let Some(post_id) = options.post_id {
      query = query.filter(comment::post_id.eq(post_id));
    };

    if let Some(parent_path) = options.parent_path.as_ref() {
      query = query.filter(comment::path.contained_by(parent_path));
    };

    if let Some(search_term) = options.search_term {
      query = query.filter(comment::content.ilike(fuzzy_search(&search_term)));
    };

    if let Some(community_id) = options.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if let Some(listing_type) = options.listing_type {
      match listing_type {
        ListingType::Subscribed => query = query.filter(community_follower::pending.is_not_null()), // TODO could be this: and(community_follower::person_id.eq(person_id_join)),
        ListingType::Local => {
          query = query.filter(community::local.eq(true)).filter(
            community::hidden
              .eq(false)
              .or(community_follower::person_id.eq(person_id_join)),
          )
        }
        ListingType::All => {
          query = query.filter(
            community::hidden
              .eq(false)
              .or(community_follower::person_id.eq(person_id_join)),
          )
        }
      }
    }

    if options.saved_only.unwrap_or(false) {
      query = query.filter(comment_saved::comment_id.is_not_null());
    }

    let is_profile_view = options.is_profile_view.unwrap_or(false);
    let is_creator = options.creator_id == options.local_user.map(|l| l.person.id);
    // only show deleted comments to creator
    if !is_creator {
      query = query.filter(comment::deleted.eq(false));
    }

    let is_admin = options.local_user.map(|l| l.person.admin).unwrap_or(false);
    // only show removed comments to admin when viewing user profile
    if !(is_profile_view && is_admin) {
      query = query.filter(comment::removed.eq(false));
    }

    if !options
      .local_user
      .map(|l| l.local_user.show_bot_accounts)
      .unwrap_or(true)
    {
      query = query.filter(person::bot_account.eq(false));
    };

    if options.local_user.is_some() {
      // Filter out the rows with missing languages
      query = query.filter(local_user_language::language_id.is_not_null());

      // Don't show blocked communities or persons
      if options.post_id.is_none() {
        query = query.filter(community_block::person_id.is_null());
      }
      query = query.filter(person_block::person_id.is_null());
    }

    // A Max depth given means its a tree fetch
    let (limit, offset) = if let Some(max_depth) = options.max_depth {
      let depth_limit = if let Some(parent_path) = options.parent_path.as_ref() {
        parent_path.0.split('.').count() as i32 + max_depth
        // Add one because of root "0"
      } else {
        max_depth + 1
      };

      query = query.filter(nlevel(comment::path).le(depth_limit));

      // only order if filtering by a post id. DOS potential otherwise and max_depth + !post_id isn't used anyways (afaik)
      if options.post_id.is_some() {
        // Always order by the parent path first
        query = query.order_by(subpath(comment::path, 0, -1));
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

    query = match options.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query
        .then_order_by(comment_aggregates::hot_rank.desc())
        .then_order_by(comment_aggregates::score.desc()),
      CommentSortType::Controversial => {
        query.then_order_by(comment_aggregates::controversy_rank.desc())
      }
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    // Note: deleted and removed comments are done on the front side
    query
      .limit(limit)
      .offset(offset)
      .load::<CommentViewTuple>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl CommentView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    // If a person is given, then my_vote (res.9), if None, should be 0, not null
    // Necessary to differentiate between other person's votes
    let mut res = queries().read(pool, (comment_id, my_person_id)).await?;
    if my_person_id.is_some() && res.my_vote.is_none() {
      res.my_vote = Some(0);
    }
    Ok(res)
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
  pub local_user: Option<&'a LocalUserView>,
  pub search_term: Option<String>,
  pub saved_only: Option<bool>,
  pub is_profile_view: Option<bool>,
  pub show_deleted_and_removed: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub max_depth: Option<i32>,
}

impl<'a> CommentQuery<'a> {
  pub async fn list(self, pool: &mut DbPool<'_>) -> Result<Vec<CommentView>, Error> {
    queries().list(pool, self).await
  }
}

impl JoinView for CommentView {
  type JoinTuple = CommentViewTuple;
  fn from_tuple(
    (
      comment,
      creator,
      post,
      community,
      counts,
      creator_banned_from_community,
      subscribed,
      saved,
      creator_blocked,
      my_vote,
    ): Self::JoinTuple,
  ) -> Self {
    Self {
      counts: counts.into_full(&comment),
      community: community.into_full(post.community_id),
      post: post.into_full(comment.post_id),
      creator: creator.into_full(comment.creator_id),
      comment,
      creator_banned_from_community,
      subscribed,
      saved,
      creator_blocked,
      my_vote,
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    comment_view::{
      Comment,
      CommentQuery,
      CommentSortType,
      CommentView,
      DbPool,
    },
    structs::LocalUserView,
  };
  use lemmy_db_schema::{
    aggregates::structs::CommentAggregates,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::LanguageId,
    source::{
      actor_language::LocalUserLanguage,
      comment::{CommentInsertForm, CommentLike, CommentLikeForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      language::Language,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      post::{Post, PostInsertForm},
    },
    traits::{Blockable, Crud, Likeable},
    utils::build_db_pool_for_tests,
    SubscribedType,
  };
  use serial_test::serial;

  struct Data {
    inserted_instance: Instance,
    inserted_comment_0: Comment,
    inserted_comment_1: Comment,
    inserted_comment_2: Comment,
    inserted_post: Post,
    local_user_view: LocalUserView,
    inserted_person_2: Person,
    inserted_community: Community,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> Data {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("timmy".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let inserted_person = Person::create(pool, &new_person).await.unwrap();
    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted(String::new())
      .build();
    let inserted_local_user = LocalUser::create(pool, &local_user_form).await.unwrap();

    let new_person_2 = PersonInsertForm::builder()
      .name("sara".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();
    let inserted_person_2 = Person::create(pool, &new_person_2).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community 5".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post 2".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();
    let english_id = Language::read_id_from_code(pool, Some("en")).await.unwrap();

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
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .language_id(english_id)
      .build();

    let inserted_comment_0 = Comment::create(pool, &comment_form_0, None).await.unwrap();

    let comment_form_1 = CommentInsertForm::builder()
      .content("Comment 1, A test blocked comment".into())
      .creator_id(inserted_person_2.id)
      .post_id(inserted_post.id)
      .language_id(english_id)
      .build();

    let inserted_comment_1 = Comment::create(pool, &comment_form_1, Some(&inserted_comment_0.path))
      .await
      .unwrap();

    let finnish_id = Language::read_id_from_code(pool, Some("fi")).await.unwrap();
    let comment_form_2 = CommentInsertForm::builder()
      .content("Comment 2".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .language_id(finnish_id)
      .build();

    let inserted_comment_2 = Comment::create(pool, &comment_form_2, Some(&inserted_comment_0.path))
      .await
      .unwrap();

    let comment_form_3 = CommentInsertForm::builder()
      .content("Comment 3".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .language_id(english_id)
      .build();

    let _inserted_comment_3 =
      Comment::create(pool, &comment_form_3, Some(&inserted_comment_1.path))
        .await
        .unwrap();

    let polish_id = Language::read_id_from_code(pool, Some("pl"))
      .await
      .unwrap()
      .unwrap();
    let comment_form_4 = CommentInsertForm::builder()
      .content("Comment 4".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .language_id(Some(polish_id))
      .build();

    let inserted_comment_4 = Comment::create(pool, &comment_form_4, Some(&inserted_comment_1.path))
      .await
      .unwrap();

    let comment_form_5 = CommentInsertForm::builder()
      .content("Comment 5".into())
      .creator_id(inserted_person.id)
      .post_id(inserted_post.id)
      .build();

    let _inserted_comment_5 =
      Comment::create(pool, &comment_form_5, Some(&inserted_comment_4.path))
        .await
        .unwrap();

    let timmy_blocks_sara_form = PersonBlockForm {
      person_id: inserted_person.id,
      target_id: inserted_person_2.id,
    };

    let inserted_block = PersonBlock::block(pool, &timmy_blocks_sara_form)
      .await
      .unwrap();

    let expected_block = PersonBlock {
      id: inserted_block.id,
      person_id: inserted_person.id,
      target_id: inserted_person_2.id,
      published: inserted_block.published,
    };
    assert_eq!(expected_block, inserted_block);

    let comment_like_form = CommentLikeForm {
      comment_id: inserted_comment_0.id,
      post_id: inserted_post.id,
      person_id: inserted_person.id,
      score: 1,
    };

    let _inserted_comment_like = CommentLike::like(pool, &comment_like_form).await.unwrap();

    let local_user_view = LocalUserView {
      local_user: inserted_local_user.clone(),
      person: inserted_person.clone(),
      counts: Default::default(),
    };
    Data {
      inserted_instance,
      inserted_comment_0,
      inserted_comment_1,
      inserted_comment_2,
      inserted_post,
      local_user_view,
      inserted_person_2,
      inserted_community,
    }
  }

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    let expected_comment_view_no_person = expected_comment_view(&data, pool).await;

    let mut expected_comment_view_with_person = expected_comment_view_no_person.clone();
    expected_comment_view_with_person.my_vote = Some(1);

    let read_comment_views_no_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.inserted_post.id)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    assert_eq!(
      expected_comment_view_no_person,
      read_comment_views_no_person[0]
    );

    let read_comment_views_with_person = CommentQuery {
      sort: (Some(CommentSortType::Old)),
      post_id: (Some(data.inserted_post.id)),
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    assert_eq!(
      expected_comment_view_with_person,
      read_comment_views_with_person[0]
    );

    // Make sure its 1, not showing the blocked comment
    assert_eq!(5, read_comment_views_with_person.len());

    let read_comment_from_blocked_person = CommentView::read(
      pool,
      data.inserted_comment_1.id,
      Some(data.local_user_view.person.id),
    )
    .await
    .unwrap();

    // Make sure block set the creator blocked
    assert!(read_comment_from_blocked_person.creator_blocked);

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn test_comment_tree() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    let top_path = data.inserted_comment_0.path.clone();
    let read_comment_views_top_path = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(top_path)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_child_path = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(child_path)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    // Make sure the comment parent-limited fetch is correct
    assert_eq!(6, read_comment_views_top_path.len());
    assert_eq!(4, read_comment_views_child_path.len());

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
    .await
    .unwrap();

    // Make sure a depth limited one only has the top comment
    assert_eq!(
      expected_comment_view(&data, pool).await,
      read_comment_views_top_max_depth[0]
    );
    assert_eq!(1, read_comment_views_top_max_depth.len());

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_parent_max_depth = CommentQuery {
      post_id: (Some(data.inserted_post.id)),
      parent_path: (Some(child_path)),
      max_depth: (Some(1)),
      sort: (Some(CommentSortType::New)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();

    // Make sure a depth limited one, and given child comment 1, has 3
    assert!(read_comment_views_parent_max_depth[2]
      .comment
      .content
      .eq("Comment 3"));
    assert_eq!(3, read_comment_views_parent_max_depth.len());

    cleanup(data, pool).await;
  }

  #[tokio::test]
  #[serial]
  async fn test_languages() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    // by default, user has all languages enabled and should see all comments
    // (except from blocked user)
    let all_languages = CommentQuery {
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(5, all_languages.len());

    // change user lang to finnish, should only show one post in finnish and one undetermined
    let finnish_id = Language::read_id_from_code(pool, Some("fi"))
      .await
      .unwrap()
      .unwrap();
    LocalUserLanguage::update(pool, vec![finnish_id], data.local_user_view.local_user.id)
      .await
      .unwrap();
    let finnish_comments = CommentQuery {
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(2, finnish_comments.len());
    let finnish_comment = finnish_comments
      .iter()
      .find(|c| c.comment.language_id == finnish_id);
    assert!(finnish_comment.is_some());
    assert_eq!(
      data.inserted_comment_2.content,
      finnish_comment.unwrap().comment.content
    );

    // now show all comments with undetermined language (which is the default value)
    LocalUserLanguage::update(
      pool,
      vec![UNDETERMINED_ID],
      data.local_user_view.local_user.id,
    )
    .await
    .unwrap();
    let undetermined_comment = CommentQuery {
      local_user: (Some(&data.local_user_view)),
      ..Default::default()
    }
    .list(pool)
    .await
    .unwrap();
    assert_eq!(1, undetermined_comment.len());

    cleanup(data, pool).await;
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) {
    CommentLike::remove(
      pool,
      data.local_user_view.person.id,
      data.inserted_comment_0.id,
    )
    .await
    .unwrap();
    Comment::delete(pool, data.inserted_comment_0.id)
      .await
      .unwrap();
    Comment::delete(pool, data.inserted_comment_1.id)
      .await
      .unwrap();
    Post::delete(pool, data.inserted_post.id).await.unwrap();
    Community::delete(pool, data.inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, data.local_user_view.person.id)
      .await
      .unwrap();
    Person::delete(pool, data.inserted_person_2.id)
      .await
      .unwrap();
    Instance::delete(pool, data.inserted_instance.id)
      .await
      .unwrap();
  }

  async fn expected_comment_view(data: &Data, pool: &mut DbPool<'_>) -> CommentView {
    let agg = CommentAggregates::read(pool, data.inserted_comment_0.id)
      .await
      .unwrap();
    CommentView {
      creator_banned_from_community: false,
      my_vote: None,
      subscribed: SubscribedType::NotSubscribed,
      saved: false,
      creator_blocked: false,
      comment: Comment {
        id: data.inserted_comment_0.id,
        content: "Comment 0".into(),
        creator_id: data.local_user_view.person.id,
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
        id: data.local_user_view.person.id,
        name: "timmy".into(),
        display_name: None,
        published: data.local_user_view.person.published,
        avatar: None,
        actor_id: data.local_user_view.person.actor_id.clone(),
        local: true,
        banned: false,
        deleted: false,
        admin: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: data.local_user_view.person.inbox_url.clone(),
        shared_inbox_url: None,
        matrix_user_id: None,
        ban_expires: None,
        instance_id: data.inserted_instance.id,
        private_key: data.local_user_view.person.private_key.clone(),
        public_key: data.local_user_view.person.public_key.clone(),
        last_refreshed_at: data.local_user_view.person.last_refreshed_at,
      },
      post: Post {
        id: data.inserted_post.id,
        name: data.inserted_post.name.clone(),
        creator_id: data.local_user_view.person.id,
        url: None,
        body: None,
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
      },
      counts: CommentAggregates {
        id: agg.id,
        comment_id: data.inserted_comment_0.id,
        score: 1,
        upvotes: 1,
        downvotes: 0,
        published: agg.published,
        child_count: 5,
        hot_rank: 1728,
        controversy_rank: 0.0,
      },
    }
  }
}
