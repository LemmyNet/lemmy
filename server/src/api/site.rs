use std::str::FromStr;

use failure::Error;
use serde::{Deserialize, Serialize};

use crate::api::{Perform, self};
use crate::db::category;
use crate::db::comment_view;
use crate::db::community;
use crate::db::community_view;
use crate::db::moderator;
use crate::db::moderator_views;
use crate::db::post_view;
use crate::db::user;
use crate::db::user_view;
use crate::db::{
    Crud,
    SearchType,
    SortType,
    establish_connection,
};

#[derive(Serialize, Deserialize)]
pub struct ListCategories;

#[derive(Serialize, Deserialize)]
pub struct ListCategoriesResponse {
  op: String,
  categories: Vec<category::Category>,
}

#[derive(Serialize, Deserialize)]
pub struct Search {
  q: String,
  type_: String,
  community_id: Option<i32>,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
  op: String,
  type_: String,
  comments: Vec<comment_view::CommentView>,
  posts: Vec<post_view::PostView>,
  communities: Vec<community_view::CommunityView>,
  users: Vec<user_view::UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetModlog {
  mod_user_id: Option<i32>,
  community_id: Option<i32>,
  page: Option<i64>,
  limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct GetModlogResponse {
  op: String,
  removed_posts: Vec<moderator_views::ModRemovePostView>,
  locked_posts: Vec<moderator_views::ModLockPostView>,
  stickied_posts: Vec<moderator_views::ModStickyPostView>,
  removed_comments: Vec<moderator_views::ModRemoveCommentView>,
  removed_communities: Vec<moderator_views::ModRemoveCommunityView>,
  banned_from_community: Vec<moderator_views::ModBanFromCommunityView>,
  banned: Vec<moderator_views::ModBanView>,
  added_to_community: Vec<moderator_views::ModAddCommunityView>,
  added: Vec<moderator_views::ModAddView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSite {
  name: String,
  description: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditSite {
  name: String,
  description: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetSite;

#[derive(Serialize, Deserialize)]
pub struct SiteResponse {
  op: String,
  site: community_view::SiteView,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteResponse {
  op: String,
  site: Option<community_view::SiteView>,
  admins: Vec<user_view::UserView>,
  banned: Vec<user_view::UserView>,
  pub online: usize,
}

#[derive(Serialize, Deserialize)]
pub struct TransferSite {
  user_id: i32,
  auth: String,
}

impl Perform<ListCategoriesResponse> for api::Oper<ListCategories> {
  fn perform(&self) -> Result<ListCategoriesResponse, Error> {
    let _data: &ListCategories = &self.data;
    let conn = establish_connection();

    let categories: Vec<category::Category> = category::Category::list_all(&conn)?;

    // Return the jwt
    Ok(ListCategoriesResponse {
      op: self.op.to_string(),
      categories: categories,
    })
  }
}

impl Perform<GetModlogResponse> for api::Oper<GetModlog> {
  fn perform(&self) -> Result<GetModlogResponse, Error> {
    let data: &GetModlog = &self.data;
    let conn = establish_connection();

    let removed_posts = moderator_views::ModRemovePostView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let locked_posts = moderator_views::ModLockPostView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let stickied_posts = moderator_views::ModStickyPostView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let removed_comments = moderator_views::ModRemoveCommentView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let banned_from_community = moderator_views::ModBanFromCommunityView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let added_to_community = moderator_views::ModAddCommunityView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;

    // These arrays are only for the full modlog, when a community isn't given
    let mut removed_communities = Vec::new();
    let mut banned = Vec::new();
    let mut added = Vec::new();

    if data.community_id.is_none() {
      removed_communities =
        moderator_views::ModRemoveCommunityView::list(&conn, data.mod_user_id, data.page, data.limit)?;
      banned = moderator_views::ModBanView::list(&conn, data.mod_user_id, data.page, data.limit)?;
      added = moderator_views::ModAddView::list(&conn, data.mod_user_id, data.page, data.limit)?;
    }

    // Return the jwt
    Ok(GetModlogResponse {
      op: self.op.to_string(),
      removed_posts: removed_posts,
      locked_posts: locked_posts,
      stickied_posts: stickied_posts,
      removed_comments: removed_comments,
      removed_communities: removed_communities,
      banned_from_community: banned_from_community,
      banned: banned,
      added_to_community: added_to_community,
      added: added,
    })
  }
}

impl Perform<SiteResponse> for api::Oper<CreateSite> {
  fn perform(&self) -> Result<SiteResponse, Error> {
    let data: &CreateSite = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    if crate::has_slurs(&data.name)
      || (data.description.is_some() && crate::has_slurs(&data.description.to_owned().unwrap()))
    {
      return Err(api::APIError::err(&self.op, "no_slurs"))?;
    }

    let user_id = claims.id;

    // Make sure user is an admin
    if !user_view::UserView::read(&conn, user_id)?.admin {
      return Err(api::APIError::err(&self.op, "not_an_admin"))?;
    }

    let site_form = community::SiteForm {
      name: data.name.to_owned(),
      description: data.description.to_owned(),
      creator_id: user_id,
      updated: None,
    };

    match community::Site::create(&conn, &site_form) {
      Ok(site) => site,
      Err(_e) => return Err(api::APIError::err(&self.op, "site_already_exists"))?,
    };

    let site_view = community_view::SiteView::read(&conn)?;

    Ok(SiteResponse {
      op: self.op.to_string(),
      site: site_view,
    })
  }
}

impl Perform<SiteResponse> for api::Oper<EditSite> {
  fn perform(&self) -> Result<SiteResponse, Error> {
    let data: &EditSite = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    if crate::has_slurs(&data.name)
      || (data.description.is_some() && crate::has_slurs(&data.description.to_owned().unwrap()))
    {
      return Err(api::APIError::err(&self.op, "no_slurs"))?;
    }

    let user_id = claims.id;

    // Make sure user is an admin
    if user_view::UserView::read(&conn, user_id)?.admin == false {
      return Err(api::APIError::err(&self.op, "not_an_admin"))?;
    }

    let found_site = community::Site::read(&conn, 1)?;

    let site_form = community::SiteForm {
      name: data.name.to_owned(),
      description: data.description.to_owned(),
      creator_id: found_site.creator_id,
      updated: Some(crate::naive_now()),
    };

    match community::Site::update(&conn, 1, &site_form) {
      Ok(site) => site,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_site"))?,
    };

    let site_view = community_view::SiteView::read(&conn)?;

    Ok(SiteResponse {
      op: self.op.to_string(),
      site: site_view,
    })
  }
}

impl Perform<GetSiteResponse> for api::Oper<GetSite> {
  fn perform(&self) -> Result<GetSiteResponse, Error> {
    let _data: &GetSite = &self.data;
    let conn = establish_connection();

    // It can return a null site in order to redirect
    let site_view = match community::Site::read(&conn, 1) {
      Ok(_site) => Some(community_view::SiteView::read(&conn)?),
      Err(_e) => None,
    };

    let mut admins = user_view::UserView::admins(&conn)?;
    if site_view.is_some() {
      let site_creator_id = site_view.to_owned().unwrap().creator_id;
      let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
      let creator_user = admins.remove(creator_index);
      admins.insert(0, creator_user);
    }

    let banned = user_view::UserView::banned(&conn)?;

    Ok(GetSiteResponse {
      op: self.op.to_string(),
      site: site_view,
      admins: admins,
      banned: banned,
      online: 0
    })
  }
}

impl Perform<SearchResponse> for api::Oper<Search> {
  fn perform(&self) -> Result<SearchResponse, Error> {
    let data: &Search = &self.data;
    let conn = establish_connection();

    let sort = SortType::from_str(&data.sort)?;
    let type_ = SearchType::from_str(&data.type_)?;

    let mut posts = Vec::new();
    let mut comments = Vec::new();
    let mut communities = Vec::new();
    let mut users = Vec::new();

    // TODO no clean / non-nsfw searching rn

    match type_ {
      SearchType::Posts => {
        posts = post_view::PostView::list(
          &conn,
          post_view::PostListingType::All,
          &sort,
          data.community_id,
          None,
          Some(data.q.to_owned()),
          None,
          None,
          true,
          false,
          false,
          data.page,
          data.limit,
        )?;
      }
      SearchType::Comments => {
        comments = comment_view::CommentView::list(
          &conn,
          &sort,
          None,
          None,
          Some(data.q.to_owned()),
          None,
          false,
          data.page,
          data.limit,
        )?;
      }
      SearchType::Communities => {
        communities = community_view::CommunityView::list(
          &conn,
          &sort,
          None,
          true,
          Some(data.q.to_owned()),
          data.page,
          data.limit,
        )?;
      }
      SearchType::Users => {
        users = user_view::UserView::list(&conn, &sort, Some(data.q.to_owned()), data.page, data.limit)?;
      }
      SearchType::All => {
        posts = post_view::PostView::list(
          &conn,
          post_view::PostListingType::All,
          &sort,
          data.community_id,
          None,
          Some(data.q.to_owned()),
          None,
          None,
          true,
          false,
          false,
          data.page,
          data.limit,
        )?;
        comments = comment_view::CommentView::list(
          &conn,
          &sort,
          None,
          None,
          Some(data.q.to_owned()),
          None,
          false,
          data.page,
          data.limit,
        )?;
        communities = community_view::CommunityView::list(
          &conn,
          &sort,
          None,
          true,
          Some(data.q.to_owned()),
          data.page,
          data.limit,
        )?;
        users = user_view::UserView::list(&conn, &sort, Some(data.q.to_owned()), data.page, data.limit)?;
      }
      SearchType::Url => {
        posts = post_view::PostView::list(
          &conn,
          post_view::PostListingType::All,
          &sort,
          data.community_id,
          None,
          None,
          Some(data.q.to_owned()),
          None,
          true,
          false,
          false,
          data.page,
          data.limit,
        )?;
      }
    };

    // Return the jwt
    Ok(SearchResponse {
      op: self.op.to_string(),
      type_: data.type_.to_owned(),
      comments: comments,
      posts: posts,
      communities: communities,
      users: users,
    })
  }
}

impl Perform<GetSiteResponse> for api::Oper<TransferSite> {
  fn perform(&self) -> Result<GetSiteResponse, Error> {
    let data: &TransferSite = &self.data;
    let conn = establish_connection();

    let claims = match user::Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(api::APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let read_site = community::Site::read(&conn, 1)?;

    // Make sure user is the creator
    if read_site.creator_id != user_id {
      return Err(api::APIError::err(&self.op, "not_an_admin"))?;
    }

    let site_form = community::SiteForm {
      name: read_site.name,
      description: read_site.description,
      creator_id: data.user_id,
      updated: Some(crate::naive_now()),
    };

    match community::Site::update(&conn, 1, &site_form) {
      Ok(site) => site,
      Err(_e) => return Err(api::APIError::err(&self.op, "couldnt_update_site"))?,
    };

    // Mod tables
    let form = moderator::ModAddForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      removed: Some(false),
    };

    moderator::ModAdd::create(&conn, &form)?;

    let site_view = community_view::SiteView::read(&conn)?;

    let mut admins = user_view::UserView::admins(&conn)?;
    let creator_index = admins
      .iter()
      .position(|r| r.id == site_view.creator_id)
      .unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    let banned = user_view::UserView::banned(&conn)?;

    Ok(GetSiteResponse {
      op: self.op.to_string(),
      site: Some(site_view),
      admins: admins,
      banned: banned,
      online: 0
    })
  }
}
