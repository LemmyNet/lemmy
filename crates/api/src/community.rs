use crate::Perform;
use actix_web::web::Data;
use anyhow::Context;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  check_community_deleted_or_removed,
  community::*,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
  remove_user_data_in_community,
};
use lemmy_apub::{
  activities::block::SiteOrCommunity,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::{
    block::{block_user::BlockUser, undo_block_user::UndoBlockUser},
    community::{add_mod::AddMod, remove_mod::RemoveMod},
    following::{follow::FollowCommunity as FollowCommunityApub, undo_follow::UndoFollowCommunity},
  },
};
use lemmy_db_schema::{
  source::{
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
      CommunityModerator,
      CommunityModeratorForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
    },
    community_block::{CommunityBlock, CommunityBlockForm},
    moderator::{
      ModAddCommunity,
      ModAddCommunityForm,
      ModBanFromCommunity,
      ModBanFromCommunityForm,
      ModTransferCommunity,
      ModTransferCommunityForm,
    },
    person::Person,
  },
  traits::{Bannable, Blockable, Crud, Followable, Joinable},
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
  person_view::PersonViewSafe,
};
use lemmy_utils::{location_info, utils::naive_from_unix, ConnectionId, LemmyError};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for FollowCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &FollowCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;
    let community: ApubCommunity = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    if community.local {
      if data.follow {
        check_community_ban(local_user_view.person.id, community_id, context.pool()).await?;
        check_community_deleted_or_removed(community_id, context.pool()).await?;

        let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
        blocking(context.pool(), follow)
          .await?
          .map_err(LemmyError::from)
          .map_err(|e| e.with_message("community_follower_already_exists"))?;
      } else {
        let unfollow =
          move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
        blocking(context.pool(), unfollow)
          .await?
          .map_err(LemmyError::from)
          .map_err(|e| e.with_message("community_follower_already_exists"))?;
      }
    } else if data.follow {
      // Dont actually add to the community followers here, because you need
      // to wait for the accept
      FollowCommunityApub::send(&local_user_view.person.clone().into(), &community, context)
        .await?;
    } else {
      UndoFollowCommunity::send(&local_user_view.person.clone().into(), &community, context)
        .await?;
      let unfollow = move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
      blocking(context.pool(), unfollow)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_follower_already_exists"))?;
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let mut community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await??;

    // TODO: this needs to return a "pending" state, until Accept is received from the remote server
    // For now, just assume that remote follows are accepted.
    // Otherwise, the subscribed will be null
    if !community.local {
      community_view.subscribed = data.follow;
    }

    Ok(CommunityResponse { community_view })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for BlockCommunity {
  type Response = BlockCommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<BlockCommunityResponse, LemmyError> {
    let data: &BlockCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_block_form = CommunityBlockForm {
      person_id,
      community_id,
    };

    if data.block {
      let block = move |conn: &'_ _| CommunityBlock::block(conn, &community_block_form);
      blocking(context.pool(), block)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_block_already_exists"))?;

      // Also, unfollow the community, and send a federated unfollow
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id,
        pending: false,
      };
      blocking(context.pool(), move |conn: &'_ _| {
        CommunityFollower::unfollow(conn, &community_follower_form)
      })
      .await?
      .ok();
      let community = blocking(context.pool(), move |conn| {
        Community::read(conn, community_id)
      })
      .await??;
      UndoFollowCommunity::send(&local_user_view.person.into(), &community.into(), context).await?;
    } else {
      let unblock = move |conn: &'_ _| CommunityBlock::unblock(conn, &community_block_form);
      blocking(context.pool(), unblock)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_block_already_exists"))?;
    }

    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await??;

    Ok(BlockCommunityResponse {
      blocked: data.block,
      community_view,
    })
  }
}

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
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_user_already_banned"))?;

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
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_user_already_banned"))?;
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

