use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_private_instance,
  get_local_user_view_from_jwt_opt,
  site::{GetModlog, GetModlogResponse},
};
use lemmy_db_views_moderator::{
  mod_add_community_view::ModAddCommunityView,
  mod_add_view::ModAddView,
  mod_ban_from_community_view::ModBanFromCommunityView,
  mod_ban_view::ModBanView,
  mod_hide_community_view::ModHideCommunityView,
  mod_lock_post_view::ModLockPostView,
  mod_remove_comment_view::ModRemoveCommentView,
  mod_remove_community_view::ModRemoveCommunityView,
  mod_remove_post_view::ModRemovePostView,
  mod_sticky_post_view::ModStickyPostView,
  mod_transfer_community_view::ModTransferCommunityView,
};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetModlog {
  type Response = GetModlogResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetModlogResponse, LemmyError> {
    let data: &GetModlog = self;

    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

    let community_id = data.community_id;
    let mod_person_id = data.mod_person_id;
    let page = data.page;
    let limit = data.limit;
    let removed_posts = blocking(context.pool(), move |conn| {
      ModRemovePostView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let locked_posts = blocking(context.pool(), move |conn| {
      ModLockPostView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let stickied_posts = blocking(context.pool(), move |conn| {
      ModStickyPostView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let removed_comments = blocking(context.pool(), move |conn| {
      ModRemoveCommentView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let banned_from_community = blocking(context.pool(), move |conn| {
      ModBanFromCommunityView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let added_to_community = blocking(context.pool(), move |conn| {
      ModAddCommunityView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let transferred_to_community = blocking(context.pool(), move |conn| {
      ModTransferCommunityView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    let hidden_communities = blocking(context.pool(), move |conn| {
      ModHideCommunityView::list(conn, community_id, mod_person_id, page, limit)
    })
    .await??;

    // These arrays are only for the full modlog, when a community isn't given
    let (removed_communities, banned, added) = if data.community_id.is_none() {
      blocking(context.pool(), move |conn| {
        Ok((
          ModRemoveCommunityView::list(conn, mod_person_id, page, limit)?,
          ModBanView::list(conn, mod_person_id, page, limit)?,
          ModAddView::list(conn, mod_person_id, page, limit)?,
        )) as Result<_, LemmyError>
      })
      .await??
    } else {
      (Vec::new(), Vec::new(), Vec::new())
    };

    // Return the jwt
    Ok(GetModlogResponse {
      removed_posts,
      locked_posts,
      stickied_posts,
      removed_comments,
      removed_communities,
      banned_from_community,
      banned,
      added_to_community,
      added,
      transferred_to_community,
      hidden_communities,
    })
  }
}
