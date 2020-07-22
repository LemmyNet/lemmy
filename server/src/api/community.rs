use super::*;
use crate::{
  api::{claims::Claims, is_admin, is_mod_or_admin, APIError, Oper, Perform},
  apub::ActorType,
  blocking,
  websocket::{
    server::{JoinCommunityRoom, SendCommunityRoomMessage},
    UserOperation,
    WebsocketInfo,
  },
  DbPool,
};
use lemmy_db::{naive_now, Bannable, Crud, Followable, Joinable, SortType};
use lemmy_utils::{
  generate_actor_keypair,
  is_valid_community_name,
  make_apub_endpoint,
  naive_from_unix,
  slur_check,
  slurs_vec_to_str,
  EndpointType,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct GetCommunity {
  id: Option<i32>,
  pub name: Option<String>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunityResponse {
  pub community: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommunity {
  name: String,
  title: String,
  description: Option<String>,
  category_id: i32,
  nsfw: bool,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommunityResponse {
  pub community: CommunityView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommunities {
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BanFromCommunity {
  pub community_id: i32,
  user_id: i32,
  ban: bool,
  reason: Option<String>,
  expires: Option<i64>,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BanFromCommunityResponse {
  user: UserView,
  banned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AddModToCommunity {
  pub community_id: i32,
  user_id: i32,
  added: bool,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddModToCommunityResponse {
  moderators: Vec<CommunityModeratorView>,
}

#[derive(Serialize, Deserialize)]
pub struct EditCommunity {
  pub edit_id: i32,
  title: String,
  description: Option<String>,
  category_id: i32,
  nsfw: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteCommunity {
  pub edit_id: i32,
  deleted: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveCommunity {
  pub edit_id: i32,
  removed: bool,
  reason: Option<String>,
  expires: Option<i64>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct FollowCommunity {
  community_id: i32,
  follow: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunities {
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunitiesResponse {
  communities: Vec<CommunityFollowerView>,
}

#[derive(Serialize, Deserialize)]
pub struct TransferCommunity {
  community_id: i32,
  user_id: i32,
  auth: String,
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetCommunity> {
  type Response = GetCommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &GetCommunity = &self.data;

    let user_id: Option<i32> = match &data.auth {
      Some(auth) => match Claims::decode(&auth) {
        Ok(claims) => {
          let user_id = claims.claims.id;
          Some(user_id)
        }
        Err(_e) => None,
      },
      None => None,
    };

    let name = data.name.to_owned().unwrap_or_else(|| "main".to_string());
    let community = match data.id {
      Some(id) => blocking(pool, move |conn| Community::read(conn, id)).await??,
      None => match blocking(pool, move |conn| Community::read_from_name(conn, &name)).await? {
        Ok(community) => community,
        Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
      },
    };

    let community_id = community.id;
    let community_view = match blocking(pool, move |conn| {
      CommunityView::read(conn, community_id, user_id)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let community_id = community.id;
    let moderators: Vec<CommunityModeratorView> = match blocking(pool, move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await?
    {
      Ok(moderators) => moderators,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let online = if let Some(ws) = websocket_info {
      if let Some(id) = ws.id {
        ws.chatserver.do_send(JoinCommunityRoom {
          community_id: community.id,
          id,
        });
      }

      // TODO
      1
    // let fut = async {
    //   ws.chatserver.send(GetCommunityUsersOnline {community_id}).await.unwrap()
    // };
    // Runtime::new().unwrap().block_on(fut)
    } else {
      0
    };

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
impl Perform for Oper<CreateCommunity> {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &CreateCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    if let Err(slurs) = slur_check(&data.name) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    if let Err(slurs) = slur_check(&data.title) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    if let Some(description) = &data.description {
      if let Err(slurs) = slur_check(description) {
        return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
      }
    }

    if !is_valid_community_name(&data.name) {
      return Err(APIError::err("invalid_community_name").into());
    }

    let user_id = claims.id;

    // Check for a site ban
    let user_view = blocking(pool, move |conn| UserView::read(conn, user_id)).await??;
    if user_view.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Double check for duplicate community actor_ids
    let actor_id = make_apub_endpoint(EndpointType::Community, &data.name).to_string();
    let actor_id_cloned = actor_id.to_owned();
    let community_dupe = blocking(pool, move |conn| {
      Community::read_from_actor_id(conn, &actor_id_cloned)
    })
    .await?;
    if community_dupe.is_ok() {
      return Err(APIError::err("community_already_exists").into());
    }

    // When you create a community, make sure the user becomes a moderator and a follower
    let keypair = generate_actor_keypair()?;

    let community_form = CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      category_id: data.category_id,
      creator_id: user_id,
      removed: None,
      deleted: None,
      nsfw: data.nsfw,
      updated: None,
      actor_id,
      local: true,
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      last_refreshed_at: None,
      published: None,
    };

    let inserted_community =
      match blocking(pool, move |conn| Community::create(conn, &community_form)).await? {
        Ok(community) => community,
        Err(_e) => return Err(APIError::err("community_already_exists").into()),
      };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id,
    };

    let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
    if blocking(pool, join).await?.is_err() {
      return Err(APIError::err("community_moderator_already_exists").into());
    }

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id,
    };

    let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
    if blocking(pool, follow).await?.is_err() {
      return Err(APIError::err("community_follower_already_exists").into());
    }

    let community_view = blocking(pool, move |conn| {
      CommunityView::read(conn, inserted_community.id, Some(user_id))
    })
    .await??;

    Ok(CommunityResponse {
      community: community_view,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<EditCommunity> {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &EditCommunity = &self.data;

    if let Err(slurs) = slur_check(&data.title) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    if let Some(description) = &data.description {
      if let Err(slurs) = slur_check(description) {
        return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
      }
    }

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Verify its a mod (only mods can edit it)
    let edit_id = data.edit_id;
    let mods: Vec<i32> = blocking(pool, move |conn| {
      CommunityModeratorView::for_community(conn, edit_id)
        .map(|v| v.into_iter().map(|m| m.user_id).collect())
    })
    .await??;
    if !mods.contains(&user_id) {
      return Err(APIError::err("not_a_moderator").into());
    }

    let edit_id = data.edit_id;
    let read_community = blocking(pool, move |conn| Community::read(conn, edit_id)).await??;

    let community_form = CommunityForm {
      name: read_community.name,
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      category_id: data.category_id.to_owned(),
      creator_id: read_community.creator_id,
      removed: Some(read_community.removed),
      deleted: Some(read_community.deleted),
      nsfw: data.nsfw,
      updated: Some(naive_now()),
      actor_id: read_community.actor_id,
      local: read_community.local,
      private_key: read_community.private_key,
      public_key: read_community.public_key,
      last_refreshed_at: None,
      published: None,
    };

    let edit_id = data.edit_id;
    match blocking(pool, move |conn| {
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
    let community_view = blocking(pool, move |conn| {
      CommunityView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = CommunityResponse {
      community: community_view,
    };

    send_community_websocket(&res, websocket_info, UserOperation::EditCommunity);

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<DeleteCommunity> {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &DeleteCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Verify its the creator (only a creator can delete the community)
    let edit_id = data.edit_id;
    let read_community = blocking(pool, move |conn| Community::read(conn, edit_id)).await??;
    if read_community.creator_id != user_id {
      return Err(APIError::err("no_community_edit_allowed").into());
    }

    // Do the delete
    let edit_id = data.edit_id;
    let deleted = data.deleted;
    let updated_community = match blocking(pool, move |conn| {
      Community::update_deleted(conn, edit_id, deleted)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_update_community").into()),
    };

    // Send apub messages
    if deleted {
      updated_community
        .send_delete(&user, &self.client, pool)
        .await?;
    } else {
      updated_community
        .send_undo_delete(&user, &self.client, pool)
        .await?;
    }

    let edit_id = data.edit_id;
    let community_view = blocking(pool, move |conn| {
      CommunityView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = CommunityResponse {
      community: community_view,
    };

    send_community_websocket(&res, websocket_info, UserOperation::DeleteCommunity);

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<RemoveCommunity> {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &RemoveCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Check for a site ban
    let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Verify its an admin (only an admin can remove a community)
    is_admin(pool, user_id).await?;

    // Do the remove
    let edit_id = data.edit_id;
    let removed = data.removed;
    let updated_community = match blocking(pool, move |conn| {
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
      mod_user_id: user_id,
      community_id: data.edit_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
      expires,
    };
    blocking(pool, move |conn| ModRemoveCommunity::create(conn, &form)).await??;

    // Apub messages
    if removed {
      updated_community
        .send_remove(&user, &self.client, pool)
        .await?;
    } else {
      updated_community
        .send_undo_remove(&user, &self.client, pool)
        .await?;
    }

    let edit_id = data.edit_id;
    let community_view = blocking(pool, move |conn| {
      CommunityView::read(conn, edit_id, Some(user_id))
    })
    .await??;

    let res = CommunityResponse {
      community: community_view,
    };

    send_community_websocket(&res, websocket_info, UserOperation::RemoveCommunity);

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<ListCommunities> {
  type Response = ListCommunitiesResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<ListCommunitiesResponse, LemmyError> {
    let data: &ListCommunities = &self.data;

    let user_claims: Option<Claims> = match &data.auth {
      Some(auth) => match Claims::decode(&auth) {
        Ok(claims) => Some(claims.claims),
        Err(_e) => None,
      },
      None => None,
    };

    let user_id = match &user_claims {
      Some(claims) => Some(claims.id),
      None => None,
    };

    let show_nsfw = match &user_claims {
      Some(claims) => claims.show_nsfw,
      None => false,
    };

    let sort = SortType::from_str(&data.sort)?;

    let page = data.page;
    let limit = data.limit;
    let communities = blocking(pool, move |conn| {
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
impl Perform for Oper<FollowCommunity> {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &FollowCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_id = data.community_id;
    let community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      user_id,
    };

    if community.local {
      if data.follow {
        let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
        if blocking(pool, follow).await?.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        }
      } else {
        let unfollow =
          move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
        if blocking(pool, unfollow).await?.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        }
      }
    } else {
      let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;

      if data.follow {
        // Dont actually add to the community followers here, because you need
        // to wait for the accept
        user
          .send_follow(&community.actor_id, &self.client, pool)
          .await?;
      } else {
        user
          .send_unfollow(&community.actor_id, &self.client, pool)
          .await?;
        let unfollow =
          move |conn: &'_ _| CommunityFollower::unfollow(conn, &community_follower_form);
        if blocking(pool, unfollow).await?.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        }
      }
      // TODO: this needs to return a "pending" state, until Accept is received from the remote server
    }

    let community_id = data.community_id;
    let community_view = blocking(pool, move |conn| {
      CommunityView::read(conn, community_id, Some(user_id))
    })
    .await??;

    Ok(CommunityResponse {
      community: community_view,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetFollowedCommunities> {
  type Response = GetFollowedCommunitiesResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetFollowedCommunitiesResponse, LemmyError> {
    let data: &GetFollowedCommunities = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let communities = match blocking(pool, move |conn| {
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
impl Perform for Oper<BanFromCommunity> {
  type Response = BanFromCommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<BanFromCommunityResponse, LemmyError> {
    let data: &BanFromCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_id = data.community_id;

    // Verify that only mods or admins can ban
    is_mod_or_admin(pool, user_id, community_id).await?;

    let community_user_ban_form = CommunityUserBanForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    if data.ban {
      let ban = move |conn: &'_ _| CommunityUserBan::ban(conn, &community_user_ban_form);
      if blocking(pool, ban).await?.is_err() {
        return Err(APIError::err("community_user_already_banned").into());
      }
    } else {
      let unban = move |conn: &'_ _| CommunityUserBan::unban(conn, &community_user_ban_form);
      if blocking(pool, unban).await?.is_err() {
        return Err(APIError::err("community_user_already_banned").into());
      }
    }

    // Mod tables
    // TODO eventually do correct expires
    let expires = match data.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None,
    };

    let form = ModBanFromCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires,
    };
    blocking(pool, move |conn| ModBanFromCommunity::create(conn, &form)).await??;

    let user_id = data.user_id;
    let user_view = blocking(pool, move |conn| UserView::read(conn, user_id)).await??;

    let res = BanFromCommunityResponse {
      user: user_view,
      banned: data.ban,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendCommunityRoomMessage {
        op: UserOperation::BanFromCommunity,
        response: res.clone(),
        community_id: data.community_id,
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<AddModToCommunity> {
  type Response = AddModToCommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<AddModToCommunityResponse, LemmyError> {
    let data: &AddModToCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    let community_id = data.community_id;

    // Verify that only mods or admins can add mod
    is_mod_or_admin(pool, user_id, community_id).await?;

    if data.added {
      let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
      if blocking(pool, join).await?.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    } else {
      let leave = move |conn: &'_ _| CommunityModerator::leave(conn, &community_moderator_form);
      if blocking(pool, leave).await?.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    blocking(pool, move |conn| ModAddCommunity::create(conn, &form)).await??;

    let community_id = data.community_id;
    let moderators = blocking(pool, move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    let res = AddModToCommunityResponse { moderators };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendCommunityRoomMessage {
        op: UserOperation::AddModToCommunity,
        response: res.clone(),
        community_id: data.community_id,
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<TransferCommunity> {
  type Response = GetCommunityResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &TransferCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_id = data.community_id;
    let read_community = blocking(pool, move |conn| Community::read(conn, community_id)).await??;

    let site_creator_id =
      blocking(pool, move |conn| Site::read(conn, 1).map(|s| s.creator_id)).await??;

    let mut admins = blocking(pool, move |conn| UserView::admins(conn)).await??;

    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    // Make sure user is the creator, or an admin
    if user_id != read_community.creator_id && !admins.iter().map(|a| a.id).any(|x| x == user_id) {
      return Err(APIError::err("not_an_admin").into());
    }

    let community_id = data.community_id;
    let new_creator = data.user_id;
    let update = move |conn: &'_ _| Community::update_creator(conn, community_id, new_creator);
    if blocking(pool, update).await?.is_err() {
      return Err(APIError::err("couldnt_update_community").into());
    };

    // You also have to re-do the community_moderator table, reordering it.
    let community_id = data.community_id;
    let mut community_mods = blocking(pool, move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;
    let creator_index = community_mods
      .iter()
      .position(|r| r.user_id == data.user_id)
      .unwrap();
    let creator_user = community_mods.remove(creator_index);
    community_mods.insert(0, creator_user);

    let community_id = data.community_id;
    blocking(pool, move |conn| {
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
      if blocking(pool, join).await?.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    blocking(pool, move |conn| ModAddCommunity::create(conn, &form)).await??;

    let community_id = data.community_id;
    let community_view = match blocking(pool, move |conn| {
      CommunityView::read(conn, community_id, Some(user_id))
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let community_id = data.community_id;
    let moderators = match blocking(pool, move |conn| {
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
  websocket_info: Option<WebsocketInfo>,
  op: UserOperation,
) {
  if let Some(ws) = websocket_info {
    // Strip out the user id and subscribed when sending to others
    let mut res_sent = res.clone();
    res_sent.community.user_id = None;
    res_sent.community.subscribed = None;

    ws.chatserver.do_send(SendCommunityRoomMessage {
      op,
      response: res_sent,
      community_id: res.community.id,
      my_id: ws.id,
    });
  }
}
