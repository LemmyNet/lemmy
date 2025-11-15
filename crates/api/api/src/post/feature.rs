use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, is_admin},
};
use lemmy_db_schema::{
  PostFeatureType,
  source::{
    community::Community,
    modlog::{Modlog, ModlogInsertForm},
    post::{Post, PostUpdateForm},
  },
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{FeaturePost, PostResponse};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn feature_post(
  data: Json<FeaturePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id).await?;

  let community = Community::read(&mut context.pool(), orig_post.community_id).await?;
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  if data.feature_type == PostFeatureType::Local {
    is_admin(&local_user_view)?;
  }

  // Update the post
  let post_id = data.post_id;
  let (post_form, modlog_form) = if data.feature_type == PostFeatureType::Community {
    (
      PostUpdateForm {
        featured_community: Some(data.featured),
        ..Default::default()
      },
      ModlogInsertForm::mod_feature_post_community(
        local_user_view.person.id,
        &orig_post,
        data.featured,
      ),
    )
  } else {
    (
      PostUpdateForm {
        featured_local: Some(data.featured),
        ..Default::default()
      },
      ModlogInsertForm::admin_feature_post_site(
        local_user_view.person.id,
        &orig_post,
        data.featured,
      ),
    )
  };
  let post = Post::update(&mut context.pool(), post_id, &post_form).await?;

  // Mod tables
  Modlog::create(&mut context.pool(), &[modlog_form]).await?;

  ActivityChannel::submit_activity(
    SendActivityData::FeaturePost(post, local_user_view.person.clone(), data.featured),
    &context,
  )?;

  build_post_response(&context, orig_post.community_id, local_user_view, post_id).await
}
