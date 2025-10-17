use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, is_admin},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    modlog::{Modlog, ModlogInsertForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
  PostFeatureType,
};
use lemmy_db_schema_file::enums::ModlogKind;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{FeaturePost, PostResponse};
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
  let (post_form, modlog_kind) = if data.feature_type == PostFeatureType::Community {
    (
      PostUpdateForm {
        featured_community: Some(data.featured),
        ..Default::default()
      },
      ModlogKind::ModFeaturePostCommunity,
    )
  } else {
    (
      PostUpdateForm {
        featured_local: Some(data.featured),
        ..Default::default()
      },
      ModlogKind::AdminFeaturePostSite,
    )
  };
  let post = Post::update(&mut context.pool(), post_id, &post_form).await?;

  // Mod tables
  let form = ModlogInsertForm {
    target_post_id: Some(data.post_id),
    target_community_id: Some(community.id),
    ..ModlogInsertForm::new(modlog_kind, data.featured, local_user_view.person.id)
  };
  Modlog::create(&mut context.pool(), &[form]).await?;

  ActivityChannel::submit_activity(
    SendActivityData::FeaturePost(post, local_user_view.person.clone(), data.featured),
    &context,
  )?;

  build_post_response(&context, orig_post.community_id, local_user_view, post_id).await
}
