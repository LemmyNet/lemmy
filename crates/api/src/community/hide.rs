use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, HideCommunity},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{is_admin, local_user_view_from_jwt, sanitize_html_api_opt},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModHideCommunity, ModHideCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn hide_community(
  data: Json<HideCommunity>,
  context: Data<LemmyContext>,
) -> Result<Json<CommunityResponse>, LemmyError> {
  // Verify its a admin (only admin can hide or unhide it)
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;
  is_admin(&local_user_view)?;

  let community_form = CommunityUpdateForm {
    hidden: Some(data.hidden),
    ..Default::default()
  };

  let mod_hide_community_form = ModHideCommunityForm {
    community_id: data.community_id,
    mod_person_id: local_user_view.person.id,
    reason: sanitize_html_api_opt(&data.reason),
    hidden: Some(data.hidden),
  };

  let community_id = data.community_id;
  let community = Community::update(&mut context.pool(), community_id, &community_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateCommunityHiddenStatus)?;

  ModHideCommunity::create(&mut context.pool(), &mod_hide_community_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )
  .await?;

  build_community_response(&context, local_user_view, community_id).await
}
