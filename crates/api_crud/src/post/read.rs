use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetPost, GetPostResponse},
  utils::{check_private_instance, is_mod_or_admin_opt, mark_post_as_read, update_read_comments},
};
use lemmy_db_schema::{
  source::{comment::Comment, post::Post},
  traits::Crud,
};
use lemmy_db_views::{
  post_view::PostQuery,
  structs::{LocalUserView, PostView, SiteView},
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn get_post(
  data: Query<GetPost>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let person_id = local_user_view.as_ref().map(|u| u.person.id);

  // I'd prefer fetching the post_view by a comment join, but it adds a lot of boilerplate
  let post_id = if let Some(id) = data.id {
    id
  } else if let Some(comment_id) = data.comment_id {
    Comment::read(&mut context.pool(), comment_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindComment)?
      .post_id
  } else {
    Err(LemmyErrorType::CouldntFindPost)?
  };

  // Check to see if the person is a mod or admin, to show deleted / removed
  let community_id = Post::read(&mut context.pool(), post_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPost)?
    .community_id;

  let is_mod_or_admin = is_mod_or_admin_opt(
    &mut context.pool(),
    local_user_view.as_ref(),
    Some(community_id),
  )
  .await
  .is_ok();

  let local_user = local_user_view.map(|l| l.local_user);
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    local_user.as_ref(),
    is_mod_or_admin,
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindPost)?;

  let post_id = post_view.post.id;
  if let Some(person_id) = person_id {
    mark_post_as_read(person_id, post_id, &mut context.pool()).await?;

    update_read_comments(
      person_id,
      post_id,
      post_view.counts.comments,
      &mut context.pool(),
    )
    .await?;
  }

  // Necessary for the sidebar subscribed
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    local_user.as_ref(),
    is_mod_or_admin,
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindCommunity)?;

  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  // Fetch the cross_posts
  let cross_posts = if let Some(url) = &post_view.post.url {
    let mut x_posts = PostQuery {
      url_search: Some(url.inner().as_str().into()),
      local_user: local_user.as_ref(),
      ..Default::default()
    }
    .list(&local_site.site, &mut context.pool())
    .await?;

    // Don't return this post as one of the cross_posts
    x_posts.retain(|x| x.post.id != post_id);
    x_posts
  } else {
    Vec::new()
  };

  // Return the jwt
  Ok(Json(GetPostResponse {
    post_view,
    community_view,
    moderators,
    cross_posts,
  }))
}
