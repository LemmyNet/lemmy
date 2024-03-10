use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetPost, GetPostResponse},
  utils::{check_private_instance, is_mod_or_admin_opt, mark_post_as_read},
};
use lemmy_db_schema::{
  aggregates::structs::{PersonPostAggregates, PersonPostAggregatesForm},
  source::{comment::Comment, post::Post},
  traits::Crud,
};
use lemmy_db_views::{
  post_view::PostQuery,
  structs::{LocalUserView, PostView, SiteView},
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn get_post(
  data: Query<GetPost>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> Result<Json<GetPostResponse>, LemmyError> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site.local_site)?;

  let person_id = local_user_view.as_ref().map(|u| u.person.id);

  // I'd prefer fetching the post_view by a comment join, but it adds a lot of boilerplate
  let post_id = if let Some(id) = data.id {
    id
  } else if let Some(comment_id) = data.comment_id {
    Comment::read(&mut context.pool(), comment_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntFindPost)?
      .post_id
  } else {
    Err(LemmyErrorType::CouldntFindPost)?
  };

  // Check to see if the person is a mod or admin, to show deleted / removed
  let community_id = Post::read(&mut context.pool(), post_id).await?.community_id;
  let is_mod_or_admin = is_mod_or_admin_opt(
    &mut context.pool(),
    local_user_view.as_ref(),
    Some(community_id),
  )
  .await
  .is_ok();

  let post_view = PostView::read(&mut context.pool(), post_id, person_id, is_mod_or_admin)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindPost)?;

  // Mark the post as read
  let post_id = post_view.post.id;
  if let Some(person_id) = person_id {
    mark_post_as_read(person_id, post_id, &mut context.pool()).await?;
  }

  // Necessary for the sidebar subscribed
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    person_id,
    is_mod_or_admin,
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntFindCommunity)?;

  // Insert into PersonPostAggregates
  // to update the read_comments count
  if let Some(person_id) = person_id {
    let read_comments = post_view.counts.comments;
    let person_post_agg_form = PersonPostAggregatesForm {
      person_id,
      post_id,
      read_comments,
    };
    PersonPostAggregates::upsert(&mut context.pool(), &person_post_agg_form)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntFindPost)?;
  }

  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  // Fetch the cross_posts
  let cross_posts = if let Some(url) = &post_view.post.url {
    let mut x_posts = PostQuery {
      url_search: Some(url.inner().as_str().into()),
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
