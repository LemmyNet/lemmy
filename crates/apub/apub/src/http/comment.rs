use super::check_community_content_fetchable;
use crate::protocol::collections::url_collection::UrlCollection;
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{HttpRequest, HttpResponse, web::Path};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{objects::comment::ApubComment, utils::functions::context_url};
use lemmy_db_schema::source::{comment::Comment, community::Community, post::Post};
use lemmy_db_schema_file::newtypes::CommentId;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  FEDERATION_CONTEXT,
  error::{LemmyErrorType, LemmyResult},
};
use serde::Deserialize;

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
  use super::*;
  use actix_web::test::TestRequest;
  use chrono::{Days, Utc};
  use lemmy_db_schema::{
    source::{comment::CommentInsertForm, community::CommunityInsertForm, post::PostInsertForm},
    test_data::TestData,
  };
  use lemmy_diesel_utils::traits::Crud;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_get_apub_comment_on_scheduled_post_is_not_found() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let data = TestData::create(&mut context.pool()).await?;

    let community = Community::create(
      &mut context.pool(),
      &CommunityInsertForm::new(data.instance.id, "acomm_sched".into(), "pubkey".into()),
    )
    .await?;

    let future_time = Utc::now().checked_add_days(Days::new(1)).expect("future");
    let scheduled_post = Post::create(
      &mut context.pool(),
      &PostInsertForm {
        scheduled_publish_time_at: Some(future_time),
        ..PostInsertForm::new("scheduled post".into(), data.person.id, community.id)
      },
    )
    .await?;

    // Insert a comment at the DB level (bypassing API guards)
    let comment = Comment::create(
      &mut context.pool(),
      &CommentInsertForm::new(
        data.person.id,
        scheduled_post.id,
        community.id,
        "comment on scheduled post".into(),
      ),
      None,
    )
    .await?;

    let info = Path::from(CommentQuery {
      comment_id: comment.id.0.to_string(),
    });
    let request = TestRequest::default().to_http_request();

    // Comment on a scheduled post must not be exposed via ActivityPub
    let result = get_comment(info, &context, &request).await;
    assert!(
      result.is_err(),
      "expected Err for comment on scheduled post, got Ok"
    );

    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_get_apub_comment_on_published_post_is_ok() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let data = TestData::create(&mut context.pool()).await?;

    let community = Community::create(
      &mut context.pool(),
      &CommunityInsertForm::new(data.instance.id, "acomm_pub".into(), "pubkey3".into()),
    )
    .await?;

    let post = Post::create(
      &mut context.pool(),
      &PostInsertForm::new("published post".into(), data.person.id, community.id),
    )
    .await?;

    let comment = Comment::create(
      &mut context.pool(),
      &CommentInsertForm::new(
        data.person.id,
        post.id,
        community.id,
        "comment on published post".into(),
      ),
      None,
    )
    .await?;

    let info = Path::from(CommentQuery {
      comment_id: comment.id.0.to_string(),
    });
    let request = TestRequest::default().to_http_request();

    // Comment on a published post must be accessible via ActivityPub
    let result = get_comment(info, &context, &request).await;
    assert!(
      result.is_ok(),
      "expected Ok for comment on published post, got Err"
    );

    data.delete(&mut context.pool()).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_get_apub_comment_context_scheduled_is_not_found() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let data = TestData::create(&mut context.pool()).await?;

    let community = Community::create(
      &mut context.pool(),
      &CommunityInsertForm::new(data.instance.id, "acomm_ctx_sched".into(), "pubkey4".into()),
    )
    .await?;

    let future_time = Utc::now().checked_add_days(Days::new(1)).expect("future");
    let scheduled_post = Post::create(
      &mut context.pool(),
      &PostInsertForm {
        scheduled_publish_time_at: Some(future_time),
        ..PostInsertForm::new("scheduled post ctx".into(), data.person.id, community.id)
      },
    )
    .await?;

    let comment = Comment::create(
      &mut context.pool(),
      &CommentInsertForm::new(
        data.person.id,
        scheduled_post.id,
        community.id,
        "comment for context test".into(),
      ),
      None,
    )
    .await?;

    let info = Path::from(CommentQuery {
      comment_id: comment.id.0.to_string(),
    });
    let request = TestRequest::default().to_http_request();

    // Context route must also return 404 when the post is scheduled
    let result = get_apub_comment_context(info, context.clone(), request).await;
    assert!(
      result.is_err(),
      "expected Err from context route for comment on scheduled post, got Ok"
    );

    data.delete(&mut context.pool()).await?;
    Ok(())
  }
}

#[derive(Deserialize)]
pub(crate) struct CommentQuery {
  comment_id: String,
}

async fn get_comment(
  info: Path<CommentQuery>,
  context: &Data<LemmyContext>,
  request: &HttpRequest,
) -> LemmyResult<ApubComment> {
  let id = CommentId(info.comment_id.parse::<i32>()?);
  // Can't use CommentView here because it excludes deleted/removed/local-only items
  let comment: ApubComment = Comment::read(&mut context.pool(), id).await?.into();
  let post = Post::read(&mut context.pool(), comment.post_id).await?;
  // AP routes are unauthenticated; never expose a comment whose post hasn't been published yet.
  if post.scheduled_publish_time_at.is_some() {
    return Err(LemmyErrorType::NotFound.into());
  }
  let community = Community::read(&mut context.pool(), post.community_id).await?;
  check_community_content_fetchable(&community, request, context).await?;
  Ok(comment)
}

/// Return the ActivityPub json representation of a local comment over HTTP.
pub(crate) async fn get_apub_comment(
  info: Path<CommentQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let comment = get_comment(info, &context, &request).await?;
  comment.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_comment_context(
  info: Path<CommentQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let comment = get_comment(info, &context, &request).await?;
  if !comment.local {
    return Err(LemmyErrorType::NotFound.into());
  }
  let post = Post::read(&mut context.pool(), comment.post_id).await?;
  UrlCollection::new_response(&post, context_url(&comment.ap_id), &context).await
}
