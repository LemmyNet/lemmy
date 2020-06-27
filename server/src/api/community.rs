use super::*;
use crate::{
  api::{APIError, Oper, Perform},
  apub::{
    extensions::signatures::generate_actor_keypair, make_apub_endpoint, ActorType, EndpointType,
  },
  db::{Bannable, Crud, Followable, Joinable, SortType},
  is_valid_community_name, naive_from_unix, naive_now, slur_check, slurs_vec_to_str,
  websocket::{
    server::{JoinCommunityRoom, SendCommunityRoomMessage},
    UserOperation, WebsocketInfo,
  },
  DbPool, LemmyError,
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
  pub admins: Vec<UserView>,
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
  name: String,
  title: String,
  description: Option<String>,
  category_id: i32,
  removed: Option<bool>,
  deleted: Option<bool>,
  nsfw: bool,
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
    pool: DbPool,
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
    let community: Community = match data.id {
      Some(id) => unblock!(pool, conn, Community::read(&conn, id)?),
      None => match unblock!(pool, conn, Community::read_from_name(&conn, &name)) {
        Ok(community) => community,
        Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
      },
    };

    let community_id = community.id;
    let community_view: CommunityView = match unblock!(
      pool,
      conn,
      CommunityView::read(&conn, community_id, user_id)
    ) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let community_id = community.id;
    let moderators: Vec<CommunityModeratorView> = match unblock!(
      pool,
      conn,
      CommunityModeratorView::for_community(&conn, community_id)
    ) {
      Ok(moderators) => moderators,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let site: Site = unblock!(pool, conn, Site::read(&conn, 1)?);
    let site_creator_id = site.creator_id;
    let mut admins: Vec<UserView> = unblock!(pool, conn, UserView::admins(&conn)?);
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

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
      admins,
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
    pool: DbPool,
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
    let user_view: UserView = unblock!(pool, conn, UserView::read(&conn, user_id)?);
    if user_view.banned {
      return Err(APIError::err("site_ban").into());
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
      actor_id: make_apub_endpoint(EndpointType::Community, &data.name).to_string(),
      local: true,
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      last_refreshed_at: None,
      published: None,
    };

    let inserted_community: Community =
      match unblock!(pool, conn, Community::create(&conn, &community_form)) {
        Ok(community) => community,
        Err(_e) => return Err(APIError::err("community_already_exists").into()),
      };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id,
    };

    let _inserted_community_moderator: CommunityModerator = match unblock!(
      pool,
      conn,
      CommunityModerator::join(&conn, &community_moderator_form)
    ) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("community_moderator_already_exists").into()),
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id,
    };

    let _inserted_community_follower: CommunityFollower = match unblock!(
      pool,
      conn,
      CommunityFollower::follow(&conn, &community_follower_form)
    ) {
      Ok(user) => user,
      Err(_e) => return Err(APIError::err("community_follower_already_exists").into()),
    };

    let community_view: CommunityView = unblock!(
      pool,
      conn,
      CommunityView::read(&conn, inserted_community.id, Some(user_id))?
    );

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
    pool: DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &EditCommunity = &self.data;

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

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    if !is_valid_community_name(&data.name) {
      return Err(APIError::err("invalid_community_name").into());
    }

    let user_id = claims.id;

    // Check for a site ban
    let user: User_ = unblock!(pool, conn, User_::read(&conn, user_id)?);
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Verify its a mod
    let edit_id = data.edit_id;
    let mut editors: Vec<i32> = Vec::new();
    editors.append(&mut unblock!(
      pool,
      conn,
      CommunityModeratorView::for_community(&conn, edit_id)?
        .into_iter()
        .map(|m| m.user_id)
        .collect()
    ));
    editors.append(&mut unblock!(
      pool,
      conn,
      UserView::admins(&conn)?.into_iter().map(|a| a.id).collect()
    ));
    if !editors.contains(&user_id) {
      return Err(APIError::err("no_community_edit_allowed").into());
    }

    let edit_id = data.edit_id;
    let read_community: Community = unblock!(pool, conn, Community::read(&conn, edit_id)?);

    let community_form = CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      category_id: data.category_id.to_owned(),
      creator_id: user_id,
      removed: data.removed.to_owned(),
      deleted: data.deleted.to_owned(),
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
    let updated_community: Community = match unblock!(
      pool,
      conn,
      Community::update(&conn, edit_id, &community_form)
    ) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_update_community").into()),
    };

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
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
      let _: ModRemoveCommunity = unblock!(pool, conn, ModRemoveCommunity::create(&conn, &form)?);
    }

    if let Some(deleted) = data.deleted.to_owned() {
      if deleted {
        updated_community
          .send_delete(&user, &self.client, pool.clone())
          .await?;
      } else {
        updated_community
          .send_undo_delete(&user, &self.client, pool.clone())
          .await?;
      }
    } else if let Some(removed) = data.removed.to_owned() {
      if removed {
        updated_community
          .send_remove(&user, &self.client, pool.clone())
          .await?;
      } else {
        updated_community
          .send_undo_remove(&user, &self.client, pool.clone())
          .await?;
      }
    }

    let edit_id = data.edit_id;
    let community_view: CommunityView = unblock!(
      pool,
      conn,
      CommunityView::read(&conn, edit_id, Some(user_id))?
    );

    let res = CommunityResponse {
      community: community_view,
    };

    if let Some(ws) = websocket_info {
      // Strip out the user id and subscribed when sending to others
      let mut res_sent = res.clone();
      res_sent.community.user_id = None;
      res_sent.community.subscribed = None;

      ws.chatserver.do_send(SendCommunityRoomMessage {
        op: UserOperation::EditCommunity,
        response: res_sent,
        community_id: data.edit_id,
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<ListCommunities> {
  type Response = ListCommunitiesResponse;

  async fn perform(
    &self,
    pool: DbPool,
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
    let communities: Vec<CommunityView> = unblock!(
      pool,
      conn,
      CommunityQueryBuilder::create(&conn)
        .sort(&sort)
        .for_user(user_id)
        .show_nsfw(show_nsfw)
        .page(page)
        .limit(limit)
        .list()?
    );

    // Return the jwt
    Ok(ListCommunitiesResponse { communities })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<FollowCommunity> {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &FollowCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_id = data.community_id;
    let community: Community = unblock!(pool, conn, Community::read(&conn, community_id)?);
    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      user_id,
    };

    if community.local {
      if data.follow {
        let res: Result<_, _> = unblock!(
          pool,
          conn,
          CommunityFollower::follow(&conn, &community_follower_form)
        );
        if res.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        }
      } else {
        let res: Result<_, _> = unblock!(
          pool,
          conn,
          CommunityFollower::unfollow(&conn, &community_follower_form)
        );
        if res.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        }
      }
    } else {
      let user: User_ = unblock!(pool, conn, User_::read(&conn, user_id)?);

      if data.follow {
        // Dont actually add to the community followers here, because you need
        // to wait for the accept
        user
          .send_follow(&community.actor_id, &self.client, pool.clone())
          .await?;
      } else {
        user
          .send_unfollow(&community.actor_id, &self.client, pool.clone())
          .await?;
        let res: Result<_, _> = unblock!(
          pool,
          conn,
          CommunityFollower::unfollow(&conn, &community_follower_form)
        );
        if res.is_err() {
          return Err(APIError::err("community_follower_already_exists").into());
        };
      }
      // TODO: this needs to return a "pending" state, until Accept is received from the remote server
    }

    let community_id = data.community_id;
    let community_view: CommunityView = unblock!(
      pool,
      conn,
      CommunityView::read(&conn, community_id, Some(user_id))?
    );

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
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetFollowedCommunitiesResponse, LemmyError> {
    let data: &GetFollowedCommunities = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let res: Result<_, _> = unblock!(pool, conn, CommunityFollowerView::for_user(&conn, user_id));

    let communities: Vec<CommunityFollowerView> = match res {
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
    pool: DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<BanFromCommunityResponse, LemmyError> {
    let data: &BanFromCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_user_ban_form = CommunityUserBanForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    if data.ban {
      let res: Result<_, _> = unblock!(
        pool,
        conn,
        CommunityUserBan::ban(&conn, &community_user_ban_form)
      );

      if res.is_err() {
        return Err(APIError::err("community_user_already_banned").into());
      }
    } else {
      let res: Result<_, _> = unblock!(
        pool,
        conn,
        CommunityUserBan::unban(&conn, &community_user_ban_form)
      );

      if res.is_err() {
        return Err(APIError::err("community_user_already_banned").into());
      }
    }

    // Mod tables
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
    let _: ModBanFromCommunity = unblock!(pool, conn, ModBanFromCommunity::create(&conn, &form)?);

    let user_id = data.user_id;
    let user_view: UserView = unblock!(pool, conn, UserView::read(&conn, user_id)?);

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
    pool: DbPool,
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

    if data.added {
      let res: Result<_, _> = unblock!(
        pool,
        conn,
        CommunityModerator::join(&conn, &community_moderator_form)
      );
      if res.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      }
    } else {
      let res: Result<_, _> = unblock!(
        pool,
        conn,
        CommunityModerator::leave(&conn, &community_moderator_form)
      );
      if res.is_err() {
        return Err(APIError::err("community_moderator_already_exists").into());
      };
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    let _: ModAddCommunity = unblock!(pool, conn, ModAddCommunity::create(&conn, &form)?);

    let community_id = data.community_id;
    let moderators: Vec<CommunityModeratorView> = unblock!(
      pool,
      conn,
      CommunityModeratorView::for_community(&conn, community_id)?
    );

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
    pool: DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &TransferCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_id = data.community_id;
    let read_community: Community = unblock!(pool, conn, Community::read(&conn, community_id)?);

    let site: Site = unblock!(pool, conn, Site::read(&conn, 1)?);
    let site_creator_id = site.creator_id;

    let mut admins: Vec<UserView> = unblock!(pool, conn, UserView::admins(&conn)?);

    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    // Make sure user is the creator, or an admin
    if user_id != read_community.creator_id && !admins.iter().map(|a| a.id).any(|x| x == user_id) {
      return Err(APIError::err("not_an_admin").into());
    }

    let community_form = CommunityForm {
      name: read_community.name,
      title: read_community.title,
      description: read_community.description,
      category_id: read_community.category_id,
      creator_id: data.user_id, // This makes the new user the community creator
      removed: None,
      deleted: None,
      nsfw: read_community.nsfw,
      updated: Some(naive_now()),
      actor_id: read_community.actor_id,
      local: read_community.local,
      private_key: read_community.private_key,
      public_key: read_community.public_key,
      last_refreshed_at: None,
      published: None,
    };

    let community_id = data.community_id;
    let _: Community = match unblock!(
      pool,
      conn,
      Community::update(&conn, community_id, &community_form)
    ) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_update_community").into()),
    };

    // You also have to re-do the community_moderator table, reordering it.
    let community_id = data.community_id;
    let mut community_mods: Vec<CommunityModeratorView> = unblock!(
      pool,
      conn,
      CommunityModeratorView::for_community(&conn, community_id)?
    );
    let creator_index = community_mods
      .iter()
      .position(|r| r.user_id == data.user_id)
      .unwrap();
    let creator_user = community_mods.remove(creator_index);
    community_mods.insert(0, creator_user);

    let community_id = data.community_id;
    let _: usize = unblock!(
      pool,
      conn,
      CommunityModerator::delete_for_community(&conn, community_id)?
    );

    for cmod in &community_mods {
      let community_moderator_form = CommunityModeratorForm {
        community_id: cmod.community_id,
        user_id: cmod.user_id,
      };

      let _: CommunityModerator = match unblock!(
        pool,
        conn,
        CommunityModerator::join(&conn, &community_moderator_form)
      ) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_moderator_already_exists").into()),
      };
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    let _: ModAddCommunity = unblock!(pool, conn, ModAddCommunity::create(&conn, &form)?);

    let community_id = data.community_id;
    let community_view: CommunityView = match unblock!(
      pool,
      conn,
      CommunityView::read(&conn, community_id, Some(user_id))
    ) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let community_id = data.community_id;
    let moderators: Vec<CommunityModeratorView> = match unblock!(
      pool,
      conn,
      CommunityModeratorView::for_community(&conn, community_id)
    ) {
      Ok(moderators) => moderators,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    // Return the jwt
    Ok(GetCommunityResponse {
      community: community_view,
      moderators,
      admins,
      online: 0,
    })
  }
}
