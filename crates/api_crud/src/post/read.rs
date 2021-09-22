use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{blocking, get_local_user_view_from_jwt_opt, mark_post_as_read, post::*};
use lemmy_apub::{build_actor_id_from_shortname, EndpointType};
use lemmy_db_queries::{from_opt_str_to_opt_enum, DeleteableOrRemoveable, ListingType, SortType};
use lemmy_db_views::{
  comment_view::CommentQueryBuilder,
  post_view::{PostQueryBuilder, PostView},
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::GetPostUsersOnline, LemmyContext};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPost {
  type Response = GetPostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostResponse, LemmyError> {
    let data: &GetPost = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(&data.auth, context.pool(), context.secret()).await?;

    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let person_id = local_user_view.map(|u| u.person.id);

    let id = data.id;
    let mut post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, id, person_id)
    })
    .await?
    .map_err(|_| ApiError::err("couldnt_find_post"))?;

    // Blank out deleted info
    if post_view.post.deleted || post_view.post.removed {
      post_view.post = post_view.post.blank_out_deleted_or_removed_info();
    }

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

    // Blank out deleted or removed info
    for cv in comments
      .iter_mut()
      .filter(|cv| cv.comment.deleted || cv.comment.removed)
    {
      cv.comment = cv.to_owned().comment.blank_out_deleted_or_removed_info();
    }

    let community_id = post_view.community.id;
    let moderators = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await??;

    // Necessary for the sidebar
    let mut community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, person_id)
    })
    .await?
    .map_err(|_| ApiError::err("couldnt_find_community"))?;

    // Blank out deleted or removed info
    if community_view.community.deleted || community_view.community.removed {
      community_view.community = community_view.community.blank_out_deleted_or_removed_info();
    }

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

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPosts {
  type Response = GetPostsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostsResponse, LemmyError> {
    let data: &GetPosts = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(&data.auth, context.pool(), context.secret()).await?;

    let person_id = local_user_view.to_owned().map(|l| l.person.id);

    let show_nsfw = local_user_view.as_ref().map(|t| t.local_user.show_nsfw);
    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let show_read_posts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_read_posts);

    let sort: Option<SortType> = from_opt_str_to_opt_enum(&data.sort);
    let listing_type: Option<ListingType> = from_opt_str_to_opt_enum(&data.type_);

    let page = data.page;
    let limit = data.limit;
    let community_id = data.community_id;
    let community_actor_id = data
      .community_name
      .as_ref()
      .map(|t| build_actor_id_from_shortname(EndpointType::Community, t).ok())
      .unwrap_or(None);
    let saved_only = data.saved_only;

    let mut posts = blocking(context.pool(), move |conn| {
      PostQueryBuilder::create(conn)
        .listing_type(listing_type)
        .sort(sort)
        .show_nsfw(show_nsfw)
        .show_bot_accounts(show_bot_accounts)
        .show_read_posts(show_read_posts)
        .community_id(community_id)
        .community_actor_id(community_actor_id)
        .saved_only(saved_only)
        .my_person_id(person_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await?
    .map_err(|_| ApiError::err("couldnt_get_posts"))?;

    // Blank out deleted or removed info
    for pv in posts
      .iter_mut()
      .filter(|p| p.post.deleted || p.post.removed)
    {
      pv.post = pv.to_owned().post.blank_out_deleted_or_removed_info();
    }

    Ok(GetPostsResponse { posts })
  }
}
