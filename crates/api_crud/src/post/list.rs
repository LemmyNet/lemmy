use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{GetPosts, GetPostsResponse},
  utils::{
    blocking,
    check_private_instance,
    get_local_user_view_from_jwt_opt,
    listing_type_with_site_default,
  },
};
use lemmy_apub::{fetcher::resolve_actor_identifier, objects::community::ApubCommunity};
use lemmy_db_schema::{source::community::Community, traits::DeleteableOrRemoveable};
use lemmy_db_views::post_view::PostQuery;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPosts {
  type Response = GetPostsResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostsResponse, LemmyError> {
    let data: &GetPosts = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

    let person_id = local_user_view.to_owned().map(|l| l.person.id);

    let show_nsfw = local_user_view.as_ref().map(|t| t.local_user.show_nsfw);
    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let show_read_posts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_read_posts);

    let sort = data.sort;
    let listing_type = listing_type_with_site_default(data.type_, context.pool()).await?;

    let page = data.page;
    let limit = data.limit;
    let community_id = data.community_id;
    let community_actor_id = if let Some(name) = &data.community_name {
      resolve_actor_identifier::<ApubCommunity, Community>(name, context)
        .await
        .ok()
        .map(|c| c.actor_id)
    } else {
      None
    };
    let saved_only = data.saved_only;

    let mut posts = blocking(context.pool(), move |conn| {
      PostQuery::builder()
        .conn(conn)
        .listing_type(Some(listing_type))
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
        .build()
        .list()
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_get_posts"))?;

    // Blank out deleted or removed info for non-logged in users
    if person_id.is_none() {
      for pv in posts
        .iter_mut()
        .filter(|p| p.post.deleted || p.post.removed)
      {
        pv.post = pv.to_owned().post.blank_out_deleted_or_removed_info();
      }

      for pv in posts
        .iter_mut()
        .filter(|p| p.community.deleted || p.community.removed)
      {
        pv.community = pv.to_owned().community.blank_out_deleted_or_removed_info();
      }
    }

    Ok(GetPostsResponse { posts })
  }
}
