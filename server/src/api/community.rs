use super::*;

#[derive(Serialize, Deserialize)]
pub struct GetCommunity {
  id: Option<i32>,
  name: Option<String>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunityResponse {
  pub community: CommunityView,
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
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

#[derive(Serialize, Deserialize)]
pub struct ListCommunities {
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ListCommunitiesResponse {
  communities: Vec<CommunityView>,
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

impl Perform for Oper<GetCommunity> {
  type Response = GetCommunityResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommunityResponse, Error> {
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

    let conn = pool.get()?;

    let community_id = match data.id {
      Some(id) => id,
      None => {
        match Community::read_from_name(
          &conn,
          data.name.to_owned().unwrap_or_else(|| "main".to_string()),
        ) {
          Ok(community) => community.id,
          Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
        }
      }
    };

    let community_view = match CommunityView::read(&conn, community_id, user_id) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let moderators = match CommunityModeratorView::for_community(&conn, community_id) {
      Ok(moderators) => moderators,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let site_creator_id = Site::read(&conn, 1)?.creator_id;
    let mut admins = UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    let online = if let Some(ws) = websocket_info {
      if let Some(id) = ws.id {
        ws.chatserver
          .do_send(JoinCommunityRoom { community_id, id });
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

impl Perform for Oper<CreateCommunity> {
  type Response = CommunityResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, Error> {
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

    let user_id = claims.id;

    let conn = pool.get()?;

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err("site_ban").into());
    }

    // When you create a community, make sure the user becomes a moderator and a follower
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
    };

    let inserted_community = match Community::create(&conn, &community_form) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("community_already_exists").into()),
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id,
    };

    let _inserted_community_moderator =
      match CommunityModerator::join(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_moderator_already_exists").into()),
      };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id,
    };

    let _inserted_community_follower =
      match CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_follower_already_exists").into()),
      };

    let community_view = CommunityView::read(&conn, inserted_community.id, Some(user_id))?;

    Ok(CommunityResponse {
      community: community_view,
    })
  }
}

impl Perform for Oper<EditCommunity> {
  type Response = CommunityResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, Error> {
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

    let user_id = claims.id;

    let conn = pool.get()?;

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Verify its a mod
    let mut editors: Vec<i32> = Vec::new();
    editors.append(
      &mut CommunityModeratorView::for_community(&conn, data.edit_id)?
        .into_iter()
        .map(|m| m.user_id)
        .collect(),
    );
    editors.append(&mut UserView::admins(&conn)?.into_iter().map(|a| a.id).collect());
    if !editors.contains(&user_id) {
      return Err(APIError::err("no_community_edit_allowed").into());
    }

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
    };

    let _updated_community = match Community::update(&conn, data.edit_id, &community_form) {
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
      ModRemoveCommunity::create(&conn, &form)?;
    }

    let community_view = CommunityView::read(&conn, data.edit_id, Some(user_id))?;

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

impl Perform for Oper<ListCommunities> {
  type Response = ListCommunitiesResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<ListCommunitiesResponse, Error> {
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

    let conn = pool.get()?;

    let communities = CommunityQueryBuilder::create(&conn)
      .sort(&sort)
      .for_user(user_id)
      .show_nsfw(show_nsfw)
      .page(data.page)
      .limit(data.limit)
      .list()?;

    // Return the jwt
    Ok(ListCommunitiesResponse { communities })
  }
}

impl Perform for Oper<FollowCommunity> {
  type Response = CommunityResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommunityResponse, Error> {
    let data: &FollowCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      user_id,
    };

    let conn = pool.get()?;

    if data.follow {
      match CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_follower_already_exists").into()),
      };
    } else {
      match CommunityFollower::ignore(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_follower_already_exists").into()),
      };
    }

    let community_view = CommunityView::read(&conn, data.community_id, Some(user_id))?;

    Ok(CommunityResponse {
      community: community_view,
    })
  }
}

impl Perform for Oper<GetFollowedCommunities> {
  type Response = GetFollowedCommunitiesResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetFollowedCommunitiesResponse, Error> {
    let data: &GetFollowedCommunities = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let conn = pool.get()?;

    let communities: Vec<CommunityFollowerView> =
      match CommunityFollowerView::for_user(&conn, user_id) {
        Ok(communities) => communities,
        Err(_e) => return Err(APIError::err("system_err_login").into()),
      };

    // Return the jwt
    Ok(GetFollowedCommunitiesResponse { communities })
  }
}

impl Perform for Oper<BanFromCommunity> {
  type Response = BanFromCommunityResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<BanFromCommunityResponse, Error> {
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

    let conn = pool.get()?;

    if data.ban {
      match CommunityUserBan::ban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_user_already_banned").into()),
      };
    } else {
      match CommunityUserBan::unban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_user_already_banned").into()),
      };
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
    ModBanFromCommunity::create(&conn, &form)?;

    let user_view = UserView::read(&conn, data.user_id)?;

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

impl Perform for Oper<AddModToCommunity> {
  type Response = AddModToCommunityResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<AddModToCommunityResponse, Error> {
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

    let conn = pool.get()?;

    if data.added {
      match CommunityModerator::join(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_moderator_already_exists").into()),
      };
    } else {
      match CommunityModerator::leave(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => return Err(APIError::err("community_moderator_already_exists").into()),
      };
    }

    // Mod tables
    let form = ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    ModAddCommunity::create(&conn, &form)?;

    let moderators = CommunityModeratorView::for_community(&conn, data.community_id)?;

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

impl Perform for Oper<TransferCommunity> {
  type Response = GetCommunityResponse;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommunityResponse, Error> {
    let data: &TransferCommunity = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let conn = pool.get()?;

    let read_community = Community::read(&conn, data.community_id)?;

    let site_creator_id = Site::read(&conn, 1)?.creator_id;
    let mut admins = UserView::admins(&conn)?;
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
      creator_id: data.user_id,
      removed: None,
      deleted: None,
      nsfw: read_community.nsfw,
      updated: Some(naive_now()),
    };

    let _updated_community = match Community::update(&conn, data.community_id, &community_form) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_update_community").into()),
    };

    // You also have to re-do the community_moderator table, reordering it.
    let mut community_mods = CommunityModeratorView::for_community(&conn, data.community_id)?;
    let creator_index = community_mods
      .iter()
      .position(|r| r.user_id == data.user_id)
      .unwrap();
    let creator_user = community_mods.remove(creator_index);
    community_mods.insert(0, creator_user);

    CommunityModerator::delete_for_community(&conn, data.community_id)?;

    for cmod in &community_mods {
      let community_moderator_form = CommunityModeratorForm {
        community_id: cmod.community_id,
        user_id: cmod.user_id,
      };

      let _inserted_community_moderator =
        match CommunityModerator::join(&conn, &community_moderator_form) {
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
    ModAddCommunity::create(&conn, &form)?;

    let community_view = match CommunityView::read(&conn, data.community_id, Some(user_id)) {
      Ok(community) => community,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let moderators = match CommunityModeratorView::for_community(&conn, data.community_id) {
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
