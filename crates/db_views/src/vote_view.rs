use crate::structs::VoteView;
use diesel::{result::Error, ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases::creator_community_actions,
  newtypes::{CommentId, PostId},
  schema::{comment, comment_actions, community_actions, person, post, post_actions},
  utils::{action_query, actions_alias, get_conn, limit_and_offset, DbPool},
};

impl VoteView {
  pub async fn list_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;

    action_query(post_actions::like_score)
      .inner_join(person::table)
      .inner_join(post::table)
      .left_join(actions_alias(
        creator_community_actions,
        post_actions::person_id,
        post::community_id,
      ))
      .filter(post_actions::post_id.eq(post_id))
      .select((
        person::all_columns,
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        post_actions::like_score.assume_not_null(),
      ))
      .order_by(post_actions::like_score)
      .limit(limit)
      .offset(offset)
      .load::<Self>(conn)
      .await
  }

  pub async fn list_for_comment(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let (limit, offset) = limit_and_offset(page, limit)?;

    action_query(comment_actions::like_score)
      .inner_join(person::table)
      .inner_join(comment::table.inner_join(post::table))
      .left_join(actions_alias(
        creator_community_actions,
        comment_actions::person_id,
        post::community_id,
      ))
      .filter(comment_actions::comment_id.eq(comment_id))
      .select((
        person::all_columns,
        creator_community_actions
          .field(community_actions::received_ban)
          .nullable()
          .is_not_null(),
        comment_actions::like_score.assume_not_null(),
      ))
      .order_by(comment_actions::like_score)
      .limit(limit)
      .offset(offset)
      .load::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {

  use crate::structs::VoteView;
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentInsertForm, CommentLike, CommentLikeForm},
      community::{Community, CommunityInsertForm, CommunityPersonBan, CommunityPersonBanForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm, PostLike, PostLikeForm},
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
    PostLike::like(pool, &timmy_post_vote_form).await?;

    // Sara downvotes timmy's post
    let sara_post_vote_form = PostLikeForm::new(inserted_post.id, inserted_sara.id, -1);
    PostLike::like(pool, &sara_post_vote_form).await?;

    let expected_post_vote_views = [
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned_from_community: false,
        score: -1,
      },
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned_from_community: false,
        score: 1,
      },
    ];

    let read_post_vote_views = VoteView::list_for_post(pool, inserted_post.id, None, None).await?;
    assert_eq!(read_post_vote_views, expected_post_vote_views);

    // Timothy votes down his own comment
    let timmy_comment_vote_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: inserted_timmy.id,
      score: -1,
    };
    CommentLike::like(pool, &timmy_comment_vote_form).await?;

    // Sara upvotes timmy's comment
    let sara_comment_vote_form = CommentLikeForm {
      comment_id: inserted_comment.id,
      person_id: inserted_sara.id,
      score: 1,
    };
    CommentLike::like(pool, &sara_comment_vote_form).await?;

    let expected_comment_vote_views = [
      VoteView {
        creator: inserted_timmy.clone(),
        creator_banned_from_community: false,
        score: -1,
      },
      VoteView {
        creator: inserted_sara.clone(),
        creator_banned_from_community: false,
        score: 1,
      },
    ];

    let read_comment_vote_views =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None).await?;
    assert_eq!(read_comment_vote_views, expected_comment_vote_views);

    // Ban timmy from that community
    let ban_timmy_form = CommunityPersonBanForm {
      community_id: inserted_community.id,
      person_id: inserted_timmy.id,
      expires: None,
    };
    CommunityPersonBan::ban(pool, &ban_timmy_form).await?;

    // Make sure creator_banned_from_community is true
    let read_comment_vote_views_after_ban =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None).await?;

    assert!(read_comment_vote_views_after_ban
      .first()
      .is_some_and(|c| c.creator_banned_from_community));

    let read_post_vote_views_after_ban =
      VoteView::list_for_post(pool, inserted_post.id, None, None).await?;

    assert!(read_post_vote_views_after_ban
      .get(1)
      .is_some_and(|p| p.creator_banned_from_community));

    // Cleanup
    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
