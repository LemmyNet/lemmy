use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetModlog, GetModlogResponse},
  utils::{check_community_mod_of_any_or_admin_action, check_private_instance},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_views::{combined::modlog_combined_view::ModlogCombinedQuery, structs::LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn get_mod_log(
  data: Query<GetModlog>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetModlogResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let type_ = data.type_;
  let listing_type = data.listing_type;
  let community_id = data.community_id;

  let is_mod_or_admin = if let Some(local_user_view) = &local_user_view {
    check_community_mod_of_any_or_admin_action(local_user_view, &mut context.pool())
      .await
      .is_ok()
  } else {
    false
  };
  let hide_modlog_names = local_site.hide_modlog_mod_names && !is_mod_or_admin;

  let mod_person_id = if hide_modlog_names {
    None
  } else {
    data.mod_person_id
  };
  let other_person_id = data.other_person_id;
  let post_id = data.post_id;
  let comment_id = data.comment_id;
  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  // parse pagination token
  let page_after = if let Some(pa) = &data.page_cursor {
    Some(pa.read(&mut context.pool()).await?)
  } else {
    None
  };
  let page_back = data.page_back;

  let modlog = ModlogCombinedQuery {
    type_,
    listing_type,
    community_id,
    mod_person_id,
    other_person_id,
    local_user,
    post_id,
    comment_id,
    hide_modlog_names: Some(hide_modlog_names),
    page_after,
    page_back,
  }
  .list(&mut context.pool())
  .await?;

  Ok(Json(GetModlogResponse { modlog }))
}
