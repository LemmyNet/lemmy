use crate::Perform;
use actix_web::web::Data;
use anyhow::Context;
use diesel::NotFound;
use lemmy_api_common::{
  blocking,
  build_federated_instances,
  get_local_user_view_from_jwt,
  get_local_user_view_from_jwt_opt,
  is_admin,
  site::*,
};
use lemmy_apub::{
  build_actor_id_from_shortname,
  fetcher::search::{search_by_apub_id, SearchableObjects},
  EndpointType,
};
use lemmy_db_queries::{
  from_opt_str_to_opt_enum,
  source::site::Site_,
  Crud,
  DbPool,
  DeleteableOrRemoveable,
  ListingType,
  SearchType,
  SortType,
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
    community::Community,
    moderator::*,
    person::Person,
    post::Post,
    site::Site,
  },
  PersonId,
};
use lemmy_db_views::{
  comment_view::{CommentQueryBuilder, CommentView},
  post_view::{PostQueryBuilder, PostView},
  site_view::SiteView,
};
use lemmy_db_views_actor::{
  community_view::{CommunityQueryBuilder, CommunityView},
  person_view::{PersonQueryBuilder, PersonViewSafe},
};
use lemmy_db_views_moderator::{
  admin_purge_comment_view::AdminPurgeCommentView,
  admin_purge_community_view::AdminPurgeCommunityView,
  admin_purge_person_view::AdminPurgePersonView,
  admin_purge_post_view::AdminPurgePostView,
  mod_add_community_view::ModAddCommunityView,
  mod_add_view::ModAddView,
  mod_ban_from_community_view::ModBanFromCommunityView,
  mod_ban_view::ModBanView,
  mod_lock_post_view::ModLockPostView,
  mod_remove_comment_view::ModRemoveCommentView,
  mod_remove_community_view::ModRemoveCommunityView,
  mod_remove_post_view::ModRemovePostView,
  mod_sticky_post_view::ModStickyPostView,
  mod_transfer_community_view::ModTransferCommunityView,
};
use lemmy_utils::{
  location_info,
  request::purge_image_from_pictrs,
  settings::structs::Settings,
  version,
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetModlog {
  type Response = GetModlogResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetModlogResponse, LemmyError> {
    let data: &GetModlog = self;

    let community_id = data.community_id;
    let mod_person_id = data.mod_person_id;
    let page = data.page;
    let limit = data.limit;
    let removed_posts = blocking(context.pool(), move |conn| {
      ModRemovePostView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let locked_posts = blocking(context.pool(), move |conn| {
      ModLockPostView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let stickied_posts = blocking(context.pool(), move |conn| {
      ModStickyPostView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let removed_comments = blocking(context.pool(), move |conn| {
      ModRemoveCommentView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let banned_from_community = blocking(context.pool(), move |conn| {
      ModBanFromCommunityView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let added_to_community = blocking(context.pool(), move |conn| {
      ModAddCommunityView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let transferred_to_community = blocking(context.pool(), move |conn| {
      ModTransferCommunityView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    // These arrays are only for the full modlog, when a community isn't given
    let (
      removed_communities,
      banned,
      added,
      admin_purged_persons,
      admin_purged_communities,
      admin_purged_posts,
      admin_purged_comments,
    ) = if data.community_id.is_none() {
      blocking(context.pool(), move |conn| {
        Ok((
          ModRemoveCommunityView::list(conn, mod_person_id, page, limit)?,
          ModBanView::list(conn, mod_person_id, page, limit)?,
          ModAddView::list(conn, mod_person_id, page, limit)?,
          AdminPurgePersonView::list(conn, mod_person_id, page, limit)?,
          AdminPurgeCommunityView::list(conn, mod_person_id, page, limit)?,
          AdminPurgePostView::list(conn, mod_person_id, page, limit)?,
          AdminPurgeCommentView::list(conn, mod_person_id, page, limit)?,
        )) as Result<_, LemmyError>
      })
      .await??
    } else {
      (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
      )
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
      transferred_to_community,
      admin_purged_persons,
      admin_purged_communities,
      admin_purged_posts,
      admin_purged_comments,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Search {
  type Response = SearchResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<SearchResponse, LemmyError> {
    let data: &Search = self;

    let local_user_view =
      get_local_user_view_from_jwt_opt(&data.auth, context.pool(), context.secret()).await?;

    let show_nsfw = local_user_view.as_ref().map(|t| t.local_user.show_nsfw);
    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let show_read_posts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_read_posts);

    let person_id = local_user_view.map(|u| u.person.id);

    let mut posts = Vec::new();
    let mut comments = Vec::new();
    let mut communities = Vec::new();
    let mut users = Vec::new();

    // TODO no clean / non-nsfw searching rn

    let q = data.q.to_owned();
    let page = data.page;
    let limit = data.limit;
    let sort: Option<SortType> = from_opt_str_to_opt_enum(&data.sort);
    let listing_type: Option<ListingType> = from_opt_str_to_opt_enum(&data.listing_type);
    let search_type: SearchType = from_opt_str_to_opt_enum(&data.type_).unwrap_or(SearchType::All);
    let community_id = data.community_id;
    let community_actor_id = data
      .community_name
      .as_ref()
      .map(|t| build_actor_id_from_shortname(EndpointType::Community, t, &context.settings()).ok())
      .unwrap_or(None);
    let creator_id = data.creator_id;
    match search_type {
      SearchType::Posts => {
        posts = blocking(context.pool(), move |conn| {
          PostQueryBuilder::create(conn)
            .sort(sort)
            .show_nsfw(show_nsfw)
            .show_bot_accounts(show_bot_accounts)
            .show_read_posts(show_read_posts)
            .listing_type(listing_type)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Comments => {
        comments = blocking(context.pool(), move |conn| {
          CommentQueryBuilder::create(conn)
            .sort(sort)
            .listing_type(listing_type)
            .search_term(q)
            .show_bot_accounts(show_bot_accounts)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Communities => {
        communities = blocking(context.pool(), move |conn| {
          CommunityQueryBuilder::create(conn)
            .sort(sort)
            .listing_type(listing_type)
            .search_term(q)
            .my_person_id(person_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::Users => {
        users = blocking(context.pool(), move |conn| {
          PersonQueryBuilder::create(conn)
            .sort(sort)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
      SearchType::All => {
        // If the community or creator is included, dont search communities or users
        let community_or_creator_included =
          data.community_id.is_some() || data.community_name.is_some() || data.creator_id.is_some();
        let community_actor_id_2 = community_actor_id.to_owned();

        posts = blocking(context.pool(), move |conn| {
          PostQueryBuilder::create(conn)
            .sort(sort)
            .show_nsfw(show_nsfw)
            .show_bot_accounts(show_bot_accounts)
            .show_read_posts(show_read_posts)
            .listing_type(listing_type)
            .community_id(community_id)
            .community_actor_id(community_actor_id_2)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .search_term(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;

        let q = data.q.to_owned();
        let community_actor_id = community_actor_id.to_owned();

        comments = blocking(context.pool(), move |conn| {
          CommentQueryBuilder::create(conn)
            .sort(sort)
            .listing_type(listing_type)
            .search_term(q)
            .show_bot_accounts(show_bot_accounts)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .my_person_id(person_id)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;

        let q = data.q.to_owned();

        communities = if community_or_creator_included {
          vec![]
        } else {
          blocking(context.pool(), move |conn| {
            CommunityQueryBuilder::create(conn)
              .sort(sort)
              .listing_type(listing_type)
              .search_term(q)
              .my_person_id(person_id)
              .page(page)
              .limit(limit)
              .list()
          })
          .await??
        };

        let q = data.q.to_owned();

        users = if community_or_creator_included {
          vec![]
        } else {
          blocking(context.pool(), move |conn| {
            PersonQueryBuilder::create(conn)
              .sort(sort)
              .search_term(q)
              .page(page)
              .limit(limit)
              .list()
          })
          .await??
        };
      }
      SearchType::Url => {
        posts = blocking(context.pool(), move |conn| {
          PostQueryBuilder::create(conn)
            .sort(sort)
            .show_nsfw(show_nsfw)
            .show_bot_accounts(show_bot_accounts)
            .show_read_posts(show_read_posts)
            .listing_type(listing_type)
            .my_person_id(person_id)
            .community_id(community_id)
            .community_actor_id(community_actor_id)
            .creator_id(creator_id)
            .url_search(q)
            .page(page)
            .limit(limit)
            .list()
        })
        .await??;
      }
    };

    // Blank out deleted or removed info
    for cv in comments
      .iter_mut()
      .filter(|cv| cv.comment.deleted || cv.comment.removed)
    {
      cv.comment = cv.to_owned().comment.blank_out_deleted_or_removed_info();
    }

    for cv in communities
      .iter_mut()
      .filter(|cv| cv.community.deleted || cv.community.removed)
    {
      cv.community = cv.to_owned().community.blank_out_deleted_or_removed_info();
    }

    for pv in posts
      .iter_mut()
      .filter(|p| p.post.deleted || p.post.removed)
    {
      pv.post = pv.to_owned().post.blank_out_deleted_or_removed_info();
    }

    // Return the jwt
    Ok(SearchResponse {
      type_: search_type.to_string(),
      comments,
      posts,
      communities,
      users,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ResolveObject {
  type Response = ResolveObjectResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ResolveObjectResponse, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt_opt(&self.auth, context.pool(), context.secret()).await?;
    let res = search_by_apub_id(&self.q, context)
      .await
      .map_err(|e| ApiError::err("couldnt_find_object", e))?;
    convert_response(res, local_user_view.map(|l| l.person.id), context.pool())
      .await
      .map_err(|e| ApiError::err("couldnt_find_object", e).into())
  }
}

async fn convert_response(
  object: SearchableObjects,
  user_id: Option<PersonId>,
  pool: &DbPool,
) -> Result<ResolveObjectResponse, LemmyError> {
  let removed_or_deleted;
  let mut res = ResolveObjectResponse {
    comment: None,
    post: None,
    community: None,
    person: None,
  };
  use SearchableObjects::*;
  match object {
    Person(p) => {
      removed_or_deleted = p.deleted;
      res.person = Some(blocking(pool, move |conn| PersonViewSafe::read(conn, p.id)).await??)
    }
    Community(c) => {
      removed_or_deleted = c.deleted || c.removed;
      res.community =
        Some(blocking(pool, move |conn| CommunityView::read(conn, c.id, user_id)).await??)
    }
    Post(p) => {
      removed_or_deleted = p.deleted || p.removed;
      res.post = Some(blocking(pool, move |conn| PostView::read(conn, p.id, user_id)).await??)
    }
    Comment(c) => {
      removed_or_deleted = c.deleted || c.removed;
      res.comment = Some(blocking(pool, move |conn| CommentView::read(conn, c.id, user_id)).await??)
    }
  };
  // if the object was deleted from database, dont return it
  if removed_or_deleted {
    return Err(NotFound {}.into());
  }
  Ok(res)
}

#[async_trait::async_trait(?Send)]
impl Perform for TransferSite {
  type Response = GetSiteResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let data: &TransferSite = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    is_admin(&local_user_view)?;

    let read_site = blocking(context.pool(), move |conn| Site::read_simple(conn)).await??;

    // Make sure user is the creator
    if read_site.creator_id != local_user_view.person.id {
      return Err(ApiError::err_plain("not_an_admin").into());
    }

    let new_creator_id = data.person_id;
    let transfer_site = move |conn: &'_ _| Site::transfer(conn, new_creator_id);
    blocking(context.pool(), transfer_site)
      .await?
      .map_err(|e| ApiError::err("couldnt_update_site", e))?;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      removed: Some(false),
    };

    blocking(context.pool(), move |conn| ModAdd::create(conn, &form)).await??;

    let site_view = blocking(context.pool(), SiteView::read).await??;

    let mut admins = blocking(context.pool(), PersonViewSafe::admins).await??;
    let creator_index = admins
      .iter()
      .position(|r| r.person.id == site_view.creator.id)
      .context(location_info!())?;
    let creator_person = admins.remove(creator_index);
    admins.insert(0, creator_person);

    let banned = blocking(context.pool(), PersonViewSafe::banned).await??;
    let federated_instances = build_federated_instances(
      context.pool(),
      &context.settings().federation,
      &context.settings().hostname,
    )
    .await?;

    Ok(GetSiteResponse {
      site_view: Some(site_view),
      admins,
      banned,
      online: 0,
      version: version::VERSION.to_string(),
      my_user: None,
      federated_instances,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetSiteConfig {
  type Response = GetSiteConfigResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteConfigResponse, LemmyError> {
    let data: &GetSiteConfig = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins read this
    is_admin(&local_user_view)?;

    let config_hjson = Settings::read_config_file()?;

    Ok(GetSiteConfigResponse { config_hjson })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for SaveSiteConfig {
  type Response = GetSiteConfigResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteConfigResponse, LemmyError> {
    let data: &SaveSiteConfig = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins read this
    is_admin(&local_user_view)?;

    // Make sure docker doesn't have :ro at the end of the volume, so its not a read-only filesystem
    let config_hjson = Settings::save_config_file(&data.config_hjson)
      .map_err(|e| ApiError::err("couldnt_update_site", e))?;

    Ok(GetSiteConfigResponse { config_hjson })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for PurgePerson {
  type Response = PurgeItemResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

    let person_id = data.person_id;
    if data.remove_images {
      // Read the person to get their images
      let person = blocking(context.pool(), move |conn| Person::read(conn, person_id)).await??;

      if let Some(banner) = person.banner {
        purge_image_from_pictrs(context.client(), &context.settings(), &banner.into_inner())
          .await?;
      }

      if let Some(avatar) = person.avatar {
        purge_image_from_pictrs(context.client(), &context.settings(), &avatar.into_inner())
          .await?;
      }
    }

    blocking(context.pool(), move |conn| Person::delete(conn, person_id)).await??;

    // Mod tables
    let reason = data.reason.to_owned();
    let form = AdminPurgePersonForm {
      admin_person_id: local_user_view.person.id,
      reason,
    };

    blocking(context.pool(), move |conn| {
      AdminPurgePerson::create(conn, &form)
    })
    .await??;

    Ok(PurgeItemResponse { success: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for PurgeCommunity {
  type Response = PurgeItemResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

    let community_id = data.community_id;
    if data.remove_images {
      // Read the community to get its images
      let community = blocking(context.pool(), move |conn| {
        Community::read(conn, community_id)
      })
      .await??;

      if let Some(banner) = community.banner {
        purge_image_from_pictrs(context.client(), &context.settings(), &banner.into_inner())
          .await?;
      }

      if let Some(icon) = community.icon {
        purge_image_from_pictrs(context.client(), &context.settings(), &icon.into_inner()).await?;
      }
    }
    blocking(context.pool(), move |conn| {
      Community::delete(conn, community_id)
    })
    .await??;

    // Mod tables
    let reason = data.reason.to_owned();
    let form = AdminPurgeCommunityForm {
      admin_person_id: local_user_view.person.id,
      reason,
    };

    blocking(context.pool(), move |conn| {
      AdminPurgeCommunity::create(conn, &form)
    })
    .await??;

    Ok(PurgeItemResponse { success: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for PurgePost {
  type Response = PurgeItemResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

    let post_id = data.post_id;

    // Read the post to get the community_id
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

    let community_id = post.community_id;

    blocking(context.pool(), move |conn| Post::delete(conn, post_id)).await??;

    // Mod tables
    let reason = data.reason.to_owned();
    let form = AdminPurgePostForm {
      admin_person_id: local_user_view.person.id,
      reason,
      community_id,
    };

    blocking(context.pool(), move |conn| {
      AdminPurgePost::create(conn, &form)
    })
    .await??;

    Ok(PurgeItemResponse { success: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for PurgeComment {
  type Response = PurgeItemResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &Self = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins purge an item
    is_admin(&local_user_view)?;

    let comment_id = data.comment_id;

    // Read the comment to get the post_id
    let comment = blocking(context.pool(), move |conn| Comment::read(conn, comment_id)).await??;

    let post_id = comment.post_id;

    blocking(context.pool(), move |conn| {
      Comment::delete(conn, comment_id)
    })
    .await??;

    // Mod tables
    let reason = data.reason.to_owned();
    let form = AdminPurgeCommentForm {
      admin_person_id: local_user_view.person.id,
      reason,
      post_id,
    };

    blocking(context.pool(), move |conn| {
      AdminPurgeComment::create(conn, &form)
    })
    .await??;

    Ok(PurgeItemResponse { success: true })
  }
}