#[async_trait::async_trait(?Send)]
impl Perform for AddModToCommunity {
  type Response = AddModToCommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<AddModToCommunityResponse, LemmyError> {
    let data: &AddModToCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;

    // Verify that only mods or admins can add mod
    is_mod_or_admin(context.pool(), local_user_view.person.id, community_id).await?;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;
    if local_user_view.person.admin && !community.local {
      return Err(LemmyError::from_message("not_a_moderator"));
    }

    // Update in local database
    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      person_id: data.person_id,
    };
    if data.added {
      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      blocking(context.pool(), join)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_moderator_already_exists"))?;
    } else {
      let leave = move |conn: &'_ _| CommunityModerator::leave(conn, &community_moderator_form);
      blocking(context.pool(), leave)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_moderator_already_exists"))?;
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    blocking(context.pool(), move |conn| {
      ModAddCommunity::create(conn, &form)
    })
    .await??;

    // Send to federated instances
    let updated_mod_id = data.person_id;
    let updated_mod: ApubPerson = blocking(context.pool(), move |conn| {
      Person::read(conn, updated_mod_id)
    })
    .await??
    .into();
    let community: ApubCommunity = community.into();
    if data.added {
      AddMod::send(
        &community,
        &updated_mod,
        &local_user_view.person.into(),
        context,
      )
      .await?;
    } else {
      RemoveMod::send(
        &community,
        &updated_mod,
        &local_user_view.person.into(),
        context,
      )
      .await?;
    }

    // Note: in case a remote mod is added, this returns the old moderators list, it will only get
    //       updated once we receive an activity from the community (like `Announce/Add/Moderator`)
    let community_id = data.community_id;
    let moderators = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    let res = AddModToCommunityResponse { moderators };
    context.chat_server().do_send(SendCommunityRoomMessage {
      op: UserOperation::AddModToCommunity,
      response: res.clone(),
      community_id,
      websocket_id,
    });
    Ok(res)
  }
}

// TODO: we dont do anything for federation here, it should be updated the next time the community
//       gets fetched. i hope we can get rid of the community creator role soon.
#[async_trait::async_trait(?Send)]
impl Perform for TransferCommunity {
  type Response = GetCommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &TransferCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let admins = blocking(context.pool(), PersonViewSafe::admins).await??;

    // Fetch the community mods
    let community_id = data.community_id;
    let mut community_mods = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    // Make sure transferrer is either the top community mod, or an admin
    if local_user_view.person.id != community_mods[0].moderator.id
      && !admins
        .iter()
        .map(|a| a.person.id)
        .any(|x| x == local_user_view.person.id)
    {
      return Err(LemmyError::from_message("not_an_admin"));
    }

    // You have to re-do the community_moderator table, reordering it.
    // Add the transferee to the top
    let creator_index = community_mods
      .iter()
      .position(|r| r.moderator.id == data.person_id)
      .context(location_info!())?;
    let creator_person = community_mods.remove(creator_index);
    community_mods.insert(0, creator_person);

    // Delete all the mods
    let community_id = data.community_id;
    blocking(context.pool(), move |conn| {
      CommunityModerator::delete_for_community(conn, community_id)
    })
    .await??;

    // TODO: this should probably be a bulk operation
    // Re-add the mods, in the new order
    for cmod in &community_mods {
      let community_moderator_form = CommunityModeratorForm {
        community_id: cmod.community.id,
        person_id: cmod.moderator.id,
      };

      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      blocking(context.pool(), join)
        .await?
        .map_err(LemmyError::from)
        .map_err(|e| e.with_message("community_moderator_already_exists"))?;
    }

    // Mod tables
    let form = ModTransferCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    blocking(context.pool(), move |conn| {
      ModTransferCommunity::create(conn, &form)
    })
    .await??;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_find_community"))?;

    let community_id = data.community_id;
    let moderators = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_find_community"))?;

    // Return the jwt
    Ok(GetCommunityResponse {
      community_view,
      moderators,
      online: 0,
    })
  }
}
