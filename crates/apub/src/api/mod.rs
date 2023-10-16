use lemmy_db_schema::{newtypes::CommunityId, source::local_site::LocalSite, ListingType};
use lemmy_utils::error::LemmyError;

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
  local_site: &LocalSite,
  community_id: Option<CommunityId>,
) -> Result<ListingType, LemmyError> {
  // On frontpage use listing type from param or admin configured default
  let listing_type = if community_id.is_none() {
    type_.unwrap_or(local_site.default_post_listing_type)
  } else {
    // inside of community show everything
    ListingType::All
  };
  Ok(listing_type)
}
