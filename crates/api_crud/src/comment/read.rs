use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, comment::*, get_local_user_view_from_jwt_opt};
use lemmy_db_queries::{ListingType, SortType};
use lemmy_db_views::comment_view::CommentQueryBuilder;
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;
use std::str::FromStr;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetComments {
  type Response = GetCommentsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommentsResponse, LemmyError> {
    let data: &GetComments = &self;
    let local_user_view = get_local_user_view_from_jwt_opt(&data.auth, context.pool()).await?;
    let person_id = local_user_view.map(|u| u.person.id);

    let type_ = ListingType::from_str(&data.type_)?;
    let sort = SortType::from_str(&data.sort)?;

    let community_id = data.community_id;
    let community_name = data.community_name.to_owned();
    let saved_only = data.saved_only;
    let page = data.page;
    let limit = data.limit;
    let comments = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .listing_type(type_)
        .sort(&sort)
        .saved_only(saved_only)
        .community_id(community_id)
        .community_name(community_name)
        .my_person_id(person_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await?;
    let comments = match comments {
      Ok(comments) => comments,
      Err(_) => return Err(ApiError::err("couldnt_get_comments").into()),
    };

    Ok(GetCommentsResponse { comments })
  }
}
