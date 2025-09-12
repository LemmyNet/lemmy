use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_private_instance, is_mod_or_admin_opt, update_read_comments},
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    post::{Post, PostActions, PostReadForm},
  },
  traits::Crud,
  SearchType,
};
use lemmy_db_views_community::CommunityView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  api::{GetPost, GetPostResponse},
  PostView,
};
use lemmy_db_views_search_combined::impls::SearchCombinedQuery;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn get_post(
  data: Query<GetPost>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetPostResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  let person_id = local_user_view.as_ref().map(|u| u.person.id);
  let local_user = local_user_view.as_ref().map(|l| l.local_user.clone());

  // I'd prefer fetching the post_view by a comment join, but it adds a lot of boilerplate
  let post_id = if let Some(id) = data.id {
    id
  } else if let Some(comment_id) = data.comment_id {
    Comment::read(&mut context.pool(), comment_id)
      .await?
      .post_id
  } else {
    Err(LemmyErrorType::NotFound)?
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
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    local_user.as_ref(),
    local_instance_id,
    is_mod_or_admin,
  )
  .await?;

  let post_id = post_view.post.id;
  if let Some(person_id) = person_id {
    let read_form = PostReadForm::new(post_id, person_id);
    PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

    update_read_comments(
      person_id,
      post_id,
      post_view.post.comments,
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
  .await?;

  // Fetch the cross_posts
  let cross_posts = if let Some(url) = &post_view.post.url {
    SearchCombinedQuery {
      search_term: Some(url.inner().as_str().into()),
      post_url_only: Some(true),
      type_: Some(SearchType::Posts),
      ..Default::default()
    }
    .list(&mut context.pool(), &local_user_view, &site_view.site)
    .await?
    .iter()
    // Filter map to collect posts
    .filter_map(|f| f.to_post_view())
    // Don't return this post as one of the cross_posts
    .filter(|x| x.post.id != post_id)
    .cloned()
    .collect::<Vec<PostView>>()
  } else {
    Vec::new()
  };

  // Return the jwt
  Ok(Json(GetPostResponse {
    post_view,
    community_view,
    cross_posts,
  }))
}
