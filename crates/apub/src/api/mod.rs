use crate::{fetcher::resolve_ap_identifier, objects::person::ApubPerson};
use activitypub_federation::config::Data;
use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  source::{local_site::LocalSite, local_user::LocalUser, person::Person},
  CommentSortType,
  ListingType,
  PostSortType,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub mod list_comments;
pub mod list_person_content;
pub mod list_posts;
pub mod read_community;
pub mod read_person;
pub mod resolve_object;
pub mod search;
pub mod user_settings_backup;

/// Returns default listing type, depending if the query is for frontpage or community.
fn listing_type_with_default(
  type_: Option<ListingType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
  community_id: Option<CommunityId>,
) -> ListingType {
  // On frontpage use listing type from param or admin configured default
  if community_id.is_none() {
    type_.unwrap_or(
      local_user
        .map(|u| u.default_listing_type)
        .unwrap_or(local_site.default_post_listing_type),
    )
  } else {
    // inside of community show everything
    ListingType::All
  }
}

/// Returns a default instance-level post sort type, if none is given by the user.
/// Order is type, local user default, then site default.
fn post_sort_type_with_default(
  type_: Option<PostSortType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> PostSortType {
  type_.unwrap_or(
    local_user
      .map(|u| u.default_post_sort_type)
      .unwrap_or(local_site.default_post_sort_type),
  )
}

/// Returns a default post_time_range.
/// Order is the given, then local user default, then site default.
/// If zero is given, then the output is None.
fn post_time_range_seconds_with_default(
  secs: Option<i32>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> Option<i32> {
  let out = secs
    .or(local_user.and_then(|u| u.default_post_time_range_seconds))
    .or(local_site.default_post_time_range_seconds);

  // A zero is an override to None
  if out.is_some_and(|o| o == 0) {
    None
  } else {
    out
  }
}

/// Returns a default instance-level comment sort type, if none is given by the user.
/// Order is type, local user default, then site default.
fn comment_sort_type_with_default(
  type_: Option<CommentSortType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> CommentSortType {
  type_.unwrap_or(
    local_user
      .map(|u| u.default_comment_sort_type)
      .unwrap_or(local_site.default_comment_sort_type),
  )
}

async fn resolve_person_id_from_id_or_username(
  person_id: &Option<PersonId>,
  username: &Option<String>,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
) -> LemmyResult<PersonId> {
  // Check to make sure a person name or an id is given
  if username.is_none() && person_id.is_none() {
    Err(LemmyErrorType::NoIdGiven)?
  }

  Ok(match person_id {
    Some(id) => *id,
    None => {
      if let Some(username) = username {
        resolve_ap_identifier::<ApubPerson, Person>(username, context, local_user_view, true)
          .await?
          .id
      } else {
        Err(LemmyErrorType::NotFound)?
      }
    }
  })
}
