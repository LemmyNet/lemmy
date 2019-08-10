use super::*;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct GetCommunity {
  id: Option<i32>,
  name: Option<String>,
  auth: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunityResponse {
  op: String,
  community: CommunityView,
  moderators: Vec<CommunityModeratorView>,
  admins: Vec<UserView>,
}


#[derive(Serialize, Deserialize)]
pub struct CreateCommunity {
  name: String,
  title: String,
  description: Option<String>,
  category_id: i32 ,
  auth: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommunityResponse {
  op: String,
  pub community: CommunityView
}

#[derive(Serialize, Deserialize)]
pub struct ListCommunities {
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  auth: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct ListCommunitiesResponse {
  op: String,
  communities: Vec<CommunityView>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BanFromCommunity {
  pub community_id: i32,
  user_id: i32,
  ban: bool,
  reason: Option<String>,
  expires: Option<i64>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct BanFromCommunityResponse {
  op: String,
  user: UserView,
  banned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AddModToCommunity {
  pub community_id: i32,
  user_id: i32,
  added: bool,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct AddModToCommunityResponse {
  op: String,
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
  reason: Option<String>,
  expires: Option<i64>,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct FollowCommunity {
  community_id: i32,
  follow: bool,
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunities {
  auth: String
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunitiesResponse {
  op: String,
  communities: Vec<CommunityFollowerView>
}

impl Perform<GetCommunityResponse> for Oper<GetCommunity> {
  fn perform(&self) -> Result<GetCommunityResponse, Error> {
    let data: &GetCommunity = &self.data;
    let conn = establish_connection();

    let user_id: Option<i32> = match &data.auth {
      Some(auth) => {
        match Claims::decode(&auth) {
          Ok(claims) => {
            let user_id = claims.claims.id;
            Some(user_id)
          }
          Err(_e) => None
        }
      }
      None => None
    };

    let community_id = match data.id {
      Some(id) => id,
      None => Community::read_from_name(&conn, data.name.to_owned().unwrap_or("main".to_string()))?.id
    };

    let community_view = match CommunityView::read(&conn, community_id, user_id) {
      Ok(community) => community,
      Err(_e) => {
        return Err(APIError::err(&self.op, "couldnt_find_community"))?
      }
    };

    let moderators = match CommunityModeratorView::for_community(&conn, community_id) {
      Ok(moderators) => moderators,
      Err(_e) => {
        return Err(APIError::err(&self.op, "couldnt_find_community"))?
      }
    };

    let admins = UserView::admins(&conn)?;

    // Return the jwt
    Ok(
      GetCommunityResponse {
        op: self.op.to_string(),
        community: community_view,
        moderators: moderators,
        admins: admins,
      }
      )
  }
}

impl Perform<CommunityResponse> for Oper<CreateCommunity> {
  fn perform(&self) -> Result<CommunityResponse, Error> {
    let data: &CreateCommunity = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(APIError::err(&self.op, "not_logged_in"))?
      }
    };

    if has_slurs(&data.name) || 
      has_slurs(&data.title) || 
        (data.description.is_some() && has_slurs(&data.description.to_owned().unwrap())) {
          return Err(APIError::err(&self.op, "no_slurs"))?
        }

    let user_id = claims.id;

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err(&self.op, "site_ban"))?
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
      updated: None,
    };

    let inserted_community = match Community::create(&conn, &community_form) {
      Ok(community) => community,
      Err(_e) => {
        return Err(APIError::err(&self.op, "community_already_exists"))?
      }
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let _inserted_community_moderator = match CommunityModerator::join(&conn, &community_moderator_form) {
      Ok(user) => user,
      Err(_e) => {
        return Err(APIError::err(&self.op, "community_moderator_already_exists"))?
      }
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: user_id
    };

    let _inserted_community_follower = match CommunityFollower::follow(&conn, &community_follower_form) {
      Ok(user) => user,
      Err(_e) => {
        return Err(APIError::err(&self.op, "community_follower_already_exists"))?
      }
    };

    let community_view = CommunityView::read(&conn, inserted_community.id, Some(user_id))?;

    Ok(
      CommunityResponse {
        op: self.op.to_string(),
        community: community_view
      }
      )
  }
}

impl Perform<CommunityResponse> for Oper<EditCommunity> {
  fn perform(&self) -> Result<CommunityResponse, Error> {
    let data: &EditCommunity = &self.data;

    if has_slurs(&data.name) || has_slurs(&data.title) {
      return Err(APIError::err(&self.op, "no_slurs"))?
    }

    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(APIError::err(&self.op, "not_logged_in"))?
      }
    };

    let user_id = claims.id;

    // Check for a site ban
    if UserView::read(&conn, user_id)?.banned {
      return Err(APIError::err(&self.op, "site_ban"))?
    }

    // Verify its a mod
    let mut editors: Vec<i32> = Vec::new();
    editors.append(
      &mut CommunityModeratorView::for_community(&conn, data.edit_id)
      ?
      .into_iter()
      .map(|m| m.user_id)
      .collect()
      );
    editors.append(
      &mut UserView::admins(&conn)
      ?
      .into_iter()
      .map(|a| a.id)
      .collect()
      );
    if !editors.contains(&user_id) {
      return Err(APIError::err(&self.op, "no_community_edit_allowed"))?
    }

    let community_form = CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      category_id: data.category_id.to_owned(),
      creator_id: user_id,
      removed: data.removed.to_owned(),
      deleted: data.deleted.to_owned(),
      updated: Some(naive_now())
    };

    let _updated_community = match Community::update(&conn, data.edit_id, &community_form) {
      Ok(community) => community,
      Err(_e) => {
        return Err(APIError::err(&self.op, "couldnt_update_community"))?
      }
    };

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
      let expires = match data.expires {
        Some(time) => Some(naive_from_unix(time)),
        None => None
      };
      let form = ModRemoveCommunityForm {
        mod_user_id: user_id,
        community_id: data.edit_id,
        removed: Some(removed),
        reason: data.reason.to_owned(),
        expires: expires
      };
      ModRemoveCommunity::create(&conn, &form)?;
    }

    let community_view = CommunityView::read(&conn, data.edit_id, Some(user_id))?;

    Ok(
      CommunityResponse {
        op: self.op.to_string(), 
        community: community_view
      }
      )
  }
}

impl Perform<ListCommunitiesResponse> for Oper<ListCommunities> {
  fn perform(&self) -> Result<ListCommunitiesResponse, Error> {
    let data: &ListCommunities = &self.data;
    let conn = establish_connection();

    let user_id: Option<i32> = match &data.auth {
      Some(auth) => {
        match Claims::decode(&auth) {
          Ok(claims) => {
            let user_id = claims.claims.id;
            Some(user_id)
          }
          Err(_e) => None
        }
      }
      None => None
    };

    let sort = SortType::from_str(&data.sort)?;

    let communities: Vec<CommunityView> = CommunityView::list(&conn, &sort, user_id, None, data.page, data.limit)?;

    // Return the jwt
    Ok(
      ListCommunitiesResponse {
        op: self.op.to_string(),
        communities: communities
      }
      )
  }
}


impl Perform<CommunityResponse> for Oper<FollowCommunity> {
  fn perform(&self) -> Result<CommunityResponse, Error> {
    let data: &FollowCommunity = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(APIError::err(&self.op, "not_logged_in"))?
      }
    };

    let user_id = claims.id;

    let community_follower_form = CommunityFollowerForm {
      community_id: data.community_id,
      user_id: user_id
    };

    if data.follow {
      match CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(APIError::err(&self.op, "community_follower_already_exists"))?
        }
      };
    } else {
      match CommunityFollower::ignore(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(APIError::err(&self.op, "community_follower_already_exists"))?
        }
      };
    }

    let community_view = CommunityView::read(&conn, data.community_id, Some(user_id))?;

    Ok(
      CommunityResponse {
        op: self.op.to_string(), 
        community: community_view
      }
      )
  }
}


