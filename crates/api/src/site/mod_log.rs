use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{GetModlog, GetModlogResponse},
  utils::{
    blocking,
    check_private_instance,
    get_local_user_view_from_jwt_opt,
    is_admin,
    is_mod_or_admin,
  },
};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  source::site::Site,
  ModlogActionType,
};
use lemmy_db_views_moderator::structs::{
  AdminPurgeCommentView,
  AdminPurgeCommunityView,
  AdminPurgePersonView,
  AdminPurgePostView,
  ModAddCommunityView,
  ModAddView,
  ModBanFromCommunityView,
  ModBanView,
  ModHideCommunityView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemoveCommunityView,
  ModRemovePostView,
  ModStickyPostView,
  ModTransferCommunityView,
  ModlogListParams,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;
use ModlogActionType::*;

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

    let type_ = data.type_.unwrap_or(All);
    let community_id = data.community_id;

    let site = blocking(context.pool(), Site::read_local_site).await??;
    let (local_person_id, is_admin) = match local_user_view {
      Some(s) => (s.person.id, is_admin(&s).is_ok()),
      None => (PersonId(-1), false),
    };
    let community_id_value = match community_id {
      Some(s) => s,
      None => CommunityId(-1),
    };
    let is_mod_of_community = data.community_id.is_some()
      && is_mod_or_admin(context.pool(), local_person_id, community_id_value)
        .await
        .is_ok();
    let hide_modlog_names = site.hide_modlog_mod_names && !is_mod_of_community && !is_admin;

    let mod_person_id = if hide_modlog_names {
      None
    } else {
      data.mod_person_id
    };
    let other_person_id = data.other_person_id;
    let params = ModlogListParams {
      community_id,
      mod_person_id,
      other_person_id,
      page: data.page,
      limit: data.limit,
      hide_modlog_names,
    };
    let removed_posts = match type_ {
      All | ModRemovePost => {
        blocking(context.pool(), move |conn| {
          ModRemovePostView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    let locked_posts = match type_ {
      All | ModLockPost => {
        blocking(context.pool(), move |conn| {
          ModLockPostView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    let stickied_posts = match type_ {
      All | ModStickyPost => {
        blocking(context.pool(), move |conn| {
          ModStickyPostView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    let removed_comments = match type_ {
      All | ModRemoveComment => {
        blocking(context.pool(), move |conn| {
          ModRemoveCommentView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    let banned_from_community = match type_ {
      All | ModBanFromCommunity => {
        blocking(context.pool(), move |conn| {
          ModBanFromCommunityView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    let added_to_community = match type_ {
      All | ModAddCommunity => {
        blocking(context.pool(), move |conn| {
          ModAddCommunityView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    let transferred_to_community = match type_ {
      All | ModTransferCommunity => {
        blocking(context.pool(), move |conn| {
          ModTransferCommunityView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    let hidden_communities = match type_ {
      All | ModHideCommunity if other_person_id.is_none() => {
        blocking(context.pool(), move |conn| {
          ModHideCommunityView::list(conn, params)
        })
        .await??
      }
      _ => Default::default(),
    };

    // These arrays are only for the full modlog, when a community isn't given
    let (
      banned,
      added,
      removed_communities,
      admin_purged_persons,
      admin_purged_communities,
      admin_purged_posts,
      admin_purged_comments,
    ) = if data.community_id.is_none() {
      blocking(context.pool(), move |conn| {
        Ok((
          match type_ {
            All | ModBan => ModBanView::list(conn, params)?,
            _ => Default::default(),
          },
          match type_ {
            All | ModAdd => ModAddView::list(conn, params)?,
            _ => Default::default(),
          },
          match type_ {
            All | ModRemoveCommunity if other_person_id.is_none() => {
              ModRemoveCommunityView::list(conn, params)?
            }
            _ => Default::default(),
          },
          match type_ {
            All | AdminPurgePerson if other_person_id.is_none() => {
              AdminPurgePersonView::list(conn, params)?
            }
            _ => Default::default(),
          },
          match type_ {
            All | AdminPurgeCommunity if other_person_id.is_none() => {
              AdminPurgeCommunityView::list(conn, params)?
            }
            _ => Default::default(),
          },
          match type_ {
            All | AdminPurgePost if other_person_id.is_none() => {
              AdminPurgePostView::list(conn, params)?
            }
            _ => Default::default(),
          },
          match type_ {
            All | AdminPurgeComment if other_person_id.is_none() => {
              AdminPurgeCommentView::list(conn, params)?
            }
            _ => Default::default(),
          },
        )) as Result<_, LemmyError>
      })
      .await??
    } else {
      Default::default()
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
      hidden_communities,
    })
  }
}
