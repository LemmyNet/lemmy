use crate::federation::fetcher::resolve_ap_identifier;
use activitypub_federation::config::Data;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{
  community::ApubCommunity,
  multi_community::ApubMultiCommunity,
  person::ApubPerson,
};
use lemmy_db_schema::{
  newtypes::{CommunityId, MultiCommunityId, NameOrId, PersonId},
  source::{
    community::Community,
    local_site::LocalSite,
    local_user::LocalUser,
    multi_community::MultiCommunity,
    person::Person,
  },
};
use lemmy_db_schema_file::enums::{CommentSortType, ListingType, PostSortType};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

mod fetcher;
pub mod list_comments;
pub mod list_person_content;
pub mod list_posts;
pub mod read_community;
pub mod read_multi_community;
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

/// Returns a default page fetch limit.
/// Order is the given, then local user default, then site default.
fn fetch_limit_with_default(
  limit: Option<i64>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> i64 {
  limit.unwrap_or(
    local_user
      .map(|u| i64::from(u.default_items_per_page))
      .unwrap_or(i64::from(local_site.default_items_per_page)),
  )
}

async fn resolve_person_id(
  name_or_id: &NameOrId<PersonId>,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
) -> LemmyResult<PersonId> {
  Ok(match name_or_id {
    NameOrId::Id(id) => *id,
    NameOrId::Name(name) => {
      resolve_ap_identifier::<ApubPerson, Person>(name, context, local_user_view, true)
        .await?
        .id
    }
  })
}

async fn resolve_community_id(
  name_or_id: &Option<NameOrId<CommunityId>>,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
) -> LemmyResult<Option<CommunityId>> {
  Ok(match name_or_id {
    Some(NameOrId::Id(id)) => Some(*id),
    Some(NameOrId::Name(name)) => Some(
      resolve_ap_identifier::<ApubCommunity, Community>(name, context, local_user_view, true)
        .await?
        .id,
    ),
    None => None,
  })
}

async fn resolve_multi_community_id(
  name_or_id: &NameOrId<MultiCommunityId>,
  context: &Data<LemmyContext>,
  local_user_view: &Option<LocalUserView>,
) -> LemmyResult<MultiCommunityId> {
  Ok(match name_or_id {
    NameOrId::Id(id) => *id,
    NameOrId::Name(name) => {
      resolve_ap_identifier::<ApubMultiCommunity, MultiCommunity>(
        name,
        context,
        local_user_view,
        true,
      )
      .await?
      .id
    }
  })
}
