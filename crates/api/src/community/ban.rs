use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{BanFromCommunity, BanFromCommunityResponse},
  utils::{blocking, get_local_user_view_from_jwt, is_mod_or_admin, remove_user_data_in_community},
};
use lemmy_apub::{
  activities::block::SiteOrCommunity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
};
use lemmy_db_schema::{
  source::{
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
    },
    moderator::{ModBanFromCommunity, ModBanFromCommunityForm},
    person::Person,
  },
  traits::{Bannable, Crud, Followable},
};
use lemmy_db_views_actor::structs::PersonViewSafe;
use lemmy_utils::{error::LemmyError, utils::naive_from_unix, ConnectionId};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for BanFromCommunity {
  type Response = BanFromCommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<BanFromCommunityResponse, LemmyError> {
    let data: &BanFromCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;
    let banned_person_id = data.person_id;
    let remove_data = data.remove_data.unwrap_or(false);
    let expires = data.expires.map(naive_from_unix);

    // Verify that only mods or admins can ban
    is_mod_or_admin(context.pool(), local_user_view.person.id, community_id).await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: data.community_id,
      person_id: data.person_id,
      expires: Some(expires),
    };

    let community: ApubCommunity = blocking(context.pool(), move |conn: &'_ _| {
      Community::read(conn, community_id)
    })
    .await??
    .into();
    let banned_person: ApubPerson = blocking(context.pool(), move |conn: &'_ _| {
      Person::read(conn, banned_person_id)
    })
    .await??
    .into();

    if data.ban {
      let ban = move |conn: &'_ _| CommunityPersonBan::ban(conn, &community_user_ban_form);
      blocking(context.pool(), ban)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "community_user_already_banned"))?;

      // Also unsubscribe them from the community, if they are subscribed
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id: banned_person_id,
        pending: false,
      };
      blocking(context.pool(), move |conn: &'_ _| {
        CommunityFollower::unfollow(conn, &community_follower_form)
      })
      .await?
      .ok();

      BlockUser::send(
        &SiteOrCommunity::Community(community),
        &banned_person,
        &local_user_view.person.clone().into(),
        remove_data,
        data.reason.clone(),
        expires,
        context,
      )
      .await?;
    } else {
      let unban = move |conn: &'_ _| CommunityPersonBan::unban(conn, &community_user_ban_form);
      blocking(context.pool(), unban)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "community_user_already_banned"))?;
      UndoBlockUser::send(
        &SiteOrCommunity::Community(community),
        &banned_person,
        &local_user_view.person.clone().into(),
        data.reason.clone(),
        context,
      )
      .await?;
    }

    // Remove/Restore their data if that's desired
    if remove_data {
      remove_user_data_in_community(community_id, banned_person_id, context.pool()).await?;
    }

    // Mod tables
    let form = ModBanFromCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires,
    };
    blocking(context.pool(), move |conn| {
      ModBanFromCommunity::create(conn, &form)
    })
    .await??;

    let person_id = data.person_id;
    let person_view = blocking(context.pool(), move |conn| {
      PersonViewSafe::read(conn, person_id)
    })
    .await??;

    let res = BanFromCommunityResponse {
      person_view,
      banned: data.ban,
    };

    context.chat_server().do_send(SendCommunityRoomMessage {
      op: UserOperation::BanFromCommunity,
      response: res.clone(),
      community_id,
      websocket_id,
    });

    Ok(res)
  }
}
