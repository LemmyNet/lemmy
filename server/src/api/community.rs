use std::str::FromStr;

use failure::Error;
use serde::{Deserialize, Serialize};

use crate::api::{Perform, self};
use crate::db::community;
use crate::db::community_view;
use crate::db::moderator;
use crate::db::user;
use crate::db::user_view;
use crate::db::{
    Bannable,
    Crud,
    Followable,
    Joinable,
    SortType,
    establish_connection,
};

#[derive(Serialize, Deserialize)]
pub struct GetCommunity {
  id: Option<i32>,
  name: Option<String>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunityResponse {
  op: String,
  community: community_view::CommunityView,
  moderators: Vec<community_view::CommunityModeratorView>,
  admins: Vec<user_view::UserView>,
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
  op: String,
  pub community: community_view::CommunityView,
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
  op: String,
  communities: Vec<community_view::CommunityView>,
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

#[derive(Serialize, Deserialize)]
pub struct BanFromCommunityResponse {
  op: String,
  user: user_view::UserView,
  banned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AddModToCommunity {
  pub community_id: i32,
  user_id: i32,
  added: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddModToCommunityResponse {
  op: String,
  moderators: Vec<community_view::CommunityModeratorView>,
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
  op: String,
  communities: Vec<community_view::CommunityFollowerView>,
}

#[derive(Serialize, Deserialize)]
pub struct TransferCommunity {
  community_id: i32,
  user_id: i32,
  auth: String,
}

impl Perform<GetCommunityResponse> for api::Oper<GetCommunity> {
  fn perform(&self) -> Result<GetCommunityResponse, Error> {
    let data: &GetCommunity = &self.data;
    let conn = establish_connection();

    let user_id: Option<i32> = match &data.auth {
      Some(auth) => match user::Claims::decode(&auth) {
        Ok(claims) => {
          let user_id = claims.claims.id;
          Some(user_id)
        }
        Err(_e) => None,
      },
      None => None,
    };

    let community_id = match data.id {
      Some(id) => id,
      None => {
        community::Community::read_from_name(&conn, data.name.to_owned().unwrap_or("main".to_string()))?.id
      }
    };

    let community_view = match community_view::CommunityView::read(&conn, community_id, user_id) {
      Ok(community) => community,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_find_community"))?,
    };

    let moderators = match community_view::CommunityModeratorView::for_community(&conn, community_id) {
      Ok(moderators) => moderators,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_find_community"))?,
    };

    let site_creator_id = community::Site::read(&conn, 1)?.creator_id;
    let mut admins = user_view::UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    // Return the jwt
    Ok(GetCommunityResponse {
      op: self.op.to_string(),
      community: community_view,
      moderators: moderators,
      admins: admins,
    })
  }
}

impl Perform<CommunityResponse> for api::Oper<CreateCommunity> {
  fn perform(&self) -> Result<CommunityResponse, Error> {
    let data: &CreateCommunity = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    if crate::has_slurs(&data.name)
      || crate::has_slurs(&data.title)
      || (data.description.is_some() && crate::has_slurs(&data.description.to_owned().unwrap()))
    {
      return Err(api::APIError::err(&self.op, "no_slurs"))?;
    }

    let user_id = claims.id;

    // Check for a site ban
    if user_view::UserView::read(&conn, user_id)?.banned {
      return Err(api::APIError::err(&self.op, "site_ban"))?;
    }

    // When you create a community, make sure the user becomes a moderator and a follower
    let community_form = community::CommunityForm {
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

    let inserted_community = match community::Community::create(&conn, &community_form) {
      Ok(community) => community,
      Err(_e) => return Err(api::APIError::err(&self.op, "community_already_exists"))?,
    };

    let community_moderator_form = community::CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id: user_id,
    };

    let _inserted_community_moderator =
      match community::CommunityModerator::join(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(api::APIError::err(
            &self.op,
            "community_moderator_already_exists",
          ))?
        }
      };

    let community_follower_form = community::CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: user_id,
    };

    let _inserted_community_follower =
      match community::CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(api::APIError::err(&self.op, "community_follower_already_exists"))?,
      };

    let community_view = community_view::CommunityView::read(&conn, inserted_community.id, Some(user_id))?;

    Ok(CommunityResponse {
      op: self.op.to_string(),
      community: community_view,
    })
  }
}

impl Perform<CommunityResponse> for api::Oper<EditCommunity> {
  fn perform(&self) -> Result<CommunityResponse, Error> {
    let data: &EditCommunity = &self.data;

    if crate::has_slurs(&data.name) || crate::has_slurs(&data.title) {
      return Err(api::APIError::err(&self.op, "no_slurs"))?;
    }

    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    // Check for a site ban
    if user_view::UserView::read(&conn, user_id)?.banned {
      return Err(api::APIError::err(&self.op, "site_ban"))?;
    }

    // Verify its a mod
    let mut editors: Vec<i32> = Vec::new();
    editors.append(
      &mut community_view::CommunityModeratorView::for_community(&conn, data.edit_id)?
        .into_iter()
        .map(|m| m.user_id)
        .collect(),
    );
    editors.append(&mut user_view::UserView::admins(&conn)?.into_iter().map(|a| a.id).collect());
    if !editors.contains(&user_id) {
      return Err(api::APIError::err(&self.op, "no_community_edit_allowed"))?;
    }

    let community_form = community::CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      category_id: data.category_id.to_owned(),
      creator_id: user_id,
      removed: data.removed.to_owned(),
      deleted: data.deleted.to_owned(),
      nsfw: data.nsfw,
      updated: Some(crate::naive_now()),
    };

