use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{FeaturePost, PostResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, is_admin},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModFeaturePost, ModFeaturePostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  PostFeatureType,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn feature_post(
  data: Json<FeaturePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindPost)?;

  check_community_mod_action(
    &local_user_view.person,
    orig_post.community_id,
    false,
    &mut context.pool(),
  )
  .await?;

  if data.feature_type == PostFeatureType::Local {
    is_admin(&local_user_view)?;
  }

  // Update the post
  let post_id = data.post_id;
  let new_post: PostUpdateForm = if data.feature_type == PostFeatureType::Community {
    PostUpdateForm {
      featured_community: Some(data.featured),
      ..Default::default()
    }
  } else {
    PostUpdateForm {
      featured_local: Some(data.featured),
      ..Default::default()
    }
  };
  let post = Post::update(&mut context.pool(), post_id, &new_post).await?;

  // Mod tables
  let form = ModFeaturePostForm {
    mod_person_id: local_user_view.person.id,
    post_id: data.post_id,
    featured: data.featured,
    is_featured_community: data.feature_type == PostFeatureType::Community,
  };

  ModFeaturePost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::FeaturePost(post, local_user_view.person.clone(), data.featured),
    &context,
  )
  .await?;

  build_post_response(
    &context,
    orig_post.community_id,
    &local_user_view.person,
    post_id,
  )
  .await
}
