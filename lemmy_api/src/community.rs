use crate::{
  check_optional_url,
  get_user_from_jwt,
  get_user_from_jwt_opt,
  is_admin,
  is_mod_or_admin,
  Perform,
};
use actix_web::web::Data;
use anyhow::Context;
use lemmy_apub::ActorType;
use lemmy_db::{
  comment::Comment,
  comment_view::CommentQueryBuilder,
  community::*,
  community_view::*,
  diesel_option_overwrite,
  moderator::*,
  naive_now,
  post::Post,
  site::*,
  user_view::*,
  Bannable,
  Crud,
  Followable,
  Joinable,
  SortType,
};
use lemmy_structs::{blocking, community::*};
use lemmy_utils::{
  apub::{generate_actor_keypair, make_apub_endpoint, EndpointType},
  location_info,
  utils::{check_slurs, check_slurs_opt, is_valid_community_name, naive_from_unix},
  APIError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{
  messages::{GetCommunityUsersOnline, JoinCommunityRoom, SendCommunityRoomMessage},
  LemmyContext,
  UserOperation,
};
use std::str::FromStr;

#[async_trait::async_trait(?Send)]
impl Perform for GetCommunity {
  type Response = GetCommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &GetCommunity = &self;
    let user = get_user_from_jwt_opt(&data.auth, context.pool()).await?;
    let user_id = user.map(|u| u.id);

    let name = data.name.to_owned().unwrap_or_else(|| "main".to_string());
    let community = match data.id {
      Some(id) => blocking(context.pool(), move |conn| Community::read(conn, id)).await??,
      None => match blocking(context.pool(), move |conn| {
        Community::read_from_name(conn, &name)
      })
      .await?
      {
        Ok(community) => community,
        Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
      },
    };

    let community_id = community.id;
    let community_view = match blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, user_id)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let community_id = community.id;
    let moderators: Vec<CommunityModeratorView> = match blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await?
    {
      Ok(moderators) => moderators,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let online = context
      .chat_server()
      .send(GetCommunityUsersOnline { community_id })
      .await
      .unwrap_or(1);

    let res = GetCommunityResponse {
      community: community_view,
      moderators,
      online,
    };

    // Return the jwt
    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for CreateCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &CreateCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.name)?;
    check_slurs(&data.title)?;
    check_slurs_opt(&data.description)?;

    if !is_valid_community_name(&data.name) {
      return Err(APIError::err("invalid_community_name").into());
    }

    // Double check for duplicate community actor_ids
    let actor_id = make_apub_endpoint(EndpointType::Community, &data.name).to_string();
    let actor_id_cloned = actor_id.to_owned();
    let community_dupe = blocking(context.pool(), move |conn| {
      Community::read_from_actor_id(conn, &actor_id_cloned)
    })
    .await?;
    if community_dupe.is_ok() {
      return Err(APIError::err("community_already_exists").into());
    }

    // Check to make sure the icon and banners are urls
    let icon = diesel_option_overwrite(&data.icon);
    let banner = diesel_option_overwrite(&data.banner);

    check_optional_url(&icon)?;
    check_optional_url(&banner)?;

    // When you create a community, make sure the user becomes a moderator and a follower
    let keypair = generate_actor_keypair()?;

    let community_form = CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      icon,
      banner,
      category_id: data.category_id,
      creator_id: user.id,
      removed: None,
      deleted: None,
      nsfw: data.nsfw,
      updated: None,
      actor_id: Some(actor_id),
      local: true,
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      last_refreshed_at: None,
      published: None,
    };

    let inserted_community = match blocking(context.pool(), move |conn| {
      Community::create(conn, &community_form)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("community_already_exists").into()),
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id: user.id,
    };

    let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
    if blocking(context.pool(), join).await?.is_err() {
      return Err(APIError::err("community_moderator_already_exists").into());
    }

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: user.id,
    };

    let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
    if blocking(context.pool(), follow).await?.is_err() {
      return Err(APIError::err("community_follower_already_exists").into());
    }

    let user_id = user.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, inserted_community.id, Some(user_id))
    })
    .await??;

    Ok(CommunityResponse {
      community: community_view,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for EditCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &EditCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.title)?;
    check_slurs_opt(&data.description)?;

    // Verify its a mod (only mods can edit it)
    let edit_id = data.edit_id;
    let mods: Vec<i32> = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, edit_id)
        .map(|v| v.into_iter().map(|m| m.user_id).collect())
    })
    .await??;
    if !mods.contains(&user.id) {
      return Err(APIError::err("not_a_moderator").into());
    }

    let edit_id = data.edit_id;
    let read_community =
      blocking(context.pool(), move |conn| Community::read(conn, edit_id)).await??;

    let icon = diesel_option_overwrite(&data.icon);
    let banner = diesel_option_overwrite(&data.banner);

    check_optional_url(&icon)?;
    check_optional_url(&banner)?;

    let community_form = CommunityForm {
      name: read_community.name,
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      icon,
      banner,
      category_id: data.category_id.to_owned(),
      creator_id: read_community.creator_id,
      removed: Some(read_community.removed),
      deleted: Some(read_community.deleted),
      nsfw: data.nsfw,
      updated: Some(naive_now()),
      actor_id: Some(read_community.actor_id),
      local: read_community.local,
      private_key: read_community.private_key,
      public_key: read_community.public_key,
      last_refreshed_at: None,
      published: None,
    };

    let edit_id = data.edit_id;
    match blocking(context.pool(), move |conn| {
      Community::update(conn, edit_id, &community_form)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_update_community").into()),
    };

    // TODO there needs to be some kind of an apub update
    // process for communities and users

    let edit_id = data.edit_id;
    let user_id = user.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = CommunityResponse {
      community: community_view,
    };

    send_community_websocket(&res, context, websocket_id, UserOperation::EditCommunity);

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for DeleteCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &DeleteCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Verify its the creator (only a creator can delete the community)
    let edit_id = data.edit_id;
    let read_community =
      blocking(context.pool(), move |conn| Community::read(conn, edit_id)).await??;
    if read_community.creator_id != user.id {
      return Err(APIError::err("no_community_edit_allowed").into());
    }

    // Do the delete
    let edit_id = data.edit_id;
    let deleted = data.deleted;
    let updated_community = match blocking(context.pool(), move |conn| {
      Community::update_deleted(conn, edit_id, deleted)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_update_community").into()),
    };

    // Send apub messages
    if deleted {
      updated_community.send_delete(context).await?;
    } else {
      updated_community.send_undo_delete(context).await?;
    }

    let edit_id = data.edit_id;
    let user_id = user.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = CommunityResponse {
      community: community_view,
    };

    send_community_websocket(&res, context, websocket_id, UserOperation::DeleteCommunity);

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for RemoveCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &RemoveCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    // Verify its an admin (only an admin can remove a community)
    is_admin(context.pool(), user.id).await?;

    // Do the remove
    let edit_id = data.edit_id;
    let removed = data.removed;
    let updated_community = match blocking(context.pool(), move |conn| {
      Community::update_removed(conn, edit_id, removed)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_update_community").into()),
    };

    // Mod tables
    let expires = match data.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None,
    };
    let form = ModRemoveCommunityForm {
      mod_user_id: user.id,
      community_id: data.edit_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
      expires,
    };
    blocking(context.pool(), move |conn| {
      ModRemoveCommunity::create(conn, &form)
    })
    .await??;

    // Apub messages
    if removed {
      updated_community.send_remove(context).await?;
    } else {
      updated_community.send_undo_remove(context).await?;
    }

    let edit_id = data.edit_id;
    let user_id = user.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = CommunityResponse {
      community: community_view,
    };

    send_community_websocket(&res, context, websocket_id, UserOperation::RemoveCommunity);

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ListCommunities {
  type Response = ListCommunitiesResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListCommunitiesResponse, LemmyError> {
    let data: &ListCommunities = &self;
    let user = get_user_from_jwt_opt(&data.auth, context.pool()).await?;

    let user_id = match &user {
      Some(user) => Some(user.id),
      None => None,
    };

    let show_nsfw = match &user {
      Some(user) => user.show_nsfw,
      None => false,
    };

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let communities = blocking(context.pool(), move |conn| {
      CommunityQueryBuilder::create(conn)
        .sort(&sort)
        .for_user(user_id)
        .show_nsfw(show_nsfw)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    // Return the jwt
    Ok(ListCommunitiesResponse { communities })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for FollowCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &FollowCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community_id;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      user_id: user.id,
    };

    if community.local {
      if data.follow {
        let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
        if blocking(context.pool(), follow).await?.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        }
      } else {
        let unfollow =
          move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
        if blocking(context.pool(), unfollow).await?.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        }
      }
    } else if data.follow {
      // Dont actually add to the community followers here, because you need
      // to wait for the accept
      user.send_follow(&community.actor_id()?, context).await?;
    } else {
      user.send_unfollow(&community.actor_id()?, context).await?;
      let unfollow = move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
      if blocking(context.pool(), unfollow).await?.is_err() {
        return Err(APIError::err("community_follower_already_exists").into());
      }
    }

    let community_id = data.community_id;
    let user_id = user.id;
    let mut community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(user_id))
    })
    .await??;

    // TODO: this needs to return a "pending" state, until Accept is received from the remote server
    // For now, just assume that remote follows are accepted.
    // Otherwise, the subscribed will be null
    if !community.local {
      community_view.subscribed = Some(data.follow);
    }

    Ok(CommunityResponse {
      community: community_view,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetFollowedCommunities {
  type Response = GetFollowedCommunitiesResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetFollowedCommunitiesResponse, LemmyError> {
    let data: &GetFollowedCommunities = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let user_id = user.id;
    let communities = match blocking(context.pool(), move |conn| {
      CommunityFollowerView::for_user(conn, user_id)
    })
    .await?
    {
      Ok(communities) => communities,
      _ => return Err(APIError::err("system_err_login").into()),
    };

    // Return the jwt
    Ok(GetFollowedCommunitiesResponse { communities })
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
    let data: &BanFromCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community_id;
    let banned_user_id = data.user_id;

    // Verify that only mods or admins can ban
    is_mod_or_admin(context.pool(), user.id, community_id).await?;

    let community_user_ban_form = CommunityUserBanForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    if data.ban {
      let ban = move |conn: &'_ _| CommunityUserBan::ban(conn, &community_user_ban_form);
      if blocking(context.pool(), ban).await?.is_err() {
        return Err(APIError::err("community_user_already_banned").into());
      }
    } else {
      let unban = move |conn: &'_ _| CommunityUserBan::unban(conn, &community_user_ban_form);
      if blocking(context.pool(), unban).await?.is_err() {
        return Err(APIError::err("community_user_already_banned").into());
      }
    }

    // Remove/Restore their data if that's desired
    if let Some(remove_data) = data.remove_data {
      // Posts
      blocking(context.pool(), move |conn: &'_ _| {
        Post::update_removed_for_creator(conn, banned_user_id, Some(community_id), remove_data)
      })
      .await??;

      // Comments
      // Diesel doesn't allow updates with joins, so this has to be a loop
      let comments = blocking(context.pool(), move |conn| {
        CommentQueryBuilder::create(conn)
          .for_creator_id(banned_user_id)
          .for_community_id(community_id)
          .limit(std::i64::MAX)
          .list()
      })
      .await??;

      for comment in &comments {
        let comment_id = comment.id;
        blocking(context.pool(), move |conn: &'_ _| {
          Comment::update_removed(conn, comment_id, remove_data)
        })
        .await??;
      }
    }

    // Mod tables
    // TODO eventually do correct expires
    let expires = match data.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None,
    };

    let form = ModBanFromCommunityForm {
      mod_user_id: user.id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires,
    };
    blocking(context.pool(), move |conn| {
      ModBanFromCommunity::create(conn, &form)
    })
    .await??;

    let user_id = data.user_id;
    let user_view = blocking(context.pool(), move |conn| {
      UserView::get_user_secure(conn, user_id)
    })
    .await??;

    let res = BanFromCommunityResponse {
      user: user_view,
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
    let data: &AddModToCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    let community_id = data.community_id;

    // Verify that only mods or admins can add mod
    is_mod_or_admin(context.pool(), user.id, community_id).await?;

    if data.added {
      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      if blocking(context.pool(), join).await?.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    } else {
      let leave = move |conn: &'_ _| CommunityModerator::leave(conn, &community_moderator_form);
      if blocking(context.pool(), leave).await?.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user.id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    blocking(context.pool(), move |conn| {
      ModAddCommunity::create(conn, &form)
    })
    .await??;

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

#[async_trait::async_trait(?Send)]
impl Perform for TransferCommunity {
  type Response = GetCommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &TransferCommunity = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    let community_id = data.community_id;
    let read_community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let site_creator_id = blocking(context.pool(), move |conn| {
      Site::read(conn, 1).map(|s| s.creator_id)
    })
    .await??;

    let mut admins = blocking(context.pool(), move |conn| UserView::admins(conn)).await??;

    let creator_index = admins
      .iter()
      .position(|r| r.id == site_creator_id)
      .context(location_info!())?;
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    // Make sure user is the creator, or an admin
    if user.id != read_community.creator_id && !admins.iter().map(|a| a.id).any(|x| x == user.id) {
      return Err(APIError::err("not_an_admin").into());
    }

    let community_id = data.community_id;
    let new_creator = data.user_id;
    let update = move |conn: &'_ _| Community::update_creator(conn, community_id, new_creator);
    if blocking(context.pool(), update).await?.is_err() {
      return Err(APIError::err("couldnt_update_community").into());
    };

    // You also have to re-do the community_moderator table, reordering it.
    let community_id = data.community_id;
    let mut community_mods = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;
    let creator_index = community_mods
      .iter()
      .position(|r| r.user_id == data.user_id)
      .context(location_info!())?;
    let creator_user = community_mods.remove(creator_index);
    community_mods.insert(0, creator_user);

    let community_id = data.community_id;
    blocking(context.pool(), move |conn| {
      CommunityModerator::delete_for_community(conn, community_id)
    })
    .await??;

    // TODO: this should probably be a bulk operation
    for cmod in &community_mods {
      let community_moderator_form = CommunityModeratorForm {
        community_id: cmod.community_id,
        user_id: cmod.user_id,
      };

      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      if blocking(context.pool(), join).await?.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user.id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    blocking(context.pool(), move |conn| {
      ModAddCommunity::create(conn, &form)
    })
    .await??;

    let community_id = data.community_id;
    let user_id = user.id;
    let community_view = match blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(user_id))
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let community_id = data.community_id;
    let moderators = match blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await?
    {
      Ok(moderators) => moderators,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    // Return the jwt
    Ok(GetCommunityResponse {
      community: community_view,
      moderators,
      online: 0,
    })
  }
}

pub fn send_community_websocket(
  res: &CommunityResponse,
  context: &Data<LemmyContext>,
  websocket_id: Option<ConnectionId>,
  op: UserOperation,
) {
  // Strip out the user id and subscribed when sending to others
  let mut res_sent = res.clone();
  res_sent.community.user_id = None;
  res_sent.community.subscribed = None;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op,
    response: res_sent,
    community_id: res.community.id,
    websocket_id,
  });
}

#[async_trait::async_trait(?Send)]
impl Perform for CommunityJoin {
  type Response = CommunityJoinResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityJoinResponse, LemmyError> {
    let data: &CommunityJoin = &self;

    if let Some(ws_id) = websocket_id {
      context.chat_server().do_send(JoinCommunityRoom {
        community_id: data.community_id,
        id: ws_id,
      });
    }

    Ok(CommunityJoinResponse { joined: true })
  }
}