    let _updated_community = match community::Community::update(&conn, data.edit_id, &community_form) {
      Ok(community) => community,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_community"))?,
    };

    // Mod tables
    if let Some(removed) = data.removed.to_owned() {
      let expires = match data.expires {
        Some(time) => Some(crate::naive_from_unix(time)),
        None => None,
      };
      let form = moderator::ModRemoveCommunityForm {
        mod_user_id: user_id,
        community_id: data.edit_id,
        removed: Some(removed),
        reason: data.reason.to_owned(),
        expires: expires,
      };
      moderator::ModRemoveCommunity::create(&conn, &form)?;
    }

    let community_view = community_view::CommunityView::read(&conn, data.edit_id, Some(user_id))?;

    Ok(CommunityResponse {
      op: self.op.to_string(),
      community: community_view,
    })
  }
}

impl Perform<ListCommunitiesResponse> for api::Oper<ListCommunities> {
  fn perform(&self) -> Result<ListCommunitiesResponse, Error> {
    let data: &ListCommunities = &self.data;
    let conn = establish_connection();

    let user_claims: Option<user::Claims> = match &data.auth {
      Some(auth) => match user::Claims::decode(&auth) {
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

    let communities: Vec<community_view::CommunityView> = community_view::CommunityView::list(
      &conn, &sort, user_id, show_nsfw, None, data.page, data.limit,
    )?;

    // Return the jwt
    Ok(ListCommunitiesResponse {
      op: self.op.to_string(),
      communities: communities,
    })
  }
}

impl Perform<CommunityResponse> for api::Oper<FollowCommunity> {
  fn perform(&self) -> Result<CommunityResponse, Error> {
    let data: &FollowCommunity = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let community_follower_form = community::CommunityFollowerForm {
      community_id: data.community_id,
      user_id: user_id,
    };

    if data.follow {
      match community::CommunityFollower::follow(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(api::APIError::err(&self.op, "community_follower_already_exists"))?,
      };
    } else {
      match community::CommunityFollower::ignore(&conn, &community_follower_form) {
        Ok(user) => user,
        Err(_e) => return Err(api::APIError::err(&self.op, "community_follower_already_exists"))?,
      };
    }

    let community_view = community_view::CommunityView::read(&conn, data.community_id, Some(user_id))?;

    Ok(CommunityResponse {
      op: self.op.to_string(),
      community: community_view,
    })
  }
}

impl Perform<GetFollowedCommunitiesResponse> for api::Oper<GetFollowedCommunities> {
  fn perform(&self) -> Result<GetFollowedCommunitiesResponse, Error> {
    let data: &GetFollowedCommunities = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let communities: Vec<community_view::CommunityFollowerView> =
      match community_view::CommunityFollowerView::for_user(&conn, user_id) {
        Ok(communities) => communities,
        Err(_e) => return Err(api::APIError::err(&self.op, "system_err_login"))?,
      };

    // Return the jwt
    Ok(GetFollowedCommunitiesResponse {
      op: self.op.to_string(),
      communities: communities,
    })
  }
}

impl Perform<BanFromCommunityResponse> for api::Oper<BanFromCommunity> {
  fn perform(&self) -> Result<BanFromCommunityResponse, Error> {
    let data: &BanFromCommunity = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let community_user_ban_form = community::CommunityUserBanForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    if data.ban {
      match community::CommunityUserBan::ban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => return Err(api::APIError::err(&self.op, "community_user_already_banned"))?,
      };
    } else {
      match community::CommunityUserBan::unban(&conn, &community_user_ban_form) {
        Ok(user) => user,
        Err(_e) => return Err(api::APIError::err(&self.op, "community_user_already_banned"))?,
      };
    }

    // Mod tables
    let expires = match data.expires {
      Some(time) => Some(crate::naive_from_unix(time)),
      None => None,
    };

    let form = moderator::ModBanFromCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      reason: data.reason.to_owned(),
      banned: Some(data.ban),
      expires: expires,
    };
    moderator::ModBanFromCommunity::create(&conn, &form)?;

    let user_view = user_view::UserView::read(&conn, data.user_id)?;

    Ok(BanFromCommunityResponse {
      op: self.op.to_string(),
      user: user_view,
      banned: data.ban,
    })
  }
}

