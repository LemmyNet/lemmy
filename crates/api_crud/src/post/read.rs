use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_private_instance,
  get_local_user_view_from_jwt_opt,
  mark_post_as_read,
  post::*,
};
use lemmy_db_schema::traits::DeleteableOrRemoveable;
use lemmy_db_views::{comment_view::CommentQueryBuilder, post_view::PostView};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::{messages::GetPostUsersOnline, LemmyContext};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPost {
  type Response = GetPostResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostResponse, LemmyError> {
    let data: &GetPost = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let person_id = local_user_view.map(|u| u.person.id);

    let id = data.id;
    let mut post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, id, person_id)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_post"))?;

    // Mark the post as read
    if let Some(person_id) = person_id {
      mark_post_as_read(person_id, id, context.pool()).await?;
    }

    let id = data.id;
    let mut comments = blocking(context.pool(), move |conn| {
      CommentQueryBuilder::create(conn)
        .my_person_id(person_id)
        .show_bot_accounts(show_bot_accounts)
        .post_id(id)
        .limit(9999)
        .list()
    })
    .await??;

    // Necessary for the sidebar
    let community_id = post_view.community.id;
    let mut community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, person_id)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    // Blank out deleted or removed info for non-logged in users
    if person_id.is_none() {
      if post_view.post.deleted || post_view.post.removed {
        post_view.post = post_view.post.blank_out_deleted_or_removed_info();
      }

      for cv in comments
        .iter_mut()
        .filter(|cv| cv.comment.deleted || cv.comment.removed)
      {
        cv.comment = cv.to_owned().comment.blank_out_deleted_or_removed_info();
      }
      if community_view.community.deleted || community_view.community.removed {
        community_view.community = community_view.community.blank_out_deleted_or_removed_info();
      }
    }

    let moderators = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    let online = context
      .chat_server()
      .send(GetPostUsersOnline { post_id: data.id })
      .await
      .unwrap_or(1);

    // Return the jwt
    Ok(GetPostResponse {
      post_view,
      community_view,
      comments,
      moderators,
      online,
    })
  }
}
