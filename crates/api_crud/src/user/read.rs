use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{GetPersonDetails, GetPersonDetailsResponse},
  utils::{blocking, check_private_instance, get_local_user_view_from_jwt_opt},
};
use lemmy_apub::{fetcher::resolve_actor_identifier, objects::person::ApubPerson};
use lemmy_db_schema::{source::person::Person, utils::post_to_comment_sort_type};
use lemmy_db_views::{comment_view::CommentQuery, post_view::PostQuery};
use lemmy_db_views_actor::structs::{CommunityModeratorView, PersonViewSafe};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetPersonDetails {
  type Response = GetPersonDetailsResponse;

  #[tracing::instrument(skip(self, context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPersonDetailsResponse, LemmyError> {
    let data: &GetPersonDetails = self;

    // Check to make sure a person name or an id is given
    if data.username.is_none() && data.person_id.is_none() {
      return Err(LemmyError::from_message("no_id_given"));
    }

    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

    let show_nsfw = local_user_view.as_ref().map(|t| t.local_user.show_nsfw);
    let show_bot_accounts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_bot_accounts);
    let show_read_posts = local_user_view
      .as_ref()
      .map(|t| t.local_user.show_read_posts);

    let person_details_id = match data.person_id {
      Some(id) => id,
      None => {
        if let Some(username) = &data.username {
          resolve_actor_identifier::<ApubPerson, Person>(username, context)
            .await
            .map_err(|e| e.with_message("couldnt_find_that_username_or_email"))?
            .id
        } else {
          return Err(LemmyError::from_message(
            "couldnt_find_that_username_or_email",
          ));
        }
      }
    };

    let person_id = local_user_view.map(|uv| uv.person.id);

    // You don't need to return settings for the user, since this comes back with GetSite
    // `my_user`
    let person_view = blocking(context.pool(), move |conn| {
      PersonViewSafe::read(conn, person_details_id)
    })
    .await??;

    let sort = data.sort;
    let page = data.page;
    let limit = data.limit;
    let saved_only = data.saved_only;
    let community_id = data.community_id;

    let (posts, comments) = blocking(context.pool(), move |conn| {
      let posts_query = PostQuery::builder()
        .conn(conn)
        .sort(sort)
        .show_nsfw(show_nsfw)
        .show_bot_accounts(show_bot_accounts)
        .show_read_posts(show_read_posts)
        .saved_only(saved_only)
        .community_id(community_id)
        .my_person_id(person_id)
        .page(page)
        .limit(limit);

      let comments_query = CommentQuery::builder()
        .conn(conn)
        .my_person_id(person_id)
        .show_bot_accounts(show_bot_accounts)
        .sort(sort.map(post_to_comment_sort_type))
        .saved_only(saved_only)
        .community_id(community_id)
        .page(page)
        .limit(limit);

      // If its saved only, you don't care what creator it was
      // Or, if its not saved, then you only want it for that specific creator
      let (posts, comments) = if !saved_only.unwrap_or(false) {
        (
          posts_query
            .creator_id(Some(person_details_id))
            .build()
            .list()?,
          comments_query
            .creator_id(Some(person_details_id))
            .build()
            .list()?,
        )
      } else {
        (posts_query.build().list()?, comments_query.build().list()?)
      };

      Ok((posts, comments)) as Result<_, LemmyError>
    })
    .await??;

    let moderates = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_person(conn, person_details_id)
    })
    .await??;

    // Return the jwt
    Ok(GetPersonDetailsResponse {
      person_view,
      moderates,
      comments,
      posts,
    })
  }
}
