use crate::structs::VoteView;
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommentId, PostId},
  schema::{comment_like, community_person_ban, person, post, post_like},
  utils::{get_conn, limit_and_offset, DbPool},
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

    post_like::table
      .inner_join(person::table)
      .inner_join(post::table)
      // Join to community_person_ban to get creator_banned_from_community
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post_like::person_id)),
        ),
      )
      .filter(post_like::post_id.eq(post_id))
      .select((
        person::all_columns,
        community_person_ban::community_id.nullable().is_not_null(),
        post_like::score,
      ))
      .order_by(post_like::score)
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

    comment_like::table
      .inner_join(person::table)
      .inner_join(post::table)
      // Join to community_person_ban to get creator_banned_from_community
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment_like::person_id)),
        ),
      )
      .filter(comment_like::comment_id.eq(comment_id))
      .select((
        person::all_columns,
        community_person_ban::community_id.nullable().is_not_null(),
        comment_like::score,
      ))
      .order_by(comment_like::score)
      .limit(limit)
      .offset(offset)
      .load::<Self>(conn)
      .await
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
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
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn post_and_comment_vote_views() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::new_local("timmy_vv", inserted_instance.id);

    let inserted_timmy = Person::create(pool, &new_person).await.unwrap();

    let new_person_2 = PersonInsertForm::new_local("sara_vv", inserted_instance.id);

    let inserted_sara = Person::create(pool, &new_person_2).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community vv".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post vv".into())
      .creator_id(inserted_timmy.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let comment_form = CommentInsertForm::builder()
      .content("A test comment vv".into())
      .creator_id(inserted_timmy.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await.unwrap();

    // Timmy upvotes his own post
    let timmy_post_vote_form = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_timmy.id,
      score: 1,
    };
    PostLike::like(pool, &timmy_post_vote_form).await.unwrap();

    // Sara downvotes timmy's post
    let sara_post_vote_form = PostLikeForm {
      post_id: inserted_post.id,
      person_id: inserted_sara.id,
      score: -1,
    };
    PostLike::like(pool, &sara_post_vote_form).await.unwrap();

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

    let read_post_vote_views = VoteView::list_for_post(pool, inserted_post.id, None, None)
      .await
      .unwrap();
    assert_eq!(read_post_vote_views, expected_post_vote_views);

    // Timothy votes down his own comment
    let timmy_comment_vote_form = CommentLikeForm {
      post_id: inserted_post.id,
      comment_id: inserted_comment.id,
      person_id: inserted_timmy.id,
      score: -1,
    };
    CommentLike::like(pool, &timmy_comment_vote_form)
      .await
      .unwrap();

    // Sara upvotes timmy's comment
    let sara_comment_vote_form = CommentLikeForm {
      post_id: inserted_post.id,
      comment_id: inserted_comment.id,
      person_id: inserted_sara.id,
      score: 1,
    };
    CommentLike::like(pool, &sara_comment_vote_form)
      .await
      .unwrap();

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

    let read_comment_vote_views = VoteView::list_for_comment(pool, inserted_comment.id, None, None)
      .await
      .unwrap();
    assert_eq!(read_comment_vote_views, expected_comment_vote_views);

    // Ban timmy from that community
    let ban_timmy_form = CommunityPersonBanForm {
      community_id: inserted_community.id,
      person_id: inserted_timmy.id,
      expires: None,
    };
    CommunityPersonBan::ban(pool, &ban_timmy_form)
      .await
      .unwrap();

    // Make sure creator_banned_from_community is true
    let read_comment_vote_views_after_ban =
      VoteView::list_for_comment(pool, inserted_comment.id, None, None)
        .await
        .unwrap();

    assert!(
      read_comment_vote_views_after_ban
        .first()
        .unwrap()
        .creator_banned_from_community
    );

    let read_post_vote_views_after_ban =
      VoteView::list_for_post(pool, inserted_post.id, None, None)
        .await
        .unwrap();

    assert!(
      read_post_vote_views_after_ban
        .get(1)
        .unwrap()
        .creator_banned_from_community
    );

    // Cleanup
    Instance::delete(pool, inserted_instance.id).await.unwrap();
  }
}
