use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_comment_response,
  comment::{CommentResponse, GetComment},
  context::LemmyContext,
  utils::{check_private_instance, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), context).await;
    let local_site = LocalSite::read(&mut context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    build_comment_response(context, data.id, local_user_view, None, vec![]).await
  }
}
