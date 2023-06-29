use crate::{api::PerformApub, fetcher::resolve_actor_identifier, objects::person::ApubPerson};
use activitypub_federation::config::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{GetPersonDetails, GetPersonDetailsResponse},
  utils::{check_private_instance, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{
  source::{local_site::LocalSite, person::Person},
  utils::post_to_comment_sort_type,
};
use lemmy_db_views::{comment_view::CommentQuery, post_view::PostQuery};
use lemmy_db_views_actor::structs::{CommunityModeratorView, PersonView};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait]
impl PerformApub for GetPersonDetails {
  type Response = GetPersonDetailsResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<GetPersonDetailsResponse, LemmyError> {
    let data: &GetPersonDetails = self;

    // Check to make sure a person name or an id is given
    if data.username.is_none() && data.person_id.is_none() {
      return Err(LemmyError::from_message("no_id_given"));
    }

    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), context).await;
    let local_site = LocalSite::read(context.pool()).await?;

    check_private_instance(&local_user_view, &local_site)?;

    let person_details_id = match data.person_id {
      Some(id) => id,
      None => {
        if let Some(username) = &data.username {
          resolve_actor_identifier::<ApubPerson, Person>(username, context, &local_user_view, true)
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
    let person_view = PersonView::read(context.pool(), person_details_id).await?;

    let sort = data.sort;
    let page = data.page;
    let limit = data.limit;
    let saved_only = data.saved_only;
    let community_id = data.community_id;

    let posts_query = PostQuery::builder()
      .pool(context.pool())
      .sort(sort)
      .saved_only(saved_only)
      .local_user(local_user_view.as_ref())
      .community_id(community_id)
      .is_profile_view(Some(true))
      .page(page)
      .limit(limit);

    // If its saved only, you don't care what creator it was
    // Or, if its not saved, then you only want it for that specific creator
    let posts = if !saved_only.unwrap_or(false) {
      posts_query
        .creator_id(Some(person_details_id))
        .build()
        .list()
    } else {
      posts_query.build().list()
    }
    .await?;

    let comments_query = CommentQuery::builder()
      .pool(context.pool())
      .local_user(local_user_view.as_ref())
      .sort(sort.map(post_to_comment_sort_type))
      .saved_only(saved_only)
      .is_profile_view(Some(true))
      .community_id(community_id)
      .page(page)
      .limit(limit);

    // If its saved only, you don't care what creator it was
    // Or, if its not saved, then you only want it for that specific creator
    let comments = if !saved_only.unwrap_or(false) {
      comments_query
        .creator_id(Some(person_details_id))
        .build()
        .list()
    } else {
      comments_query.build().list()
    }
    .await?;

    let moderates = CommunityModeratorView::for_person(context.pool(), person_details_id).await?;

    // Return the jwt
    Ok(GetPersonDetailsResponse {
      person_view,
      moderates,
      comments,
      posts,
    })
  }
}
