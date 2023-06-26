use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{GetPost, GetPostResponse},
  utils::{
    check_private_instance,
    is_mod_or_admin_opt,
    local_user_view_from_jwt_opt,
    mark_post_as_read,
  },
};
use lemmy_db_schema::{
  aggregates::structs::{PersonPostAggregates, PersonPostAggregatesForm},
  source::{comment::Comment, local_site::LocalSite, post::Post},
  traits::Crud,
};
use lemmy_db_views::{post_view::PostQuery, structs::PostView};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPost {
  type Response = GetPostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<GetPostResponse, LemmyError> {
    let data: &GetPost = self;
    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), context).await;
    let local_site = LocalSite::read(context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    let person_id = local_user_view.as_ref().map(|u| u.person.id);

    // I'd prefer fetching the post_view by a comment join, but it adds a lot of boilerplate
    let post_id = if let Some(id) = data.id {
      id
    } else if let Some(comment_id) = data.comment_id {
      Comment::read(context.pool(), comment_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_post"))?
        .post_id
    } else {
      Err(LemmyError::from_message("couldnt_find_post"))?
    };

    // Check to see if the person is a mod or admin, to show deleted / removed
    let community_id = Post::read(context.pool(), post_id).await?.community_id;
    let is_mod_or_admin =
      is_mod_or_admin_opt(context.pool(), local_user_view.as_ref(), Some(community_id))
        .await
        .is_ok();

    let post_view = PostView::read(context.pool(), post_id, person_id, Some(is_mod_or_admin))
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_post"))?;

    // Mark the post as read
    let post_id = post_view.post.id;
    if let Some(person_id) = person_id {
      mark_post_as_read(person_id, post_id, context.pool()).await?;
    }

    // Necessary for the sidebar subscribed
    let community_view = CommunityView::read(
      context.pool(),
      community_id,
      person_id,
      Some(is_mod_or_admin),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    // Insert into PersonPostAggregates
    // to update the read_comments count
    if let Some(person_id) = person_id {
      let read_comments = post_view.counts.comments;
      let person_post_agg_form = PersonPostAggregatesForm {
        person_id,
        post_id,
        read_comments,
        ..PersonPostAggregatesForm::default()
      };
      PersonPostAggregates::upsert(context.pool(), &person_post_agg_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_post"))?;
    }

    let moderators = CommunityModeratorView::for_community(context.pool(), community_id).await?;

    // Fetch the cross_posts
    let cross_posts = if let Some(url) = &post_view.post.url {
      let mut x_posts = PostQuery::builder()
        .pool(context.pool())
        .url_search(Some(url.inner().as_str().into()))
        .build()
        .list()
        .await?;

      // Don't return this post as one of the cross_posts
      x_posts.retain(|x| x.post.id != post_id);
      x_posts
    } else {
      Vec::new()
    };

    // Return the jwt
    Ok(GetPostResponse {
      post_view,
      community_view,
      moderators,
      cross_posts,
    })
  }
}
