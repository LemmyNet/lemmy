use super::*;
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct ListCategories;

#[derive(Serialize, Deserialize)]
pub struct ListCategoriesResponse {
  op: String,
  categories: Vec<Category>,
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
  comments: Vec<CommentView>,
  posts: Vec<PostView>,
  communities: Vec<CommunityView>,
  users: Vec<UserView>,
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
  removed_posts: Vec<ModRemovePostView>,
  locked_posts: Vec<ModLockPostView>,
  stickied_posts: Vec<ModStickyPostView>,
  removed_comments: Vec<ModRemoveCommentView>,
  removed_communities: Vec<ModRemoveCommunityView>,
  banned_from_community: Vec<ModBanFromCommunityView>,
  banned: Vec<ModBanView>,
  added_to_community: Vec<ModAddCommunityView>,
  added: Vec<ModAddView>,
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
  site: SiteView,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteResponse {
  op: String,
  site: Option<SiteView>,
  admins: Vec<UserView>,
  banned: Vec<UserView>,
  pub online: usize,
}

#[derive(Serialize, Deserialize)]
pub struct TransferSite {
  user_id: i32,
  auth: String,
}

impl Perform<ListCategoriesResponse> for Oper<ListCategories> {
  fn perform(&self) -> Result<ListCategoriesResponse, Error> {
    let _data: &ListCategories = &self.data;
    let conn = establish_connection();

    let categories: Vec<Category> = Category::list_all(&conn)?;

    // Return the jwt
    Ok(ListCategoriesResponse {
      op: self.op.to_string(),
      categories: categories,
    })
  }
}

impl Perform<GetModlogResponse> for Oper<GetModlog> {
  fn perform(&self) -> Result<GetModlogResponse, Error> {
    let data: &GetModlog = &self.data;
    let conn = establish_connection();

    let removed_posts = ModRemovePostView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let locked_posts = ModLockPostView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let stickied_posts = ModStickyPostView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let removed_comments = ModRemoveCommentView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let banned_from_community = ModBanFromCommunityView::list(
      &conn,
      data.community_id,
      data.mod_user_id,
      data.page,
      data.limit,
    )?;
    let added_to_community = ModAddCommunityView::list(
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
        ModRemoveCommunityView::list(&conn, data.mod_user_id, data.page, data.limit)?;
      banned = ModBanView::list(&conn, data.mod_user_id, data.page, data.limit)?;
      added = ModAddView::list(&conn, data.mod_user_id, data.page, data.limit)?;
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

impl Perform<SiteResponse> for Oper<CreateSite> {
  fn perform(&self) -> Result<SiteResponse, Error> {
    let data: &CreateSite = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in"))?,
    };

    if has_slurs(&data.name)
      || (data.description.is_some() && has_slurs(&data.description.to_owned().unwrap()))
    {
      return Err(APIError::err(&self.op, "no_slurs"))?;
    }

    let user_id = claims.id;

    // Make sure user is an admin
    if !UserView::read(&conn, user_id)?.admin {
      return Err(APIError::err(&self.op, "not_an_admin"))?;
    }

    let site_form = SiteForm {
      name: data.name.to_owned(),
      description: data.description.to_owned(),
      creator_id: user_id,
      updated: None,
    };

    match Site::create(&conn, &site_form) {
      Ok(site) => site,
      Err(_e) => return Err(APIError::err(&self.op, "site_already_exists"))?,
    };

    let site_view = SiteView::read(&conn)?;

    Ok(SiteResponse {
      op: self.op.to_string(),
      site: site_view,
    })
  }
}

impl Perform<SiteResponse> for Oper<EditSite> {
  fn perform(&self) -> Result<SiteResponse, Error> {
    let data: &EditSite = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in"))?,
    };

    if has_slurs(&data.name)
      || (data.description.is_some() && has_slurs(&data.description.to_owned().unwrap()))
    {
      return Err(APIError::err(&self.op, "no_slurs"))?;
    }

    let user_id = claims.id;

    // Make sure user is an admin
    if UserView::read(&conn, user_id)?.admin == false {
      return Err(APIError::err(&self.op, "not_an_admin"))?;
    }

    let found_site = Site::read(&conn, 1)?;

    let site_form = SiteForm {
      name: data.name.to_owned(),
      description: data.description.to_owned(),
      creator_id: found_site.creator_id,
      updated: Some(naive_now()),
    };

    match Site::update(&conn, 1, &site_form) {
      Ok(site) => site,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_site"))?,
    };

    let site_view = SiteView::read(&conn)?;

    Ok(SiteResponse {
      op: self.op.to_string(),
      site: site_view,
    })
  }
}

impl Perform<GetSiteResponse> for Oper<GetSite> {
  fn perform(&self) -> Result<GetSiteResponse, Error> {
    let _data: &GetSite = &self.data;
    let conn = establish_connection();

    // It can return a null site in order to redirect
    let site_view = match Site::read(&conn, 1) {
      Ok(_site) => Some(SiteView::read(&conn)?),
      Err(_e) => None,
    };

    let mut admins = UserView::admins(&conn)?;
    if site_view.is_some() {
      let site_creator_id = site_view.to_owned().unwrap().creator_id;
      let creator_index = admins.iter().position(|r| r.id == site_creator_id).unwrap();
      let creator_user = admins.remove(creator_index);
      admins.insert(0, creator_user);
    }

    let banned = UserView::banned(&conn)?;

    Ok(GetSiteResponse {
      op: self.op.to_string(),
      site: site_view,
      admins: admins,
      banned: banned,
      online: 0,
    })
  }
}

impl Perform<SearchResponse> for Oper<Search> {
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
        posts = PostQueryBuilder::create(&conn)
          .sort(&sort)
          .show_nsfw(true)
          .for_community_id_optional(data.community_id)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;
      }
      SearchType::Comments => {
        comments = CommentQueryBuilder::create(&conn)
          .sort(&sort)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;
      }
      SearchType::Communities => {
        communities = CommunityQueryBuilder::create(&conn)
          .sort(&sort)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;
      }
      SearchType::Users => {
        users = UserQueryBuilder::create(&conn)
          .sort(&sort)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;
      }
      SearchType::All => {
        posts = PostQueryBuilder::create(&conn)
          .sort(&sort)
          .show_nsfw(true)
          .for_community_id_optional(data.community_id)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;

        comments = CommentQueryBuilder::create(&conn)
          .sort(&sort)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;

        communities = CommunityQueryBuilder::create(&conn)
          .sort(&sort)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;

        users = UserQueryBuilder::create(&conn)
          .sort(&sort)
          .search_term(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;
      }
      SearchType::Url => {
        posts = PostQueryBuilder::create(&conn)
          .sort(&sort)
          .show_nsfw(true)
          .for_community_id_optional(data.community_id)
          .url_search(data.q.to_owned())
          .page_optional(data.page)
          .limit_optional(data.limit)
          .list()?;
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

impl Perform<GetSiteResponse> for Oper<TransferSite> {
  fn perform(&self) -> Result<GetSiteResponse, Error> {
    let data: &TransferSite = &self.data;
    let conn = establish_connection();

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err(&self.op, "not_logged_in"))?,
    };

    let user_id = claims.id;

    let read_site = Site::read(&conn, 1)?;

    // Make sure user is the creator
    if read_site.creator_id != user_id {
      return Err(APIError::err(&self.op, "not_an_admin"))?;
    }

    let site_form = SiteForm {
      name: read_site.name,
      description: read_site.description,
      creator_id: data.user_id,
      updated: Some(naive_now()),
    };

    match Site::update(&conn, 1, &site_form) {
      Ok(site) => site,
      Err(_e) => return Err(APIError::err(&self.op, "couldnt_update_site"))?,
    };

    // Mod tables
    let form = ModAddForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      removed: Some(false),
    };

    ModAdd::create(&conn, &form)?;

    let site_view = SiteView::read(&conn)?;

    let mut admins = UserView::admins(&conn)?;
    let creator_index = admins
      .iter()
      .position(|r| r.id == site_view.creator_id)
      .unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    let banned = UserView::banned(&conn)?;

    Ok(GetSiteResponse {
      op: self.op.to_string(),
      site: Some(site_view),
      admins: admins,
      banned: banned,
      online: 0,
    })
  }
}
