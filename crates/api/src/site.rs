use crate::Perform;
use actix_web::web::Data;
use anyhow::Context;
use diesel::NotFound;
use lemmy_api_common::{
  blocking,
  build_federated_instances,
  check_private_instance,
  get_local_user_view_from_jwt,
  get_local_user_view_from_jwt_opt,
  is_admin,
  send_application_approved_email,
  site::*,
};
use lemmy_apub::{
  fetcher::{
    search::{search_by_apub_id, SearchableObjects},
    webfinger::webfinger_resolve,
  },
  objects::community::ApubCommunity,
  EndpointType,
};
use lemmy_db_schema::{
  diesel_option_overwrite,
  from_opt_str_to_opt_enum,
  newtypes::PersonId,
  source::{
    local_user::{LocalUser, LocalUserForm},
    moderator::*,
    registration_application::{RegistrationApplication, RegistrationApplicationForm},
    site::Site,
  },
  traits::{Crud, DeleteableOrRemoveable},
  DbPool,
  ListingType,
  SearchType,
  SortType,
};
use lemmy_db_views::{
  comment_view::{CommentQueryBuilder, CommentView},
  local_user_view::LocalUserView,
  post_view::{PostQueryBuilder, PostView},
  registration_application_view::{
    RegistrationApplicationQueryBuilder,
    RegistrationApplicationView,
  },
  site_view::SiteView,
};
use lemmy_db_views_actor::{
  community_view::{CommunityQueryBuilder, CommunityView},
  person_view::{PersonQueryBuilder, PersonViewSafe},
};
use lemmy_db_views_moderator::{
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
use lemmy_utils::{location_info, settings::structs::Settings, version, ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetModlog {
  type Response = GetModlogResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetModlogResponse, LemmyError> {
    let data: &GetModlog = self;

    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

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
    let (removed_communities, banned, added) = if data.community_id.is_none() {
      blocking(context.pool(), move |conn| {
        Ok((
          ModRemoveCommunityView::list(conn, mod_person_id, page, limit)?,
          ModBanView::list(conn, mod_person_id, page, limit)?,
          ModAddView::list(conn, mod_person_id, page, limit)?,
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
      transferred_to_community,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Search {
  type Response = SearchResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<SearchResponse, LemmyError> {
    let data: &Search = self;

    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

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
    let community_actor_id = if let Some(name) = &data.community_name {
      webfinger_resolve::<ApubCommunity>(name, EndpointType::Community, context, &mut 0)
        .await
        .ok()
    } else {
      None
    };
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

    // Blank out deleted or removed info for non logged in users
    if person_id.is_none() {
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

      for cv in comments
        .iter_mut()
        .filter(|cv| cv.comment.deleted || cv.comment.removed)
      {
        cv.comment = cv.to_owned().comment.blank_out_deleted_or_removed_info();
      }
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

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ResolveObjectResponse, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt_opt(self.auth.as_ref(), context.pool(), context.secret())
        .await?;
    check_private_instance(&local_user_view, context.pool()).await?;

    let res = search_by_apub_id(&self.q, context)
      .await
      .map_err(LemmyError::from)
      .map_err(|e| e.with_message("couldnt_find_object"))?;
    convert_response(res, local_user_view.map(|l| l.person.id), context.pool())
      .await
      .map_err(LemmyError::from)
      .map_err(|e| e.with_message("couldnt_find_object"))
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

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let data: &TransferSite = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    is_admin(&local_user_view)?;

    let read_site = blocking(context.pool(), Site::read_simple).await??;

    // Make sure user is the creator
    if read_site.creator_id != local_user_view.person.id {
      return Err(LemmyError::from_message("not_an_admin"));
    }

    let new_creator_id = data.person_id;
    let transfer_site = move |conn: &'_ _| Site::transfer(conn, new_creator_id);
    blocking(context.pool(), transfer_site)
      .await?
      .map_err(LemmyError::from)
      .map_err(|e| e.with_message("couldnt_update_site"))?;

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

  #[tracing::instrument(skip(context, _websocket_id))]
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

  #[tracing::instrument(skip(context, _websocket_id))]
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
      .map_err(LemmyError::from)
      .map_err(|e| e.with_message("couldnt_update_site"))?;

    Ok(GetSiteConfigResponse { config_hjson })
  }
}

/// Lists registration applications, filterable by undenied only.
#[async_trait::async_trait(?Send)]
impl Perform for ListRegistrationApplications {
  type Response = ListRegistrationApplicationsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let unread_only = data.unread_only;
    let verified_email_only = blocking(context.pool(), Site::read_simple)
      .await??
      .require_email_verification;

    let page = data.page;
    let limit = data.limit;
    let registration_applications = blocking(context.pool(), move |conn| {
      RegistrationApplicationQueryBuilder::create(conn)
        .unread_only(unread_only)
        .verified_email_only(verified_email_only)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    let res = Self::Response {
      registration_applications,
    };

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for ApproveRegistrationApplication {
  type Response = RegistrationApplicationResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let app_id = data.id;

    // Only let admins do this
    is_admin(&local_user_view)?;

    // Update the registration with reason, admin_id
    let deny_reason = diesel_option_overwrite(&data.deny_reason);
    let app_form = RegistrationApplicationForm {
      admin_id: Some(local_user_view.person.id),
      deny_reason,
      ..RegistrationApplicationForm::default()
    };

    let registration_application = blocking(context.pool(), move |conn| {
      RegistrationApplication::update(conn, app_id, &app_form)
    })
    .await??;

    // Update the local_user row
    let local_user_form = LocalUserForm {
      accepted_application: Some(data.approve),
      ..LocalUserForm::default()
    };

    let approved_user_id = registration_application.local_user_id;
    blocking(context.pool(), move |conn| {
      LocalUser::update(conn, approved_user_id, &local_user_form)
    })
    .await??;

    if data.approve {
      let approved_local_user_view = blocking(context.pool(), move |conn| {
        LocalUserView::read(conn, approved_user_id)
      })
      .await??;

      if approved_local_user_view.local_user.email.is_some() {
        send_application_approved_email(&approved_local_user_view, &context.settings())?;
      }
    }

    // Read the view
    let registration_application = blocking(context.pool(), move |conn| {
      RegistrationApplicationView::read(conn, app_id)
    })
    .await??;

    Ok(Self::Response {
      registration_application,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetUnreadRegistrationApplicationCount {
  type Response = GetUnreadRegistrationApplicationCountResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Only let admins do this
    is_admin(&local_user_view)?;

    let verified_email_only = blocking(context.pool(), Site::read_simple)
      .await??
      .require_email_verification;

    let registration_applications = blocking(context.pool(), move |conn| {
      RegistrationApplicationView::get_unread_count(conn, verified_email_only)
    })
    .await??;

    Ok(Self::Response {
      registration_applications,
    })
  }
}
