use super::user::Register;
use crate::{
  api::{APIError, Oper, Perform},
  apub::fetcher::search_by_apub_id,
  blocking,
  db::{
    category::*,
    comment_view::*,
    community_view::*,
    moderator::*,
    moderator_views::*,
    post_view::*,
    site::*,
    site_view::*,
    user::*,
    user_view::*,
    Crud,
    SearchType,
    SortType,
  },
  naive_now,
  settings::Settings,
  slur_check,
  slurs_vec_to_str,
  websocket::{server::SendAllMessage, UserOperation, WebsocketInfo},
  DbPool,
  LemmyError,
};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct ListCategories {}

#[derive(Serialize, Deserialize)]
pub struct ListCategoriesResponse {
  categories: Vec<Category>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
  q: String,
  type_: String,
  community_id: Option<i32>,
  sort: String,
  page: Option<i64>,
  limit: Option<i64>,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
  pub type_: String,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub communities: Vec<CommunityView>,
  pub users: Vec<UserView>,
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
  pub name: String,
  pub description: Option<String>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditSite {
  name: String,
  description: Option<String>,
  enable_downvotes: bool,
  open_registration: bool,
  enable_nsfw: bool,
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetSite {}

#[derive(Serialize, Deserialize, Clone)]
pub struct SiteResponse {
  site: SiteView,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteResponse {
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

#[derive(Serialize, Deserialize)]
pub struct GetSiteConfig {
  auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteConfigResponse {
  config_hjson: String,
}

#[derive(Serialize, Deserialize)]
pub struct SaveSiteConfig {
  config_hjson: String,
  auth: String,
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<ListCategories> {
  type Response = ListCategoriesResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<ListCategoriesResponse, LemmyError> {
    let _data: &ListCategories = &self.data;

    let categories = blocking(pool, move |conn| Category::list_all(conn)).await??;

    // Return the jwt
    Ok(ListCategoriesResponse { categories })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetModlog> {
  type Response = GetModlogResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetModlogResponse, LemmyError> {
    let data: &GetModlog = &self.data;

    let community_id = data.community_id;
    let mod_user_id = data.mod_user_id;
    let page = data.page;
    let limit = data.limit;
    let removed_posts = blocking(pool, move |conn| {
      ModRemovePostView::list(conn, community_id, mod_user_id, page, limit)
    })
    .await??;

    let locked_posts = blocking(pool, move |conn| {
      ModLockPostView::list(conn, community_id, mod_user_id, page, limit)
    })
    .await??;

    let stickied_posts = blocking(pool, move |conn| {
      ModStickyPostView::list(conn, community_id, mod_user_id, page, limit)
    })
    .await??;

    let removed_comments = blocking(pool, move |conn| {
      ModRemoveCommentView::list(conn, community_id, mod_user_id, page, limit)
    })
    .await??;

    let banned_from_community = blocking(pool, move |conn| {
      ModBanFromCommunityView::list(conn, community_id, mod_user_id, page, limit)
    })
    .await??;

    let added_to_community = blocking(pool, move |conn| {
      ModAddCommunityView::list(conn, community_id, mod_user_id, page, limit)
    })
    .await??;

    // These arrays are only for the full modlog, when a community isn't given
    let (removed_communities, banned, added) = if data.community_id.is_none() {
      blocking(pool, move |conn| {
        Ok((
          ModRemoveCommunityView::list(conn, mod_user_id, page, limit)?,
          ModBanView::list(conn, mod_user_id, page, limit)?,
          ModAddView::list(conn, mod_user_id, page, limit)?,
        )) as Result<_, LemmyError>
      })
      .await??
    } else {
      (Vec::new(), Vec::new(), Vec::new())
    };

    // Return the jwt
    Ok(GetModlogResponse {
      removed_posts,
      locked_posts,
      stickied_posts,
      removed_comments,
      removed_communities,
      banned_from_community,
      banned,
      added_to_community,
      added,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreateSite> {
  type Response = SiteResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<SiteResponse, LemmyError> {
    let data: &CreateSite = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    if let Err(slurs) = slur_check(&data.name) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    if let Some(description) = &data.description {
      if let Err(slurs) = slur_check(description) {
        return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
      }
    }

    let user_id = claims.id;

    // Make sure user is an admin
    let user = blocking(pool, move |conn| UserView::read(conn, user_id)).await??;
    if !user.admin {
      return Err(APIError::err("not_an_admin").into());
    }

    let site_form = SiteForm {
      name: data.name.to_owned(),
      description: data.description.to_owned(),
      creator_id: user_id,
      enable_downvotes: data.enable_downvotes,
      open_registration: data.open_registration,
      enable_nsfw: data.enable_nsfw,
      updated: None,
    };

    let create_site = move |conn: &'_ _| Site::create(conn, &site_form);
    if blocking(pool, create_site).await?.is_err() {
      return Err(APIError::err("site_already_exists").into());
    }

    let site_view = blocking(pool, move |conn| SiteView::read(conn)).await??;

    Ok(SiteResponse { site: site_view })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<EditSite> {
  type Response = SiteResponse;
  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<SiteResponse, LemmyError> {
    let data: &EditSite = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    if let Err(slurs) = slur_check(&data.name) {
      return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
    }

    if let Some(description) = &data.description {
      if let Err(slurs) = slur_check(description) {
        return Err(APIError::err(&slurs_vec_to_str(slurs)).into());
      }
    }

    let user_id = claims.id;

    // Make sure user is an admin
    let user = blocking(pool, move |conn| UserView::read(conn, user_id)).await??;
    if !user.admin {
      return Err(APIError::err("not_an_admin").into());
    }

    let found_site = blocking(pool, move |conn| Site::read(conn, 1)).await??;

    let site_form = SiteForm {
      name: data.name.to_owned(),
      description: data.description.to_owned(),
      creator_id: found_site.creator_id,
      updated: Some(naive_now()),
      enable_downvotes: data.enable_downvotes,
      open_registration: data.open_registration,
      enable_nsfw: data.enable_nsfw,
    };

    let update_site = move |conn: &'_ _| Site::update(conn, 1, &site_form);
    if blocking(pool, update_site).await?.is_err() {
      return Err(APIError::err("couldnt_update_site").into());
    }

    let site_view = blocking(pool, move |conn| SiteView::read(conn)).await??;

    let res = SiteResponse { site: site_view };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendAllMessage {
        op: UserOperation::EditSite,
        response: res.clone(),
        my_id: ws.id,
      });
    }

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetSite> {
  type Response = GetSiteResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let _data: &GetSite = &self.data;

    // TODO refactor this a little
    let res = blocking(pool, move |conn| Site::read(conn, 1)).await?;
    let site_view = if res.is_ok() {
      Some(blocking(pool, move |conn| SiteView::read(conn)).await??)
    } else if let Some(setup) = Settings::get().setup.as_ref() {
      let register = Register {
        username: setup.admin_username.to_owned(),
        email: setup.admin_email.to_owned(),
        password: setup.admin_password.to_owned(),
        password_verify: setup.admin_password.to_owned(),
        admin: true,
        show_nsfw: true,
      };
      let login_response = Oper::new(register, self.client.clone())
        .perform(pool, websocket_info.clone())
        .await?;
      info!("Admin {} created", setup.admin_username);

      let create_site = CreateSite {
        name: setup.site_name.to_owned(),
        description: None,
        enable_downvotes: true,
        open_registration: true,
        enable_nsfw: true,
        auth: login_response.jwt,
      };
      Oper::new(create_site, self.client.clone())
        .perform(pool, websocket_info.clone())
        .await?;
      info!("Site {} created", setup.site_name);
      Some(blocking(pool, move |conn| SiteView::read(conn)).await??)
    } else {
      None
    };

    let mut admins = blocking(pool, move |conn| UserView::admins(conn)).await??;

    // Make sure the site creator is the top admin
    if let Some(site_view) = site_view.to_owned() {
      let site_creator_id = site_view.creator_id;
      // TODO investigate why this is sometimes coming back null
      // Maybe user_.admin isn't being set to true?
      if let Some(creator_index) = admins.iter().position(|r| r.id == site_creator_id) {
        let creator_user = admins.remove(creator_index);
        admins.insert(0, creator_user);
      }
    }

    let banned = blocking(pool, move |conn| UserView::banned(conn)).await??;

    let online = if let Some(_ws) = websocket_info {
      // TODO
      1
    // let fut = async {
    //   ws.chatserver.send(GetUsersOnline).await.unwrap()
    // };
    // Runtime::new().unwrap().block_on(fut)
    } else {
      0
    };

    Ok(GetSiteResponse {
      site: site_view,
      admins,
      banned,
      online,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<Search> {
  type Response = SearchResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<SearchResponse, LemmyError> {
    let data: &Search = &self.data;

    dbg!(&data);

    match search_by_apub_id(&data.q, &self.client, pool).await {
      Ok(r) => return Ok(r),
      Err(e) => debug!("Failed to resolve search query as activitypub ID: {}", e),
    }

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

    let type_ = SearchType::from_str(&data.type_)?;

    let mut posts = Vec::new();
    let mut comments = Vec::new();
    let mut communities = Vec::new();
    let mut users = Vec::new();

    // TODO no clean / non-nsfw searching rn

    let q = data.q.to_owned();
    let page = data.page;
    let limit = data.limit;
    let sort = SortType::from_str(&data.sort)?;
    let community_id = data.community_id;
    match type_ {
      SearchType::Posts => {
        posts = blocking(pool, move |conn| {
          PostQueryBuilder::create(conn)
            .sort(&sort)
            .show_nsfw(true)
            .for_community_id(community_id)
            .search_term(q)
            .my_user_id(user_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Comments => {
        comments = blocking(pool, move |conn| {
          CommentQueryBuilder::create(&conn)
            .sort(&sort)
            .search_term(q)
            .my_user_id(user_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Communities => {
        communities = blocking(pool, move |conn| {
          CommunityQueryBuilder::create(conn)
            .sort(&sort)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Users => {
        users = blocking(pool, move |conn| {
          UserQueryBuilder::create(conn)
            .sort(&sort)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::All => {
        posts = blocking(pool, move |conn| {
          PostQueryBuilder::create(conn)
            .sort(&sort)
            .show_nsfw(true)
            .for_community_id(community_id)
            .search_term(q)
            .my_user_id(user_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;

        let q = data.q.to_owned();
        let sort = SortType::from_str(&data.sort)?;

        comments = blocking(pool, move |conn| {
          CommentQueryBuilder::create(conn)
            .sort(&sort)
            .search_term(q)
            .my_user_id(user_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;

        let q = data.q.to_owned();
        let sort = SortType::from_str(&data.sort)?;

        communities = blocking(pool, move |conn| {
          CommunityQueryBuilder::create(conn)
            .sort(&sort)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;

        let q = data.q.to_owned();
        let sort = SortType::from_str(&data.sort)?;

        users = blocking(pool, move |conn| {
          UserQueryBuilder::create(conn)
            .sort(&sort)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Url => {
        posts = blocking(pool, move |conn| {
          PostQueryBuilder::create(conn)
            .sort(&sort)
            .show_nsfw(true)
            .for_community_id(community_id)
            .url_search(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
    };

    // Return the jwt
    Ok(SearchResponse {
      type_: data.type_.to_owned(),
      comments,
      posts,
      communities,
      users,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<TransferSite> {
  type Response = GetSiteResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let data: &TransferSite = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    let read_site = blocking(pool, move |conn| Site::read(conn, 1)).await??;

    // Make sure user is the creator
    if read_site.creator_id != user_id {
      return Err(APIError::err("not_an_admin").into());
    }

    let site_form = SiteForm {
      name: read_site.name,
      description: read_site.description,
      creator_id: data.user_id,
      updated: Some(naive_now()),
      enable_downvotes: read_site.enable_downvotes,
      open_registration: read_site.open_registration,
      enable_nsfw: read_site.enable_nsfw,
    };

    let update_site = move |conn: &'_ _| Site::update(conn, 1, &site_form);
    if blocking(pool, update_site).await?.is_err() {
      return Err(APIError::err("couldnt_update_site").into());
    };

    // Mod tables
    let form = ModAddForm {
      mod_user_id: user_id,
      other_user_id: data.user_id,
      removed: Some(false),
    };

    blocking(pool, move |conn| ModAdd::create(conn, &form)).await??;

    let site_view = blocking(pool, move |conn| SiteView::read(conn)).await??;

    let mut admins = blocking(pool, move |conn| UserView::admins(conn)).await??;
    let creator_index = admins
      .iter()
      .position(|r| r.id == site_view.creator_id)
      .unwrap();
    let creator_user = admins.remove(creator_index);
    admins.insert(0, creator_user);

    let banned = blocking(pool, move |conn| UserView::banned(conn)).await??;

    Ok(GetSiteResponse {
      site: Some(site_view),
      admins,
      banned,
      online: 0,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetSiteConfig> {
  type Response = GetSiteConfigResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetSiteConfigResponse, LemmyError> {
    let data: &GetSiteConfig = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Only let admins read this
    let admins = blocking(pool, move |conn| UserView::admins(conn)).await??;
    let admin_ids: Vec<i32> = admins.into_iter().map(|m| m.id).collect();

    if !admin_ids.contains(&user_id) {
      return Err(APIError::err("not_an_admin").into());
    }

    let config_hjson = Settings::read_config_file()?;

    Ok(GetSiteConfigResponse { config_hjson })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<SaveSiteConfig> {
  type Response = GetSiteConfigResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetSiteConfigResponse, LemmyError> {
    let data: &SaveSiteConfig = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;

    // Only let admins read this
    let admins = blocking(pool, move |conn| UserView::admins(conn)).await??;
    let admin_ids: Vec<i32> = admins.into_iter().map(|m| m.id).collect();

    if !admin_ids.contains(&user_id) {
      return Err(APIError::err("not_an_admin").into());
    }

    // Make sure docker doesn't have :ro at the end of the volume, so its not a read-only filesystem
    let config_hjson = match Settings::save_config_file(&data.config_hjson) {
      Ok(config_hjson) => config_hjson,
      Err(_e) => return Err(APIError::err("couldnt_update_site").into()),
    };

    Ok(GetSiteConfigResponse { config_hjson })
  }
}
