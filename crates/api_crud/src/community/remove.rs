use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, RemoveCommunity},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, is_admin},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModRemoveCommunity, ModRemoveCommunityForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn remove_community(
  data: Json<RemoveCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  check_community_mod_action(
    &local_user_view.person,
    data.community_id,
    true,
    &mut context.pool(),
  )
  .await?;

  // Verify its an admin (only an admin can remove a community)
  is_admin(&local_user_view)?;

  // Do the remove
  let community_id = data.community_id;
  let removed = data.removed;
  let community = Community::update(
    &mut context.pool(),
    community_id,
    &CommunityUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateCommunity)?;

  // Mod tables
  let form = ModRemoveCommunityForm {
    mod_person_id: local_user_view.person.id,
    community_id: data.community_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  ModRemoveCommunity::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveCommunity {
      moderator: local_user_view.person.clone(),
      community,
      reason: data.reason.clone(),
      removed: data.removed,
    },
    &context,
  )
  .await?;

  build_community_response(&context, local_user_view, community_id).await
}
