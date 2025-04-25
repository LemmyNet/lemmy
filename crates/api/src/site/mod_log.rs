use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::GetModlog,
  utils::{check_community_mod_of_any_or_admin_action, check_private_instance},
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_modlog_combined::{
  impls::ModlogCombinedQuery,
  GetModlogResponse,
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

  let is_mod_or_admin = if let Some(local_user_view) = &local_user_view {
    check_community_mod_of_any_or_admin_action(local_user_view, &mut context.pool())
      .await
      .is_ok()
  } else {
    false
  };
  let hide_modlog_names = local_site.hide_modlog_mod_names && !is_mod_or_admin;

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
  use lemmy_api_common::utils::remove_or_restore_user_data;
  use lemmy_db_schema::{
    source::{
      comment::{Comment, CommentInsertForm},
      community::{Community, CommunityInsertForm},
      instance::Instance,
      mod_log::moderator::{ModRemoveComment, ModRemovePost},
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::Crud,
    ModlogActionType,
  };
  use lemmy_db_views_modlog_combined::{
    impls::ModlogCombinedQuery,
    ModRemoveCommentView,
    ModRemovePostView,
    ModlogCombinedView,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_mod_remove_or_restore_data() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let pool = &mut context.pool();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_mod = PersonInsertForm::test_form(inserted_instance.id, "modder");
    let inserted_mod = Person::create(pool, &new_mod).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "chrimbus");
    let inserted_person = Person::create(pool, &new_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "mod_community crepes".to_string(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await?;

    let post_form_1 = PostInsertForm::new(
      "A test post tubular".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post_1 = Post::create(pool, &post_form_1).await?;

    let post_form_2 = PostInsertForm::new(
      "A test post radical".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post_2 = Post::create(pool, &post_form_2).await?;

    let comment_form_1 = CommentInsertForm::new(
      inserted_person.id,
      inserted_post_1.id,
      "A test comment tubular".into(),
    );
    let _inserted_comment_1 = Comment::create(pool, &comment_form_1, None).await?;

    let comment_form_2 = CommentInsertForm::new(
      inserted_person.id,
      inserted_post_2.id,
      "A test comment radical".into(),
    );
    let _inserted_comment_2 = Comment::create(pool, &comment_form_2, None).await?;

    // Remove the user data
    remove_or_restore_user_data(
      inserted_mod.id,
      inserted_person.id,
      true,
      &Some("a remove reason".to_string()),
      &context,
    )
    .await?;

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

    // Now restore the content, and make sure it got appended
    remove_or_restore_user_data(
      inserted_mod.id,
      inserted_person.id,
      false,
      &Some("a restore reason".to_string()),
      &context,
    )
    .await?;

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

    Instance::delete(pool, inserted_instance.id).await?;

    Ok(())
  }
}
