use crate::limit_ban_term;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::{BanFromCommunity, BanFromCommunityResponse},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{is_mod_or_admin, remove_user_data_in_community},
};
use lemmy_db_schema::{
  source::{
    community::{
      CommunityFollower,
      CommunityFollowerForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
    },
    moderator::{ModBanFromCommunity, ModBanFromCommunityForm},
  },
  traits::{Bannable, Crud, Followable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{time::naive_from_unix, validation::is_valid_body_field},
};

#[tracing::instrument(skip(context))]
pub async fn ban_from_community(
  data: Json<BanFromCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<BanFromCommunityResponse>, LemmyError> {
  let banned_person_id = data.person_id;
  let remove_data = data.remove_data.unwrap_or(false);
  let mut expires = data.expires.map(naive_from_unix);
  if let Some(e) = expires {
    expires = limit_ban_term(e)?;
  }

  // Verify that only mods or admins can ban
  is_mod_or_admin(
    &mut context.pool(),
    local_user_view.person.id,
    data.community_id,
  )
  .await?;
  is_valid_body_field(&data.reason, false)?;

  let community_user_ban_form = CommunityPersonBanForm {
    community_id: data.community_id,
    person_id: data.person_id,
    expires: Some(expires),
  };

  if data.ban {
    CommunityPersonBan::ban(&mut context.pool(), &community_user_ban_form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityUserAlreadyBanned)?;

    // Also unsubscribe them from the community, if they are subscribed
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      person_id: banned_person_id,
      pending: false,
    };

    CommunityFollower::unfollow(&mut context.pool(), &community_follower_form)
      .await
      .ok();
  } else {
    CommunityPersonBan::unban(&mut context.pool(), &community_user_ban_form)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityUserAlreadyBanned)?;
  }

  // Remove/Restore their data if that's desired
  if remove_data {
    remove_user_data_in_community(data.community_id, banned_person_id, &mut context.pool()).await?;
  }

  // Mod tables
  let form = ModBanFromCommunityForm {
    mod_person_id: local_user_view.person.id,
    other_person_id: data.person_id,
    community_id: data.community_id,
    reason: data.reason.clone(),
    banned: Some(data.ban),
    expires,
  };

  ModBanFromCommunity::create(&mut context.pool(), &form).await?;

  let person_view = PersonView::read(&mut context.pool(), data.person_id).await?;

  ActivityChannel::submit_activity(
    SendActivityData::BanFromCommunity(
      local_user_view.person,
      data.community_id,
      person_view.person.clone(),
      data.0.clone(),
    ),
    &context,
  )
  .await?;

  Ok(Json(BanFromCommunityResponse {
    person_view,
    banned: data.ban,
  }))
}
