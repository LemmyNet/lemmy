use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetModlog, GetModlogResponse},
  utils::{check_private_instance, is_admin, is_mod_or_admin, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  source::local_site::LocalSite,
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
) -> Result<Json<GetModlogResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let type_ = data.type_.unwrap_or(All);
  let community_id = data.community_id;

  let (local_person_id, is_admin) = match local_user_view {
    Some(s) => (s.person.id, is_admin(&s).is_ok()),
    None => (PersonId(-1), false),
  };
  let community_id_value = match community_id {
    Some(s) => s,
    None => CommunityId(-1),
  };
  let is_mod_of_community = data.community_id.is_some()
    && is_mod_or_admin(&mut context.pool(), local_person_id, community_id_value)
      .await
      .is_ok();
  let hide_modlog_names = local_site.hide_modlog_mod_names && !is_mod_of_community && !is_admin;

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
