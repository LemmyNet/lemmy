use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, GetComment},
  context::LemmyContext,
  sensitive::Sensitive,
  utils::{check_private_instance, local_user_view_from_jwt_opt_new},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_db_views::structs::CommentView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    auth: Option<Sensitive<String>>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view = local_user_view_from_jwt_opt_new(auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    let person_id = local_user_view.map(|u| u.person.id);
    let id = data.id;
    let comment_view = CommentView::read(context.pool(), id, person_id)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_comment"))?;

    Ok(Self::Response {
      comment_view,
      form_id: None,
      recipient_ids: Vec::new(),
    })
  }
}
