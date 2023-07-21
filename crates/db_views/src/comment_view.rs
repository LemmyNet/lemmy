use crate::structs::CommentView;
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use diesel_ltree::{nlevel, subpath, Ltree, LtreeExtensions};
use lemmy_db_schema::{
  aggregates::structs::CommentAggregates,
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
    comment::{Comment, CommentSaved},
    community::{Community, CommunityFollower, CommunityPersonBan},
    local_user::LocalUser,
    person::Person,
    person_block::PersonBlock,
    post::Post,
  },
  traits::JoinView,
  utils::{fuzzy_search, get_conn, limit_and_offset, DbPool},
  CommentSortType,
  ListingType,
};
use typed_builder::TypedBuilder;

type CommentViewTuple = (
  Comment,
  Person,
  Post,
  Community,
  CommentAggregates,
  Option<CommunityPersonBan>,
  Option<CommunityFollower>,
  Option<CommentSaved>,
  Option<PersonBlock>,
  Option<i16>,
);

impl CommentView {
  pub async fn read(
    pool: &DbPool,
    comment_id: CommentId,
    my_person_id: Option<PersonId>,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let (
      comment,
      creator,
      post,
      community,
      counts,
      creator_banned_from_community,
      follower,
      saved,
      creator_blocked,
      comment_like,
    ) = comment::table
      .find(comment_id)
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
      .select((
        comment::all_columns,
        person::all_columns,
        post::all_columns,
        community::all_columns,
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .first::<CommentViewTuple>(conn)
      .await?;

    // If a person is given, then my_vote, if None, should be 0, not null
    // Necessary to differentiate between other person's votes
    let my_vote = if my_person_id.is_some() && comment_like.is_none() {
      Some(0)
    } else {
      comment_like
    };

    Ok(CommentView {
      comment,
      post,
      creator,
      community,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      subscribed: CommunityFollower::to_subscribed_type(&follower),
      saved: saved.is_some(),
      creator_blocked: creator_blocked.is_some(),
      my_vote,
    })
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentQuery<'a> {
  #[builder(!default)]
  pool: &'a DbPool,
  listing_type: Option<ListingType>,
  sort: Option<CommentSortType>,
  community_id: Option<CommunityId>,
  post_id: Option<PostId>,
  parent_path: Option<Ltree>,
  creator_id: Option<PersonId>,
  local_user: Option<&'a LocalUser>,
  search_term: Option<String>,
  saved_only: Option<bool>,
  show_deleted_and_removed: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
  max_depth: Option<i32>,
}

impl<'a> CommentQuery<'a> {
  pub async fn list(self) -> Result<Vec<CommentView>, Error> {
    let conn = &mut get_conn(self.pool).await?;

    // The left join below will return None in this case
    let person_id_join = self.local_user.map(|l| l.person_id).unwrap_or(PersonId(-1));
    let local_user_id_join = self.local_user.map(|l| l.id).unwrap_or(LocalUserId(-1));

    let mut query = comment::table
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
        community_block::table.on(
          community::id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        local_user_language::table.on(
          comment::language_id
            .eq(local_user_language::language_id)
            .and(local_user_language::local_user_id.eq(local_user_id_join)),
        ),
      )
      .select((
        comment::all_columns,
        person::all_columns,
        post::all_columns,
        community::all_columns,
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        community_follower::all_columns.nullable(),
        comment_saved::all_columns.nullable(),
        person_block::all_columns.nullable(),
        comment_like::score.nullable(),
      ))
      .into_boxed();

    if let Some(creator_id) = self.creator_id {
      query = query.filter(comment::creator_id.eq(creator_id));
    };

    if let Some(post_id) = self.post_id {
      query = query.filter(comment::post_id.eq(post_id));
    };

    if let Some(parent_path) = self.parent_path.as_ref() {
      query = query.filter(comment::path.contained_by(parent_path));
    };

    if let Some(search_term) = self.search_term {
      query = query.filter(comment::content.ilike(fuzzy_search(&search_term)));
    };

    if let Some(community_id) = self.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if let Some(listing_type) = self.listing_type {
      match listing_type {
        ListingType::Subscribed => {
          query = query.filter(community_follower::person_id.is_not_null())
        } // TODO could be this: and(community_follower::person_id.eq(person_id_join)),
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

    if self.saved_only.unwrap_or(false) {
      query = query.filter(comment_saved::comment_id.is_not_null());
    }

    if !self.show_deleted_and_removed.unwrap_or(false) {
      query = query.filter(comment::deleted.eq(false));
      query = query.filter(comment::removed.eq(false));
    }

    if !self.local_user.map(|l| l.show_bot_accounts).unwrap_or(true) {
      query = query.filter(person::bot_account.eq(false));
    };

    if self.local_user.is_some() {
      // Filter out the rows with missing languages
      query = query.filter(local_user_language::language_id.is_not_null());

      // Don't show blocked communities or persons
      if self.post_id.is_none() {
        query = query.filter(community_block::person_id.is_null());
      }
      query = query.filter(person_block::person_id.is_null());
    }

    // A Max depth given means its a tree fetch
    let (limit, offset) = if let Some(max_depth) = self.max_depth {
      let depth_limit = if let Some(parent_path) = self.parent_path.as_ref() {
        parent_path.0.split('.').count() as i32 + max_depth
        // Add one because of root "0"
      } else {
        max_depth + 1
      };

      query = query.filter(nlevel(comment::path).le(depth_limit));

      // Always order by the parent path first
      query = query.order_by(subpath(comment::path, 0, -1));

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
      // limit_and_offset_unlimited(self.page, self.limit)
      limit_and_offset(self.page, self.limit)?
    };

    query = match self.sort.unwrap_or(CommentSortType::Hot) {
      CommentSortType::Hot => query
        .then_order_by(comment_aggregates::hot_rank.desc())
        .then_order_by(comment_aggregates::score.desc()),
      CommentSortType::New => query.then_order_by(comment::published.desc()),
      CommentSortType::Old => query.then_order_by(comment::published.asc()),
      CommentSortType::Top => query.order_by(comment_aggregates::score.desc()),
    };

    // Note: deleted and removed comments are done on the front side
    let res = query
      .limit(limit)
      .offset(offset)
      .load::<CommentViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(CommentView::from_tuple).collect())
  }
}

impl JoinView for CommentView {
  type JoinTuple = CommentViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      comment: a.0,
      creator: a.1,
      post: a.2,
      community: a.3,
      counts: a.4,
      creator_banned_from_community: a.5.is_some(),
      subscribed: CommunityFollower::to_subscribed_type(&a.6),
      saved: a.7.is_some(),
      creator_blocked: a.8.is_some(),
      my_vote: a.9,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::comment_view::{
    Comment,
    CommentQuery,
    CommentSortType,
    CommentView,
    Community,
    DbPool,
    LocalUser,
    Person,
    PersonBlock,
    Post,
  };
  use lemmy_db_schema::{
    aggregates::structs::CommentAggregates,
    impls::actor_language::UNDETERMINED_ID,
    newtypes::LanguageId,
    source::{
      actor_language::LocalUserLanguage,
      comment::{CommentInsertForm, CommentLike, CommentLikeForm},
      community::CommunityInsertForm,
      instance::Instance,
      language::Language,
      local_user::LocalUserInsertForm,
      person::PersonInsertForm,
      person_block::PersonBlockForm,
      post::PostInsertForm,
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
    inserted_person: Person,
    inserted_local_user: LocalUser,
    inserted_person_2: Person,
    inserted_community: Community,
  }

  async fn init_data(pool: &DbPool) -> Data {
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

    Data {
      inserted_instance,
      inserted_comment_0,
      inserted_comment_1,
      inserted_comment_2,
      inserted_post,
      inserted_person,
      inserted_local_user,
      inserted_person_2,
      inserted_community,
    }
  }

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let data = init_data(pool).await;

    let expected_comment_view_no_person = expected_comment_view(&data, pool).await;

    let mut expected_comment_view_with_person = expected_comment_view_no_person.clone();
    expected_comment_view_with_person.my_vote = Some(1);

    let read_comment_views_no_person = CommentQuery::builder()
      .pool(pool)
      .sort(Some(CommentSortType::Old))
      .post_id(Some(data.inserted_post.id))
      .build()
      .list()
      .await
      .unwrap();

    assert_eq!(
      expected_comment_view_no_person,
      read_comment_views_no_person[0]
    );

    let read_comment_views_with_person = CommentQuery::builder()
      .pool(pool)
      .sort(Some(CommentSortType::Old))
      .post_id(Some(data.inserted_post.id))
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
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
      Some(data.inserted_person.id),
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
    let data = init_data(pool).await;

    let top_path = data.inserted_comment_0.path.clone();
    let read_comment_views_top_path = CommentQuery::builder()
      .pool(pool)
      .post_id(Some(data.inserted_post.id))
      .parent_path(Some(top_path))
      .build()
      .list()
      .await
      .unwrap();

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_child_path = CommentQuery::builder()
      .pool(pool)
      .post_id(Some(data.inserted_post.id))
      .parent_path(Some(child_path))
      .build()
      .list()
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

    let read_comment_views_top_max_depth = CommentQuery::builder()
      .pool(pool)
      .post_id(Some(data.inserted_post.id))
      .max_depth(Some(1))
      .build()
      .list()
      .await
      .unwrap();

    // Make sure a depth limited one only has the top comment
    assert_eq!(
      expected_comment_view(&data, pool).await,
      read_comment_views_top_max_depth[0]
    );
    assert_eq!(1, read_comment_views_top_max_depth.len());

    let child_path = data.inserted_comment_1.path.clone();
    let read_comment_views_parent_max_depth = CommentQuery::builder()
      .pool(pool)
      .post_id(Some(data.inserted_post.id))
      .parent_path(Some(child_path))
      .max_depth(Some(1))
      .sort(Some(CommentSortType::New))
      .build()
      .list()
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
    let data = init_data(pool).await;

    // by default, user has all languages enabled and should see all comments
    // (except from blocked user)
    let all_languages = CommentQuery::builder()
      .pool(pool)
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();
    assert_eq!(5, all_languages.len());

    // change user lang to finnish, should only show one post in finnish and one undetermined
    let finnish_id = Language::read_id_from_code(pool, Some("fi"))
      .await
      .unwrap()
      .unwrap();
    LocalUserLanguage::update(pool, vec![finnish_id], data.inserted_local_user.id)
      .await
      .unwrap();
    let finnish_comments = CommentQuery::builder()
      .pool(pool)
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
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
    LocalUserLanguage::update(pool, vec![UNDETERMINED_ID], data.inserted_local_user.id)
      .await
      .unwrap();
    let undetermined_comment = CommentQuery::builder()
      .pool(pool)
      .local_user(Some(&data.inserted_local_user))
      .build()
      .list()
      .await
      .unwrap();
    assert_eq!(1, undetermined_comment.len());

    cleanup(data, pool).await;
  }

  async fn cleanup(data: Data, pool: &DbPool) {
    CommentLike::remove(pool, data.inserted_person.id, data.inserted_comment_0.id)
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
    Person::delete(pool, data.inserted_person.id).await.unwrap();
    Person::delete(pool, data.inserted_person_2.id)
      .await
      .unwrap();
    Instance::delete(pool, data.inserted_instance.id)
      .await
      .unwrap();
  }

  async fn expected_comment_view(data: &Data, pool: &DbPool) -> CommentView {
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
        creator_id: data.inserted_person.id,
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
        id: data.inserted_person.id,
        name: "timmy".into(),
        display_name: None,
        published: data.inserted_person.published,
        avatar: None,
        actor_id: data.inserted_person.actor_id.clone(),
        local: true,
        banned: false,
        deleted: false,
        admin: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: data.inserted_person.inbox_url.clone(),
        shared_inbox_url: None,
        matrix_user_id: None,
        ban_expires: None,
        instance_id: data.inserted_instance.id,
        private_key: data.inserted_person.private_key.clone(),
        public_key: data.inserted_person.public_key.clone(),
        last_refreshed_at: data.inserted_person.last_refreshed_at,
      },
      post: Post {
        id: data.inserted_post.id,
        name: data.inserted_post.name.clone(),
        creator_id: data.inserted_person.id,
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
      },
    }
  }
}
