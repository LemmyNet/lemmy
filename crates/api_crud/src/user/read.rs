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
    let local_user = local_user_view.map(|l| l.local_user);
    let local_user_clone = local_user.to_owned();

    let posts = blocking(context.pool(), move |conn| {
      let posts_query = PostQuery::builder()
        .conn(conn)
        .sort(sort)
        .saved_only(saved_only)
        .local_user(local_user.as_ref())
        .community_id(community_id)
        .page(page)
        .limit(limit);

      // If its saved only, you don't care what creator it was
      // Or, if its not saved, then you only want it for that specific creator
      if !saved_only.unwrap_or(false) {
        posts_query
          .creator_id(Some(person_details_id))
          .build()
          .list()
      } else {
        posts_query.build().list()
      }
    })
    .await??;

    let comments = blocking(context.pool(), move |conn| {
      let comments_query = CommentQuery::builder()
        .conn(conn)
        .local_user(local_user_clone.as_ref())
        .sort(sort.map(post_to_comment_sort_type))
        .saved_only(saved_only)
        .show_deleted_and_removed(Some(false))
        .community_id(community_id)
        .page(page)
        .limit(limit);

      // If its saved only, you don't care what creator it was
      // Or, if its not saved, then you only want it for that specific creator
      if !saved_only.unwrap_or(false) {
        comments_query
          .creator_id(Some(person_details_id))
          .build()
          .list()
      } else {
        comments_query.build().list()
      }
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
