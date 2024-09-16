use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::HideCommunity,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModHideCommunity, ModHideCommunityForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn hide_community(
  data: Json<HideCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Verify its a admin (only admin can hide or unhide it)
  is_admin(&local_user_view)?;

  let community_form = CommunityUpdateForm {
    hidden: Some(data.hidden),
    ..Default::default()
  };

  let mod_hide_community_form = ModHideCommunityForm {
    community_id: data.community_id,
    mod_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
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

  Ok(Json(SuccessResponse::default()))
}
