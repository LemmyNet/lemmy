use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetModlog, GetModlogResponse},
  utils::{check_community_mod_of_any_or_admin_action, check_private_instance},
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views::{
  combined::modlog_combined_view::ModlogCombinedQuery,
  structs::{LocalUserView, ModlogCombinedView, SiteView},
};
use lemmy_utils::error::LemmyResult;

pub async fn get_mod_log(
  data: Query<GetModlog>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetModlogResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  check_private_instance(&local_user_view, &local_site)?;

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

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(ModlogCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let modlog = ModlogCombinedQuery {
    type_: data.type_,
    listing_type: data.listing_type,
    community_id: data.community_id,
    mod_person_id,
    other_person_id: data.other_person_id,
    local_user: local_user_view.as_ref().map(|u| &u.local_user),
    post_id: data.post_id,
    comment_id: data.comment_id,
    hide_modlog_names: Some(hide_modlog_names),
    cursor_data,
    page_back: data.page_back,
  }
  .list(&mut context.pool())
  .await?;

  let next_page = modlog.last().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(GetModlogResponse { modlog, next_page }))
}