impl Perform<GetFollowedCommunitiesResponse> for Oper<GetFollowedCommunities> {
  fn perform(&self) -> Result<GetFollowedCommunitiesResponse, Error> {
    let data: &GetFollowedCommunities = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(APIError::err(&self.op, "not_logged_in"))?
      }
    };

    let user_id = claims.id;

    let communities: Vec<CommunityFollowerView> = match CommunityFollowerView::for_user(&conn, user_id) {
      Ok(communities) => communities,
      Err(_e) => {
        return Err(APIError::err(&self.op, "system_err_login"))?
      }
    };

    // Return the jwt
    Ok(
      GetFollowedCommunitiesResponse {
        op: self.op.to_string(),
        communities: communities
      }
      )
  }
}


impl Perform<BanFromCommunityResponse> for Oper<BanFromCommunity> {
  fn perform(&self) -> Result<BanFromCommunityResponse, Error> {
    let data: &BanFromCommunity = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(APIError::err(&self.op, "not_logged_in"))?
      }
    };

    let user_id = claims.id;

    let community_user_ban_form = CommunityUserBanForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    if data.ban {
      match CommunityUserBan::ban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(APIError::err(&self.op, "community_user_already_banned"))?
        }
      };
    } else {
      match CommunityUserBan::unban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(APIError::err(&self.op, "community_user_already_banned"))?
        }
      };
    }

    // Mod tables
    let expires = match data.expires {
      Some(time) => Some(naive_from_unix(time)),
      None => None
    };

    let form = ModBanFromCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires: expires,
    };
    ModBanFromCommunity::create(&conn, &form)?;

    let user_view = UserView::read(&conn, data.user_id)?;

    Ok(
      BanFromCommunityResponse {
        op: self.op.to_string(), 
        user: user_view,
        banned: data.ban
      }
      )
  }
}

impl Perform<AddModToCommunityResponse> for Oper<AddModToCommunity> {
  fn perform(&self) -> Result<AddModToCommunityResponse, Error> {
    let data: &AddModToCommunity = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => {
        return Err(APIError::err(&self.op, "not_logged_in"))?
      }
    };

    let user_id = claims.id;

    let community_moderator_form = CommunityModeratorForm {
      community_id: data.community_id,
      user_id: data.user_id
    };

    if data.added {
      match CommunityModerator::join(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(APIError::err(&self.op, "community_moderator_already_exists"))?
        }
      };
    } else {
      match CommunityModerator::leave(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(APIError::err(&self.op, "community_moderator_already_exists"))?
        }
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

    Ok(
      AddModToCommunityResponse {
        op: self.op.to_string(), 
        moderators: moderators,
      }
      )
  }
}
