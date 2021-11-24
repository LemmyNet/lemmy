use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, comment::*, get_local_user_view_from_jwt_opt};
use lemmy_apub::{
  fetcher::webfinger::webfinger_resolve,
  objects::community::ApubCommunity,
  EndpointType,
};
use lemmy_db_schema::{
  from_opt_str_to_opt_enum,
  traits::DeleteableOrRemoveable,
  ListingType,
  SortType,
};
use lemmy_db_views::comment_view::{CommentQueryBuilder, CommentView};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.map(|u| u.person.id);
    let id = data.id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, id, person_id)
    })
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_find_comment".into()))?;

    Ok(Self::Response {
      comment_view,
      form_id: None,
      recipient_ids: Vec::new(),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetComments {
  type Response = GetCommentsResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommentsResponse, LemmyError> {
    let data: &GetComments = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(&data.auth, context.pool(), context.secret()).await?;

    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let person_id = local_user_view.map(|u| u.person.id);

    let sort: Option<SortType> = from_opt_str_to_opt_enum(&data.sort);
    let listing_type: Option<ListingType> = from_opt_str_to_opt_enum(&data.type_);

    let community_id = data.community_id;
    let community_actor_id = if let Some(name) = &data.community_name {
      webfinger_resolve::<ApubCommunity>(name, EndpointType::Community, context, &mut 0)
        .await
        .ok()
    } else {
      None
    };
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
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_get_comments".into()))?;

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
