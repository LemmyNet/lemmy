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
use lemmy_db_schema::{
  source::{comment::Comment, community::Community},
  traits::{Crud, DeleteableOrRemoveable},
};
use lemmy_db_views::comment_view::CommentQuery;
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
    let max_depth = data.max_depth;
    let saved_only = data.saved_only;
    let page = data.page;
    let limit = data.limit;
    let parent_id = data.parent_id;

    // If a parent_id is given, fetch the comment to get the path
    let parent_path = if let Some(parent_id) = parent_id {
      Some(
        blocking(context.pool(), move |conn| Comment::read(conn, parent_id))
          .await??
          .path,
      )
    } else {
      None
    };

    let post_id = data.post_id;
    let local_user = local_user_view.map(|l| l.local_user);
    let mut comments = blocking(context.pool(), move |conn| {
      CommentQuery::builder()
        .conn(conn)
        .listing_type(Some(listing_type))
        .sort(sort)
        .max_depth(max_depth)
        .saved_only(saved_only)
        .community_id(community_id)
        .community_actor_id(community_actor_id)
        .parent_path(parent_path)
        .post_id(post_id)
        .local_user(local_user.as_ref())
        .page(page)
        .limit(limit)
        .build()
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
