use crate::Perform;
use actix_web::web::Data;
use anyhow::Context;
use lemmy_api_common::{
  check_community_ban,
  check_community_deleted_or_removed,
  community::*,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
};
use lemmy_apub::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::{
    community::{
      add_mod::AddMod,
      block_user::BlockUserFromCommunity,
      remove_mod::RemoveMod,
      undo_block_user::UndoBlockUserFromCommunity,
    },
    following::{follow::FollowCommunity as FollowCommunityApub, undo_follow::UndoFollowCommunity},
  },
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
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
    post::Post,
    site::Site,
  },
  traits::{Bannable, Blockable, Crud, Followable, Joinable},
};
use lemmy_db_views::comment_view::CommentQueryBuilder;
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
  person_view::PersonViewSafe,
};
use lemmy_utils::{location_info, utils::naive_from_unix, ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::SendCommunityRoomMessage, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for FollowCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &FollowCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let community_id = data.community_id;
    let community: ApubCommunity = context
      .conn()
      .await?
      .interact(move |conn| Community::read(conn, community_id))
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

        context
          .conn()
          .await?
          .interact(move |conn| CommunityFollower::follow(conn, &community_follower_form))
          .await?
          .map_err(|e| ApiError::err("community_follower_already_exists", e))?;
      } else {
        context
          .conn()
          .await?
          .interact(move |conn| CommunityFollower::unfollow(conn, &community_follower_form))
          .await?
          .map_err(|e| ApiError::err("community_follower_already_exists", e))?;
      }
    } else if data.follow {
      // Dont actually add to the community followers here, because you need
      // to wait for the accept
      FollowCommunityApub::send(&local_user_view.person.clone().into(), &community, context)
        .await?;
    } else {
      UndoFollowCommunity::send(&local_user_view.person.clone().into(), &community, context)
        .await?;
      context
        .conn()
        .await?
        .interact(move |conn| CommunityFollower::unfollow(conn, &community_follower_form))
        .await?
        .map_err(|e| ApiError::err("community_follower_already_exists", e))?;
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let mut community_view = context
      .conn()
      .await?
      .interact(move |conn| CommunityView::read(conn, community_id, Some(person_id)))
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
      context
        .conn()
        .await?
        .interact(move |conn| CommunityBlock::block(conn, &community_block_form))
        .await?
        .map_err(|e| ApiError::err("community_block_already_exists", e))?;

      // Also, unfollow the community, and send a federated unfollow
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id,
        pending: false,
      };
      context
        .conn()
        .await?
        .interact(move |conn| CommunityFollower::unfollow(conn, &community_follower_form))
        .await?
        .ok();
      let community = context
        .conn()
        .await?
        .interact(move |conn| Community::read(conn, community_id))
        .await??;
      UndoFollowCommunity::send(&local_user_view.person.into(), &community.into(), context).await?;
    } else {
      context
        .conn()
        .await?
        .interact(move |conn| CommunityBlock::unblock(conn, &community_block_form))
        .await?
        .map_err(|e| ApiError::err("community_block_already_exists", e))?;
    }

    let community_view = context
      .conn()
      .await?
      .interact(move |conn| CommunityView::read(conn, community_id, Some(person_id)))
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

    // Verify that only mods or admins can ban
    is_mod_or_admin(context.pool(), local_user_view.person.id, community_id).await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: data.community_id,
      person_id: data.person_id,
    };

    let community: ApubCommunity = context
      .conn()
      .await?
      .interact(move |conn| Community::read(conn, community_id))
      .await??
      .into();
    let banned_person: ApubPerson = context
      .conn()
      .await?
      .interact(move |conn| Person::read(conn, banned_person_id))
      .await??
      .into();

    if data.ban {
      context
        .conn()
        .await?
        .interact(move |conn| CommunityPersonBan::ban(conn, &community_user_ban_form))
        .await?
        .map_err(|e| ApiError::err("community_user_already_banned", e))?;

      // Also unsubscribe them from the community, if they are subscribed
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id: banned_person_id,
        pending: false,
      };
      context
        .conn()
        .await?
        .interact(move |conn| CommunityFollower::unfollow(conn, &community_follower_form))
        .await?
        .ok();

      BlockUserFromCommunity::send(
        &community,
        &banned_person,
        &local_user_view.person.clone().into(),
        context,
      )
      .await?;
    } else {
      context
        .conn()
        .await?
        .interact(move |conn| CommunityPersonBan::unban(conn, &community_user_ban_form))
        .await?
        .map_err(|e| ApiError::err("community_user_already_banned", e))?;
      UndoBlockUserFromCommunity::send(
        &community,
        &banned_person,
        &local_user_view.person.clone().into(),
        context,
      )
      .await?;
    }

    // Remove/Restore their data if that's desired
    if data.remove_data.unwrap_or(false) {
      // Posts
      context
        .conn()
        .await?
        .interact(move |conn| {
          Post::update_removed_for_creator(conn, banned_person_id, Some(community_id), true)
        })
        .await??;

      // Comments
      // TODO Diesel doesn't allow updates with joins, so this has to be a loop
      let comments = context
        .conn()
        .await?
        .interact(move |conn| {
          CommentQueryBuilder::create(conn)
            .creator_id(banned_person_id)
            .community_id(community_id)
            .limit(std::i64::MAX)
            .list()
        })
        .await??;

      for comment_view in &comments {
        let comment_id = comment_view.comment.id;
        context
          .conn()
          .await?
          .interact(move |conn| Comment::update_removed(conn, comment_id, true))
          .await??;
      }
    }

    // Mod tables
    // TODO eventually do correct expires
    let expires = data.expires.map(naive_from_unix);

    let form = ModBanFromCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires,
    };
    context
      .conn()
      .await?
      .interact(move |conn| ModBanFromCommunity::create(conn, &form))
      .await??;

    let person_id = data.person_id;
    let person_view = context
      .conn()
      .await?
      .interact(move |conn| PersonViewSafe::read(conn, person_id))
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

    // Update in local database
    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      person_id: data.person_id,
    };
    if data.added {
      context
        .conn()
        .await?
        .interact(move |conn| CommunityModerator::join(conn, &community_moderator_form))
        .await?
        .map_err(|e| ApiError::err("community_moderator_already_exists", e))?;
    } else {
      context
        .conn()
        .await?
        .interact(move |conn| CommunityModerator::leave(conn, &community_moderator_form))
        .await?
        .map_err(|e| ApiError::err("community_moderator_already_exists", e))?;
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    context
      .conn()
      .await?
      .interact(move |conn| ModAddCommunity::create(conn, &form))
      .await??;

    // Send to federated instances
    let updated_mod_id = data.person_id;
    let updated_mod: ApubPerson = context
      .conn()
      .await?
      .interact(move |conn| Person::read(conn, updated_mod_id))
      .await??
      .into();
    let community: ApubCommunity = context
      .conn()
      .await?
      .interact(move |conn| Community::read(conn, community_id))
      .await??
      .into();
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
    let moderators = context
      .conn()
      .await?
      .interact(move |conn| CommunityModeratorView::for_community(conn, community_id))
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

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &TransferCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let site_creator_id = context
      .conn()
      .await?
      .interact(move |conn| Site::read(conn, 1).map(|s| s.creator_id))
      .await??;

    let mut admins = context
      .conn()
      .await?
      .interact(|conn| PersonViewSafe::admins(conn))
      .await??;

    // Making sure the site creator, if an admin, is at the top
    let creator_index = admins
      .iter()
      .position(|r| r.person.id == site_creator_id)
      .context(location_info!())?;
    let creator_person = admins.remove(creator_index);
    admins.insert(0, creator_person);

    // Fetch the community mods
    let community_id = data.community_id;
    let mut community_mods = context
      .conn()
      .await?
      .interact(move |conn| CommunityModeratorView::for_community(conn, community_id))
      .await??;

    // Make sure transferrer is either the top community mod, or an admin
    if local_user_view.person.id != community_mods[0].moderator.id
      && !admins
        .iter()
        .map(|a| a.person.id)
        .any(|x| x == local_user_view.person.id)
    {
      return Err(ApiError::err_plain("not_an_admin").into());
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
    context
      .conn()
      .await?
      .interact(move |conn| CommunityModerator::delete_for_community(conn, community_id))
      .await??;

    // TODO: this should probably be a bulk operation
    // Re-add the mods, in the new order
    for cmod in &community_mods {
      let community_moderator_form = CommunityModeratorForm {
        community_id: cmod.community.id,
        person_id: cmod.moderator.id,
      };

      context
        .conn()
        .await?
        .interact(move |conn| CommunityModerator::join(conn, &community_moderator_form))
        .await?
        .map_err(|e| ApiError::err("community_moderator_already_exists", e))?;
    }

    // Mod tables
    let form = ModTransferCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    context
      .conn()
      .await?
      .interact(move |conn| ModTransferCommunity::create(conn, &form))
      .await??;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view = context
      .conn()
      .await?
      .interact(move |conn| CommunityView::read(conn, community_id, Some(person_id)))
      .await?
      .map_err(|e| ApiError::err("couldnt_find_community", e))?;

    let community_id = data.community_id;
    let moderators = context
      .conn()
      .await?
      .interact(move |conn| CommunityModeratorView::for_community(conn, community_id))
      .await?
      .map_err(|e| ApiError::err("couldnt_find_community", e))?;

    // Return the jwt
    Ok(GetCommunityResponse {
      community_view,
      moderators,
      online: 0,
    })
  }
}
