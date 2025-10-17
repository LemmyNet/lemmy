use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_private_instance, is_mod_or_admin_opt},
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_modlog_combined::{
  api::{GetModlog, GetModlogResponse},
  impls::ModlogCombinedQuery,
  ModlogCombinedView,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn get_mod_log(
  data: Query<GetModlog>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetModlogResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  check_private_instance(&local_user_view, &local_site)?;

  // Only show the modlog names if:
  // You're an admin or
  // You're fetching the modlog for a single community, and you're a mod
  // (Alternatively !admin/mod)
  let hide_modlog_names = if let Some(community_id) = data.community_id {
    is_mod_or_admin_opt(
      &mut context.pool(),
      local_user_view.as_ref(),
      Some(community_id),
    )
    .await
    .is_err()
  } else {
    !local_user_view
      .as_ref()
      .map(|l| l.local_user.admin)
      .unwrap_or_default()
  };

  // Only allow mod person id filters if its not hidden
  let mod_person_id = if hide_modlog_names {
    None
  } else {
    data.mod_person_id
  };

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(ModlogCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let modlog = ModlogCombinedQuery {
    type_: data.type_,
    listing_type: data.listing_type,
    community_id: data.community_id,
    mod_person_id,
    other_person_id: data.other_person_id,
    local_user: local_user_view.as_ref().map(|u| &u.local_user),
    post_id: data.post_id,
    comment_id: data.comment_id,
    hide_modlog_names: Some(hide_modlog_names),
    cursor_data,
    page_back: data.page_back,
    limit: data.limit,
  }
  .list(&mut context.pool())
  .await?;

  let next_page = modlog.last().map(PaginationCursorBuilder::to_cursor);
  let prev_page = modlog.first().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(GetModlogResponse {
    modlog,
    next_page,
    prev_page,
  }))
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_api_utils::utils::remove_or_restore_user_data;
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      mod_log::moderator::{ModRemoveComment, ModRemovePost},
      person::{Person, PersonInsertForm},
      post::{Post, PostActions, PostInsertForm, PostLikeForm},
    },
    traits::{Crud, Likeable},
    ModlogActionType,
  };
  use lemmy_db_views_comment::CommentView;
  use lemmy_db_views_modlog_combined::{
    impls::ModlogCombinedQuery,
    ModRemoveCommentView,
    ModRemovePostView,
    ModlogCombinedView,
  };
  use lemmy_db_views_post::PostView;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_mod_remove_or_restore_data() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    // John is the mod
    let john = PersonInsertForm::test_form(instance.id, "john the modder");
    let john = Person::create(pool, &john).await?;

    let sara_form = PersonInsertForm::test_form(instance.id, "sara");
    let sara = Person::create(pool, &sara_form).await?;

    let sara_local_user_form = LocalUserInsertForm::test_form(sara.id);
    let sara_local_user = LocalUser::create(pool, &sara_local_user_form, Vec::new()).await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "mod_community crepes".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let community = Community::create(pool, &community_form).await?;

    let post_form_1 = PostInsertForm::new("A test post tubular".into(), sara.id, community.id);
    let post_1 = Post::create(pool, &post_form_1).await?;

    let post_like_form_1 = PostLikeForm::new(post_1.id, sara.id, true);
    let _post_like_1 = PostActions::like(pool, &post_like_form_1).await?;

    let post_form_2 = PostInsertForm::new("A test post radical".into(), sara.id, community.id);
    let post_2 = Post::create(pool, &post_form_2).await?;

    let comment_form_1 =
      CommentInsertForm::new(sara.id, post_1.id, "A test comment tubular".into());
    let comment_1 = Comment::create(pool, &comment_form_1, None).await?;

    let comment_like_form_1 = CommentLikeForm::new(sara.id, comment_1.id, true);
    let _comment_like_1 = CommentActions::like(pool, &comment_like_form_1).await?;

    let comment_form_2 =
      CommentInsertForm::new(sara.id, post_2.id, "A test comment radical".into());
    let _comment_2 = Comment::create(pool, &comment_form_2, None).await?;

    // Read saras post to make sure it has a like
    let post_view_1 =
      PostView::read(pool, post_1.id, Some(&sara_local_user), instance.id, false).await?;
    assert_eq!(1, post_view_1.post.score);
    assert_eq!(
      Some(true),
      post_view_1.post_actions.and_then(|pa| pa.vote_is_upvote)
    );

    // Read saras comment to make sure it has a like
    let comment_view_1 =
      CommentView::read(pool, comment_1.id, Some(&sara_local_user), instance.id).await?;
    assert_eq!(1, comment_view_1.post.score);
    assert_eq!(
      Some(true),
      comment_view_1
        .comment_actions
        .and_then(|ca| ca.vote_is_upvote)
    );

    // Remove the user data
    remove_or_restore_user_data(john.id, sara.id, true, "a remove reason", &context).await?;

    // Verify that their posts and comments are removed.
    // Posts
    let post_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemovePost),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, post_modlog.len());

    assert!(matches!(
      &post_modlog[..],
      [
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: true, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: true, .. },
          ..
        }),
      ],
    ));

    // Comments
    let comment_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemoveComment),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(2, comment_modlog.len());

    assert!(matches!(
      &comment_modlog[..],
      [
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: true, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: true, .. },
          ..
        }),
      ],
    ));

    // Verify that the likes got removed
    // post
    let post_view_1 =
      PostView::read(pool, post_1.id, Some(&sara_local_user), instance.id, false).await?;
    assert_eq!(0, post_view_1.post.score);
    assert_eq!(
      None,
      post_view_1.post_actions.and_then(|pa| pa.vote_is_upvote)
    );

    // comment
    let comment_view_1 =
      CommentView::read(pool, comment_1.id, Some(&sara_local_user), instance.id).await?;
    assert_eq!(0, comment_view_1.post.score);
    assert_eq!(
      None,
      comment_view_1
        .comment_actions
        .and_then(|ca| ca.vote_is_upvote)
    );

    // Now restore the content, and make sure it got appended
    remove_or_restore_user_data(john.id, sara.id, false, "a restore reason", &context).await?;

    // Posts
    let post_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemovePost),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, post_modlog.len());

    assert!(matches!(
      &post_modlog[..],
      [
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: false, .. },
          post: Post { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: false, .. },
          post: Post { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemovePost(ModRemovePostView {
          mod_remove_post: ModRemovePost { removed: true, .. },
          post: Post { removed: false, .. },
          ..
        }),
      ],
    ));

    // Comments
    let comment_modlog = ModlogCombinedQuery {
      type_: Some(ModlogActionType::ModRemoveComment),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(4, comment_modlog.len());

    assert!(matches!(
      &comment_modlog[..],
      [
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: false, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: false, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
        ModlogCombinedView::ModRemoveComment(ModRemoveCommentView {
          mod_remove_comment: ModRemoveComment { removed: true, .. },
          comment: Comment { removed: false, .. },
          ..
        }),
      ],
    ));

    Instance::delete(pool, instance.id).await?;

    Ok(())
  }
}
