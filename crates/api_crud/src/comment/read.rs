use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, comment::*, get_local_user_view_from_jwt_opt};
use lemmy_apub::{build_actor_id_from_shortname, EndpointType};
use lemmy_db_queries::{from_opt_str_to_opt_enum, DeleteableOrRemoveable, ListingType, SortType};
use lemmy_db_views::comment_view::CommentQueryBuilder;
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetComments {
  type Response = GetCommentsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommentsResponse, LemmyError> {
    let data: &GetComments = self;
    let local_user_view = get_local_user_view_from_jwt_opt(&data.auth, context.pool()).await?;

    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let person_id = local_user_view.map(|u| u.person.id);

    let sort: Option<SortType> = from_opt_str_to_opt_enum(&data.sort);
    let listing_type: Option<ListingType> = from_opt_str_to_opt_enum(&data.type_);

    let community_id = data.community_id;
    let community_actor_id = data
      .community_name
      .as_ref()
      .map(|t| build_actor_id_from_shortname(EndpointType::Community, t).ok())
      .unwrap_or(None);
    let saved_only = data.saved_only;
    let page = data.page;
    let limit = data.limit;
    let mut comments = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .listing_type(listing_type)
        .sort(sort)
        .saved_only(saved_only)
        .community_id(community_id)
        .community_actor_id(community_actor_id)
        .my_person_id(person_id)
        .show_bot_accounts(show_bot_accounts)
        .page(page)
        .limit(limit)
        .list()
    })
    .await?
    .map_err(|_| ApiError::err("couldnt_get_comments"))?;

    // Blank out deleted or removed info
    for cv in comments
      .iter_mut()
      .filter(|cv| cv.comment.deleted || cv.comment.removed)
    {
      cv.comment = cv.to_owned().comment.blank_out_deleted_or_removed_info();
    }

    Ok(GetCommentsResponse { comments })
  }
}
