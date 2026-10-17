use super::check_community_content_fetchable;
use crate::protocol::collections::url_collection::UrlCollection;
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{HttpRequest, HttpResponse, web};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{objects::post::ApubPost, utils::functions::context_url};
use lemmy_db_schema::source::{community::Community, post::Post};
use lemmy_db_schema_file::newtypes::PostId;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  FEDERATION_CONTEXT,
  error::{LemmyErrorType, LemmyResult},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PostQuery {
  post_id: String,
}

async fn get_post(
  info: web::Path<PostQuery>,
  context: &Data<LemmyContext>,
  request: &HttpRequest,
) -> LemmyResult<ApubPost> {
  let id = PostId(info.post_id.parse::<i32>()?);
  // Can't use PostView here because it excludes deleted/removed/local-only items
  let post: ApubPost = Post::read(&mut context.pool(), id).await?.into();
  // AP routes are unauthenticated; never expose a post that hasn't been published yet.
  if post.scheduled_publish_time_at.is_some() {
    return Err(LemmyErrorType::NotFound.into());
  }
  let community = Community::read(&mut context.pool(), post.community_id).await?;

  check_community_content_fetchable(&community, request, context).await?;

  Ok(post)
}

/// Return the ActivityPub json representation of a local post over HTTP.
pub(crate) async fn get_apub_post(
  info: web::Path<PostQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let post = get_post(info, &context, &request).await?;
  post.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_post_context(
  info: web::Path<PostQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let post = get_post(info, &context, &request).await?;
  if !post.local {
    return Err(LemmyErrorType::NotFound.into());
  }
  UrlCollection::new_response(&post, context_url(&post.ap_id), &context).await
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
  use super::*;
  use actix_web::test::TestRequest;
  use chrono::{Days, Utc};
  use lemmy_db_schema::{
    source::{
      community::CommunityInsertForm,
      post::{PostInsertForm, PostUpdateForm},
    },
    test_data::TestData,
  };
  use lemmy_diesel_utils::traits::Crud;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_get_apub_scheduled_then_published() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let data = TestData::create(&mut context.pool()).await?;

    let community = Community::create(
      &mut context.pool(),
      &CommunityInsertForm::new(data.instance.id, "apost_sched".into(), "pubkey".into()),
    )
    .await?;

    let future_time = Utc::now().checked_add_days(Days::new(1)).expect("future");
    let post = Post::create(
      &mut context.pool(),
      &PostInsertForm {
        scheduled_publish_time_at: Some(future_time),
        ..PostInsertForm::new("scheduled".into(), data.person.id, community.id)
      },
    )
    .await?;

    let post_id = post.id.0.to_string();
    let request = TestRequest::default().to_http_request();

    // Scheduled post must not be exposed via ActivityPub
    let info = web::Path::from(PostQuery {
      post_id: post_id.clone(),
    });
    let result = get_post(info, &context, &request).await;
    assert!(result.is_err(), "expected Err for scheduled post, got Ok");

    // Context route must also return 404 for a scheduled post
    let info = web::Path::from(PostQuery {
      post_id: post_id.clone(),
    });
    let result = get_apub_post_context(info, context.clone(), request.clone()).await;
    assert!(
      result.is_err(),
      "expected Err from context route for scheduled post, got Ok"
    );

    // Update post to be published (clear scheduled_publish_time_at)
    Post::update(
      &mut context.pool(),
      post.id,
      &PostUpdateForm {
        scheduled_publish_time_at: Some(None),
        ..Default::default()
      },
    )
    .await?;

    // Published post must be accessible via ActivityPub
    let info = web::Path::from(PostQuery {
      post_id: post_id.clone(),
    });
    let result = get_post(info, &context, &request).await;
    assert!(result.is_ok(), "expected Ok for published post, got Err");

    // Context route must also work after publishing
    let info = web::Path::from(PostQuery { post_id });
    let result = get_apub_post_context(info, context.clone(), request).await;
    assert!(
      result.is_ok(),
      "expected Ok from context route for published post, got Err"
    );

    data.delete(&mut context.pool()).await?;
    Ok(())
  }
}
