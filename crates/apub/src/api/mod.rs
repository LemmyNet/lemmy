use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{local_site::LocalSite, local_user::LocalUser},
  ListingType,
  SortType,
};

pub mod list_comments;
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

/// Returns a default instance-level sort type, if none is given by the user.
/// Order is type, local user default, then site default.
fn sort_type_with_default(
  type_: Option<SortType>,
  local_user: Option<&LocalUser>,
  local_site: &LocalSite,
) -> SortType {
  type_.unwrap_or(
    local_user
      .map(|u| u.default_sort_type)
      .unwrap_or(local_site.default_sort_type),
  )
}
