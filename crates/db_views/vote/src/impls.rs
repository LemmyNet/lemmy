use crate::VoteView;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  aliases::creator_community_actions,
  newtypes::{CommentId, InstanceId, PaginationCursor, PersonId, PostId},
  source::{comment::CommentActions, post::PostActions},
  utils::{
    get_conn,
    limit_fetch,
    paginate,
    queries::{
      creator_banned,
      creator_home_instance_actions_join,
      creator_local_instance_actions_join,
    },
    DbPool,
  },
};
use lemmy_db_schema_file::schema::{
  comment,
  comment_actions,
  community_actions,
  person,
  post,
  post_actions,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl VoteView {
  pub fn to_post_actions_cursor(&self) -> PaginationCursor {
    // This needs a person and post
    let prefixes_and_ids = [('P', self.creator.id.0)];

    PaginationCursor::new(&prefixes_and_ids)
  }

  // TODO move this to the postactions impl soon.
  pub async fn from_post_actions_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<PostActions> {
    let [(_, person_id), (_, post_id)] = cursor.prefixes_and_ids()?;

    PostActions::read(pool, PostId(post_id), PersonId(person_id)).await
  }

  pub async fn list_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    cursor_data: Option<PostActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
    local_instance_id: InstanceId,
  ) -> LemmyResult<Vec<Self>> {
    use lemmy_db_schema::source::post::post_actions_keys as key;

    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let creator_community_actions_join = creator_community_actions.on(
      creator_community_actions
        .field(community_actions::community_id)
        .eq(post::community_id)
        .and(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(post_actions::person_id),
        ),
    );

    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    let query = post_actions::table
      .inner_join(person::table)
      .inner_join(post::table)
      .left_join(creator_community_actions_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .filter(post_actions::post_id.eq(post_id))
      .filter(post_actions::like_score.is_not_null())
      .select((
        person::all_columns,
        creator_banned(),
        creator_community_actions
          .field(community_actions::received_ban_at)
          .nullable()
          .is_not_null(),
        post_actions::like_score.assume_not_null(),
      ))
      .limit(limit)
      .into_boxed();

    // Sorting by like score
    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back)
      .then_order_by(key::like_score)
      // Tie breaker
      .then_order_by(key::liked_at);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub fn to_comment_actions_cursor(&self) -> PaginationCursor {
    // This needs a person and comment
    let prefixes_and_ids = [('P', self.creator.id.0)];

    PaginationCursor::new(&prefixes_and_ids)
  }

  pub async fn from_comment_actions_cursor(
    cursor: &PaginationCursor,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<CommentActions> {
    let [(_, person_id), (_, comment_id)] = cursor.prefixes_and_ids()?;

    CommentActions::read(pool, CommentId(comment_id), PersonId(person_id)).await
  }

  pub async fn list_for_comment(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    cursor_data: Option<CommentActions>,
    page_back: Option<bool>,
    limit: Option<i64>,
    local_instance_id: InstanceId,
  ) -> LemmyResult<Vec<Self>> {
    use lemmy_db_schema::source::comment::comment_actions_keys as key;
    let conn = &mut get_conn(pool).await?;
    let limit = limit_fetch(limit)?;

    let creator_community_actions_join = creator_community_actions.on(
      creator_community_actions
        .field(community_actions::community_id)
        .eq(post::community_id)
        .and(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(comment_actions::person_id),
        ),
    );

    let creator_local_instance_actions_join: creator_local_instance_actions_join =
      creator_local_instance_actions_join(local_instance_id);

    let query = comment_actions::table
      .inner_join(person::table)
      .inner_join(comment::table.inner_join(post::table))
      .left_join(creator_community_actions_join)
      .left_join(creator_home_instance_actions_join())
      .left_join(creator_local_instance_actions_join)
      .filter(comment_actions::comment_id.eq(comment_id))
      .filter(comment_actions::like_score.is_not_null())
      .select((
        person::all_columns,
        creator_banned(),
        creator_community_actions
          .field(community_actions::received_ban_at)
          .nullable()
          .is_not_null(),
        comment_actions::like_score.assume_not_null(),
      ))
      .limit(limit)
      .into_boxed();

    // Sorting by like score
    let paginated_query = paginate(query, SortDirection::Asc, cursor_data, None, page_back)
      .then_order_by(key::like_score)
      // Tie breaker
      .then_order_by(key::liked_at);

    paginated_query
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use crate::VoteView;
  use lemmy_db_schema::{
    newtypes::InstanceId,
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
      community::{Community, CommunityActions, CommunityInsertForm, CommunityPersonBanForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    traits::{Bannable, Crud, Likeable},
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn post_and_comment_vote_views() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "timmy_vv");

    let inserted_timmy = Person::create(pool, &new_person).await?;

    let new_person_2 = PersonInsertForm::test_form(inserted_instance.id, "sara_vv");

    let inserted_sara = Person::create(pool, &new_person_2).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "test community vv".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await?;

    let new_post = PostInsertForm::new(
      "A test post vv".into(),
      inserted_timmy.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::new(
      inserted_timmy.id,
      inserted_post.id,
      "A test comment vv".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    // Timmy upvotes his own post
    let timmy_post_vote_form = PostLikeForm::new(inserted_post.id, inserted_timmy.id, 1);
    PostActions::like(pool, &timmy_post_vote_form).await?;

    // Sara downvotes timmy's post
    let sara_post_vote_form = PostLikeForm::new(inserted_post.id, inserted_sara.id, -1);
    PostActions::like(pool, &sara_post_vote_form).await?;

    let mut expected_post_vote_views = [
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        score: -1,
      },
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        score: 1,
      },
    ];
    expected_post_vote_views[1].creator.post_count = 1;
    expected_post_vote_views[1].creator.comment_count = 1;

    let read_post_vote_views =
      VoteView::list_for_post(pool, inserted_post.id, None, None, None, InstanceId(1)).await?;
    assert_eq!(read_post_vote_views, expected_post_vote_views);

    // Timothy votes down his own comment
    let timmy_comment_vote_form = CommentLikeForm::new(inserted_timmy.id, inserted_comment.id, -1);
    CommentActions::like(pool, &timmy_comment_vote_form).await?;

    // Sara upvotes timmy's comment
    let sara_comment_vote_form = CommentLikeForm::new(inserted_sara.id, inserted_comment.id, 1);
    CommentActions::like(pool, &sara_comment_vote_form).await?;

    let mut expected_comment_vote_views = [
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        score: -1,
      },
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        score: 1,
      },
    ];
    expected_comment_vote_views[0].creator.post_count = 1;
    expected_comment_vote_views[0].creator.comment_count = 1;

    let read_comment_vote_views =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None, None, InstanceId(1))
        .await?;
    assert_eq!(read_comment_vote_views, expected_comment_vote_views);

    // Ban timmy from that community
    let ban_timmy_form = CommunityPersonBanForm::new(inserted_community.id, inserted_timmy.id);
    CommunityActions::ban(pool, &ban_timmy_form).await?;

    // Make sure creator_banned_from_community is true
    let read_comment_vote_views_after_ban =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None, None, InstanceId(1))
        .await?;

    assert!(read_comment_vote_views_after_ban
      .first()
      .is_some_and(|c| c.creator_banned_from_community));

    let read_post_vote_views_after_ban =
      VoteView::list_for_post(pool, inserted_post.id, None, None, None, InstanceId(1)).await?;

    assert!(read_post_vote_views_after_ban
      .get(1)
      .is_some_and(|p| p.creator_banned_from_community));

    // Cleanup
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