impl Perform<AddModToCommunityResponse> for api::Oper<AddModToCommunity> {
  fn perform(&self) -> Result<AddModToCommunityResponse, Error> {
    let data: &AddModToCommunity = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let community_moderator_form = community::CommunityModeratorForm {
      community_id: data.community_id,
      user_id: data.user_id,
    };

    if data.added {
      match community::CommunityModerator::join(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(api::APIError::err(
            &self.op,
            "community_moderator_already_exists",
          ))?
        }
      };
    } else {
      match community::CommunityModerator::leave(&conn, &community_moderator_form) {
        Ok(user) => user,
        Err(_e) => {
          return Err(api::APIError::err(
            &self.op,
            "community_moderator_already_exists",
          ))?
        }
      };
    }

    // Mod tables
    let form = moderator::ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(!data.added),
    };
    moderator::ModAddCommunity::create(&conn, &form)?;

    let moderators = community_view::CommunityModeratorView::for_community(&conn, data.community_id)?;

    Ok(AddModToCommunityResponse {
      op: self.op.to_string(),
      moderators: moderators,
    })
  }
}

impl Perform<GetCommunityResponse> for api::Oper<TransferCommunity> {
  fn perform(&self) -> Result<GetCommunityResponse, Error> {
    let data: &TransferCommunity = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let read_community = community::Community::read(&conn, data.community_id)?;

    let site_creator_id = community::Site::read(&conn, 1)?.creator_id;
    let mut admins = user_view::UserView::admins(&conn)?;
    let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    // Make sure user is the creator, or an admin
    if user_id != read_community.creator_id
      && !admins
        .iter()
        .map(|a| a.id)
        .collect::<Vec<i32>>()
        .contains(&user_id)
    {
      return Err(api::APIError::err(&self.op, "not_an_admin"))?;
    }

    let community_form = community::CommunityForm {
      name: read_community.name,
      title: read_community.title,
      description: read_community.description,
      category_id: read_community.category_id,
      creator_id: data.user_id,
      removed: None,
      deleted: None,
      nsfw: read_community.nsfw,
      updated: Some(crate::naive_now()),
    };

    let _updated_community = match community::Community::update(&conn, data.community_id, &community_form) {
      Ok(community) => community,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_community"))?,
    };

    // You also have to re-do the community_moderator table, reordering it.
    let mut community_mods = community_view::CommunityModeratorView::for_community(&conn, data.community_id)?;
    let creator_index = community_mods
      .iter()
      .position(|r| r.user_id == data.user_id)
      .unwrap();
    let creator_user = community_mods.remove(creator_index);
    community_mods.insert(0, creator_user);

    community::CommunityModerator::delete_for_community(&conn, data.community_id)?;

    for cmod in &community_mods {
      let community_moderator_form = community::CommunityModeratorForm {
        community_id: cmod.community_id,
        user_id: cmod.user_id,
      };

      let _inserted_community_moderator =
        match community::CommunityModerator::join(&conn, &community_moderator_form) {
          Ok(user) => user,
          Err(_e) => {
            return Err(api::APIError::err(
              &self.op,
              "community_moderator_already_exists",
            ))?
          }
        };
    }

    // Mod tables
    let form = moderator::ModAddCommunityForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      community_id: data.community_id,
      removed: Some(false),
    };
    moderator::ModAddCommunity::create(&conn, &form)?;

    let community_view = match community_view::CommunityView::read(&conn, data.community_id, Some(user_id)) {
      Ok(community) => community,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_find_community"))?,
    };

    let moderators = match community_view::CommunityModeratorView::for_community(&conn, data.community_id) {
      Ok(moderators) => moderators,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_find_community"))?,
    };

    // Return the jwt
    Ok(GetCommunityResponse {
      op: self.op.to_string(),
      community: community_view,
      moderators: moderators,
      admins: admins,
    })
  }
}
