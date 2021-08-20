use crate::Perform;
use actix_web::web::Data;
use anyhow::Context;
use lemmy_api_common::{
  check_community_ban,
  community::*,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
};
use lemmy_apub::activities::{
  community::{
    add_mod::AddMod,
    block_user::BlockUserFromCommunity,
    remove_mod::RemoveMod,
    undo_block_user::UndoBlockUserFromCommunity,
  },
  following::{follow::FollowCommunity as FollowCommunityApub, undo::UndoFollowCommunity},
};
use lemmy_db_queries::{
  source::{comment::Comment_, community::CommunityModerator_, post::Post_},
  Bannable,
  Blockable,
  Crud,
  Followable,
  Joinable,
};
use lemmy_db_schema::source::{
  comment::Comment,
  community::*,
  community_block::{CommunityBlock, CommunityBlockForm},
  moderator::*,
  person::Person,
  post::Post,
  site::*,
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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community_id;
    let community = Community::read(&&context.pool.get().await?, community_id)?;
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    if community.local {
      if data.follow {
        check_community_ban(local_user_view.person.id, community_id, context.pool()).await?;

        let follow =
          CommunityFollower::follow(&&context.pool.get().await?, &community_follower_form);
        if follow.is_err() {
          return Err(ApiError::err("community_follower_already_exists").into());
        }
      } else {
        let unfollow =
          CommunityFollower::unfollow(&&context.pool.get().await?, &community_follower_form);
        if unfollow.is_err() {
          return Err(ApiError::err("community_follower_already_exists").into());
        }
      }
    } else if data.follow {
      // Dont actually add to the community followers here, because you need
      // to wait for the accept
      FollowCommunityApub::send(&local_user_view.person, &community, context).await?;
    } else {
      UndoFollowCommunity::send(&local_user_view.person, &community, context).await?;
      let unfollow =
        CommunityFollower::unfollow(&&context.pool.get().await?, &community_follower_form);
      if unfollow.is_err() {
        return Err(ApiError::err("community_follower_already_exists").into());
      }
    }

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let mut community_view =
      CommunityView::read(&&context.pool.get().await?, community_id, Some(person_id))?;

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_block_form = CommunityBlockForm {
      person_id,
      community_id,
    };

    if data.block {
      let block = CommunityBlock::block(&&context.pool.get().await?, &community_block_form);
      if block.is_err() {
        return Err(ApiError::err("community_block_already_exists").into());
      }

      // Also, unfollow the community, and send a federated unfollow
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id,
        pending: false,
      };
      CommunityFollower::unfollow(&&context.pool.get().await?, &community_follower_form).ok();
      let community = Community::read(&&context.pool.get().await?, community_id)?;
      UndoFollowCommunity::send(&local_user_view.person, &community, context).await?;
    } else {
      let unblock = CommunityBlock::unblock(&&context.pool.get().await?, &community_block_form);
      if unblock.is_err() {
        return Err(ApiError::err("community_block_already_exists").into());
      }
    }

    let community_view =
      CommunityView::read(&&context.pool.get().await?, community_id, Some(person_id))?;

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community_id;
    let banned_person_id = data.person_id;

    // Verify that only mods or admins can ban
    is_mod_or_admin(context.pool(), local_user_view.person.id, community_id).await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: data.community_id,
      person_id: data.person_id,
    };

    let community = Community::read(&&context.pool.get().await?, community_id)?;
    let banned_person = Person::read(&&context.pool.get().await?, banned_person_id)?;

    if data.ban {
      let ban = CommunityPersonBan::ban(&&context.pool.get().await?, &community_user_ban_form);
      if ban.is_err() {
        return Err(ApiError::err("community_user_already_banned").into());
      }

      // Also unsubscribe them from the community, if they are subscribed
      let community_follower_form = CommunityFollowerForm {
        community_id: data.community_id,
        person_id: banned_person_id,
        pending: false,
      };
      CommunityFollower::unfollow(&&context.pool.get().await?, &community_follower_form).ok();

      BlockUserFromCommunity::send(&community, &banned_person, &local_user_view.person, context)
        .await?;
    } else {
      let unban = CommunityPersonBan::unban(&&context.pool.get().await?, &community_user_ban_form);
      if unban.is_err() {
        return Err(ApiError::err("community_user_already_banned").into());
      }
      UndoBlockUserFromCommunity::send(
        &community,
        &banned_person,
        &local_user_view.person,
        context,
      )
      .await?;
    }

    // Remove/Restore their data if that's desired
    if data.remove_data.unwrap_or(false) {
      // Posts
      Post::update_removed_for_creator(
        &&context.pool.get().await?,
        banned_person_id,
        Some(community_id),
        true,
      )?;

      // Comments
      // TODO Diesel doesn't allow updates with joins, so this has to be a loop
      let comments = CommentQueryBuilder::create(&&context.pool.get().await?)
        .creator_id(banned_person_id)
        .community_id(community_id)
        .limit(std::i64::MAX)
        .list()?;

      for comment_view in &comments {
        let comment_id = comment_view.comment.id;
        Comment::update_removed(&&context.pool.get().await?, comment_id, true)?;
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
    ModBanFromCommunity::create(&&context.pool.get().await?, &form)?;

    let person_id = data.person_id;
    let person_view = PersonViewSafe::read(&&context.pool.get().await?, person_id)?;

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community_id;

    // Verify that only mods or admins can add mod
    is_mod_or_admin(context.pool(), local_user_view.person.id, community_id).await?;

    // Update in local database
    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      person_id: data.person_id,
    };
    if data.added {
      let join = CommunityModerator::join(&&context.pool.get().await?, &community_moderator_form);
      if join.is_err() {
        return Err(ApiError::err("community_moderator_already_exists").into());
      }
    } else {
      let leave = CommunityModerator::leave(&&context.pool.get().await?, &community_moderator_form);
      if leave.is_err() {
        return Err(ApiError::err("community_moderator_already_exists").into());
      }
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    ModAddCommunity::create(&&context.pool.get().await?, &form)?;

    // Send to federated instances
    let updated_mod_id = data.person_id;
    let updated_mod = Person::read(&&context.pool.get().await?, updated_mod_id)?;
    let community = Community::read(&&context.pool.get().await?, community_id)?;
    if data.added {
      AddMod::send(&community, &updated_mod, &local_user_view.person, context).await?;
    } else {
      RemoveMod::send(&community, &updated_mod, &local_user_view.person, context).await?;
    }

    // Note: in case a remote mod is added, this returns the old moderators list, it will only get
    //       updated once we receive an activity from the community (like `Announce/Add/Moderator`)
    let community_id = data.community_id;
    let moderators =
      CommunityModeratorView::for_community(&&context.pool.get().await?, community_id)?;

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
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let site_creator_id = Site::read(&&context.pool.get().await?, 1).map(|s| s.creator_id)?;

    let mut admins = PersonViewSafe::admins(&&context.pool.get().await?)?;

    // Making sure the site creator, if an admin, is at the top
    let creator_index = admins
      .iter()
      .position(|r| r.person.id == site_creator_id)
      .context(location_info!())?;
    let creator_person = admins.remove(creator_index);
    admins.insert(0, creator_person);

    // Fetch the community mods
    let community_id = data.community_id;
    let mut community_mods =
      CommunityModeratorView::for_community(&&context.pool.get().await?, community_id)?;

    // Make sure transferrer is either the top community mod, or an admin
    if local_user_view.person.id != community_mods[0].moderator.id
      && !admins
        .iter()
        .map(|a| a.person.id)
        .any(|x| x == local_user_view.person.id)
    {
      return Err(ApiError::err("not_an_admin").into());
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
    CommunityModerator::delete_for_community(&&context.pool.get().await?, community_id)?;

    // TODO: this should probably be a bulk operation
    // Re-add the mods, in the new order
    for cmod in &community_mods {
      let community_moderator_form = CommunityModeratorForm {
        community_id: cmod.community.id,
        person_id: cmod.moderator.id,
      };

      let join = CommunityModerator::join(&&context.pool.get().await?, &community_moderator_form);
      if join.is_err() {
        return Err(ApiError::err("community_moderator_already_exists").into());
      }
    }

    // Mod tables
    let form = ModTransferCommunityForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    ModTransferCommunity::create(&&context.pool.get().await?, &form)?;

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view =
      CommunityView::read(&&context.pool.get().await?, community_id, Some(person_id))
        .map_err(|_| ApiError::err("couldnt_find_community"))?;

    let community_id = data.community_id;
    let moderators =
      CommunityModeratorView::for_community(&&context.pool.get().await?, community_id)
        .map_err(|_| ApiError::err("couldnt_find_community"))?;

    // Return the jwt
    Ok(GetCommunityResponse {
      community_view,
      moderators,
      online: 0,
    })
  }
}
