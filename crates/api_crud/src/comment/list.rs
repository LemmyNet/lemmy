use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{GetComments, GetCommentsResponse},
  utils::{
    blocking,
    check_private_instance,
    get_local_user_view_from_jwt_opt,
    listing_type_with_site_default,
  },
};
use lemmy_apub::{fetcher::resolve_actor_identifier, objects::community::ApubCommunity};
use lemmy_db_schema::{source::community::Community, traits::DeleteableOrRemoveable};
use lemmy_db_views::comment_view::CommentQueryBuilder;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetComments {
  type Response = GetCommentsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommentsResponse, LemmyError> {
    let data: &GetComments = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let person_id = local_user_view.map(|u| u.person.id);

    let community_id = data.community_id;
    let listing_type = listing_type_with_site_default(data.type_, context.pool()).await?;

    let community_actor_id = if let Some(name) = &data.community_name {
      resolve_actor_identifier::<ApubCommunity, Community>(name, context)
        .await
        .ok()
        .map(|c| c.actor_id)
    } else {
      None
    };
    let sort = data.sort;
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
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_get_comments"))?;

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
