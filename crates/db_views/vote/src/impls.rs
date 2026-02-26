use crate::{VoteView, VoteViewComment, VoteViewPost};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  newtypes::{CommentId, PostId},
  source::{comment::CommentActions, post::PostActions},
  utils::limit_fetch,
};
use lemmy_db_schema_file::{
  InstanceId,
  PersonId,
  aliases::creator_community_actions,
  joins::{creator_home_instance_actions_join, creator_local_instance_actions_join},
  schema::{comment, comment_actions, community_actions, person, post, post_actions},
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
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};

impl VoteView {
  pub async fn list_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    page_cursor: Option<PaginationCursor>,
    limit: Option<i64>,
    local_instance_id: InstanceId,
  ) -> LemmyResult<PagedResponse<Self>> {
    use lemmy_db_schema::source::post::post_actions_keys as key;
    let limit = limit_fetch(limit, None)?;

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
      .filter(post_actions::vote_is_upvote.is_not_null())
      .select(VoteViewPost::as_select())
      .limit(limit)
      .into_boxed();

    // Sorting by like score
    let query = VoteViewPost::paginate(query, &page_cursor, SortDirection::Asc, pool, None)
      .await?
      .then_order_by(key::vote_is_upvote)
      // Tie breaker
      .then_order_by(key::voted_at);

    let conn = &mut get_conn(pool).await?;
    let res = query
      .load::<VoteViewPost>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_vote_response(res, limit, page_cursor)
  }

  pub async fn list_for_comment(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    page_cursor: Option<PaginationCursor>,
    limit: Option<i64>,
    local_instance_id: InstanceId,
  ) -> LemmyResult<PagedResponse<Self>> {
    use lemmy_db_schema::source::comment::comment_actions_keys as key;
    let limit = limit_fetch(limit, None)?;

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
      .filter(comment_actions::vote_is_upvote.is_not_null())
      .select(VoteViewComment::as_select())
      .limit(limit)
      .into_boxed();

    // Sorting by like score
    let query = VoteViewComment::paginate(query, &page_cursor, SortDirection::Asc, pool, None)
      .await?
      .then_order_by(key::vote_is_upvote)
      // Tie breaker
      .then_order_by(key::voted_at);

    let conn = &mut get_conn(pool).await?;
    let res = query
      .load::<VoteViewComment>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_vote_response(res, limit, page_cursor)
  }
}

// https://github.com/rust-lang/rust/issues/115590
#[expect(clippy::multiple_bound_locations)]
fn paginate_vote_response<
  #[cfg(feature = "ts-rs")] T: ts_rs::TS,
  #[cfg(not(feature = "ts-rs"))] T,
>(
  data: Vec<T>,
  limit: i64,
  page_cursor: Option<PaginationCursor>,
) -> LemmyResult<PagedResponse<VoteView>>
where
  T: PaginationCursorConversion + Serialize + for<'a> Deserialize<'a>,
  VoteView: From<T>,
{
  let res = paginate_response(data, limit, page_cursor)?;
  Ok(PagedResponse {
    items: res.items.into_iter().map(Into::into).collect(),
    next_page: res.next_page,
    prev_page: res.prev_page,
  })
}

impl PaginationCursorConversion for VoteViewPost {
  type PaginatedType = PostActions;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_multi([self.creator.id.0, self.post_id.0])
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let [creator_id, post_id] = cursor.multi()?;
    PostActions::read(pool, PostId(post_id), PersonId(creator_id)).await
  }
}

impl PaginationCursorConversion for VoteViewComment {
  type PaginatedType = CommentActions;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_multi([self.creator.id.0, self.comment_id.0])
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    let [creator_id, comment_id] = cursor.multi()?;
    CommentActions::read(pool, CommentId(comment_id), PersonId(creator_id)).await
  }
}

#[cfg(test)]
mod tests {
  use crate::VoteView;
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
      community::{Community, CommunityActions, CommunityInsertForm, CommunityPersonBanForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    traits::{Bannable, Likeable},
  };
  use lemmy_db_schema_file::InstanceId;
  use lemmy_diesel_utils::{connection::build_db_pool_for_tests, traits::Crud};
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn post_and_comment_vote_views() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;

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
    let timmy_post_vote_form = PostLikeForm::new(inserted_post.id, inserted_timmy.id, Some(true));
    PostActions::like(pool, &timmy_post_vote_form).await?;

    // Sara downvotes timmy's post
    let sara_post_vote_form = PostLikeForm::new(inserted_post.id, inserted_sara.id, Some(false));
    PostActions::like(pool, &sara_post_vote_form).await?;

    let mut expected_post_vote_views = [
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        is_upvote: false,
      },
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        is_upvote: true,
      },
    ];
    expected_post_vote_views[1].creator.post_count = 1;
    expected_post_vote_views[1].creator.comment_count = 1;

    let read_post_vote_views =
      VoteView::list_for_post(pool, inserted_post.id, None, None, InstanceId(1)).await?;
    assert_eq!(read_post_vote_views.items, expected_post_vote_views);

    // Timothy votes down his own comment
    let timmy_comment_vote_form =
      CommentLikeForm::new(inserted_comment.id, inserted_timmy.id, Some(false));
    CommentActions::like(pool, &timmy_comment_vote_form).await?;

    // Sara upvotes timmy's comment
    let sara_comment_vote_form =
      CommentLikeForm::new(inserted_comment.id, inserted_sara.id, Some(true));
    CommentActions::like(pool, &sara_comment_vote_form).await?;

    let mut expected_comment_vote_views = [
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        is_upvote: false,
      },
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned: false,
        creator_banned_from_community: false,
        is_upvote: true,
      },
    ];
    expected_comment_vote_views[0].creator.post_count = 1;
    expected_comment_vote_views[0].creator.comment_count = 1;

    let read_comment_vote_views =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None, InstanceId(1)).await?;
    assert_eq!(read_comment_vote_views.items, expected_comment_vote_views);

    // Ban timmy from that community
    let ban_timmy_form = CommunityPersonBanForm::new(inserted_community.id, inserted_timmy.id);
    CommunityActions::ban(pool, &ban_timmy_form).await?;

    // Make sure creator_banned_from_community is true
    let read_comment_vote_views_after_ban =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None, InstanceId(1)).await?;

    assert!(
      read_comment_vote_views_after_ban
        .first()
        .is_some_and(|c| c.creator_banned_from_community)
    );

    let read_post_vote_views_after_ban =
      VoteView::list_for_post(pool, inserted_post.id, None, None, InstanceId(1)).await?;

    assert!(
      read_post_vote_views_after_ban
        .get(1)
        .is_some_and(|p| p.creator_banned_from_community)
    );

    // Cleanup
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
