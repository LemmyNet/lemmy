use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{ListCommentReviews, ListCommentReviewsResponse},
  context::LemmyContext,
  utils::{get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_views::review_comment_view::ReviewCommentQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for ListCommentReviews {
  type Response = ListCommentReviewsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<ListCommentReviewsResponse, LemmyError> {
    let data: &ListCommentReviews = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    is_admin(&local_user_view)?;

    let comment_reviews = ReviewCommentQuery::builder()
      .pool(context.pool())
      .unapproved_only(data.unapproved_only)
      .page(data.page)
      .limit(data.limit)
      .build()
      .list()
      .await?;

    let res = ListCommentReviewsResponse { comment_reviews };

    Ok(res)
  }
}
