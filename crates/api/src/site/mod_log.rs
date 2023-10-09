use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetModlog, GetModlogResponse},
  utils::{check_community_mod_action_opt, check_private_instance, is_admin},
};
use lemmy_db_schema::{source::local_site::LocalSite, ModlogActionType};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_moderator::structs::{
  AdminPurgeCommentView,
  AdminPurgeCommunityView,
  AdminPurgePersonView,
  AdminPurgePostView,
  ModAddCommunityView,
  ModAddView,
  ModBanFromCommunityView,
  ModBanView,
  ModFeaturePostView,
  ModHideCommunityView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemoveCommunityView,
  ModRemovePostView,
  ModTransferCommunityView,
  ModlogListParams,
};
use lemmy_utils::error::LemmyError;
use ModlogActionType::*;

#[tracing::instrument(skip(context))]
pub async fn get_mod_log(
  data: Query<GetModlog>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> Result<Json<GetModlogResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let type_ = data.type_.unwrap_or(All);
  let community_id = data.community_id;

  let mut is_mod_of_community = false;
  let mut is_admin_ = false;
  if let Some(local_user_view) = local_user_view {
    is_mod_of_community =
      check_community_mod_action_opt(&local_user_view, community_id, &mut context.pool())
        .await
        .is_ok()
        && community_id.is_some();
    is_admin_ = is_admin(&local_user_view).is_ok();
  }
  let hide_modlog_names = local_site.hide_modlog_mod_names && !is_mod_of_community && !is_admin_;

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
    All | ModRemovePost => ModRemovePostView::list(&mut context.pool(), params).await?,
    _ => Default::default(),
  };

  let locked_posts = match type_ {
    All | ModLockPost => ModLockPostView::list(&mut context.pool(), params).await?,
    _ => Default::default(),
  };

  let featured_posts = match type_ {
    All | ModFeaturePost => ModFeaturePostView::list(&mut context.pool(), params).await?,
    _ => Default::default(),
  };

  let removed_comments = match type_ {
    All | ModRemoveComment => ModRemoveCommentView::list(&mut context.pool(), params).await?,
    _ => Default::default(),
  };

  let banned_from_community = match type_ {
    All | ModBanFromCommunity => ModBanFromCommunityView::list(&mut context.pool(), params).await?,
    _ => Default::default(),
  };

  let added_to_community = match type_ {
    All | ModAddCommunity => ModAddCommunityView::list(&mut context.pool(), params).await?,
    _ => Default::default(),
  };

  let transferred_to_community = match type_ {
    All | ModTransferCommunity => {
      ModTransferCommunityView::list(&mut context.pool(), params).await?
    }
    _ => Default::default(),
  };

  let hidden_communities = match type_ {
    All | ModHideCommunity if other_person_id.is_none() => {
      ModHideCommunityView::list(&mut context.pool(), params).await?
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
    (
      match type_ {
        All | ModBan => ModBanView::list(&mut context.pool(), params).await?,
        _ => Default::default(),
      },
      match type_ {
        All | ModAdd => ModAddView::list(&mut context.pool(), params).await?,
        _ => Default::default(),
      },
      match type_ {
        All | ModRemoveCommunity if other_person_id.is_none() => {
          ModRemoveCommunityView::list(&mut context.pool(), params).await?
        }
        _ => Default::default(),
      },
      match type_ {
        All | AdminPurgePerson if other_person_id.is_none() => {
          AdminPurgePersonView::list(&mut context.pool(), params).await?
        }
        _ => Default::default(),
      },
      match type_ {
        All | AdminPurgeCommunity if other_person_id.is_none() => {
          AdminPurgeCommunityView::list(&mut context.pool(), params).await?
        }
        _ => Default::default(),
      },
      match type_ {
        All | AdminPurgePost if other_person_id.is_none() => {
          AdminPurgePostView::list(&mut context.pool(), params).await?
        }
        _ => Default::default(),
      },
      match type_ {
        All | AdminPurgeComment if other_person_id.is_none() => {
          AdminPurgeCommentView::list(&mut context.pool(), params).await?
        }
        _ => Default::default(),
      },
    )
  } else {
    Default::default()
  };

  // Return the jwt
  Ok(Json(GetModlogResponse {
    removed_posts,
    locked_posts,
    featured_posts,
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
  }))
}
