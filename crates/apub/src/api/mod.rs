use actix_web::web::Data;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{newtypes::CommunityId, source::local_site::LocalSite, ListingType};
use lemmy_utils::{error::LemmyError, ConnectionId};
use std::str::FromStr;

mod list_comments;
mod list_posts;
mod read_community;
mod read_person;
mod resolve_object;
mod search;

#[async_trait::async_trait(?Send)]
pub trait PerformApub {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}

/// Returns default listing type, depending if the query is for frontpage or community.
fn listing_type_with_default(
  type_: Option<ListingType>,
  local_site: &LocalSite,
  community_id: Option<CommunityId>,
) -> Result<ListingType, LemmyError> {
  // On frontpage use listing type from param or admin configured default
  let listing_type = if community_id.is_none() {
    type_.unwrap_or(ListingType::from_str(
      &local_site.default_post_listing_type,
    )?)
  } else {
    // inside of community show everything
    ListingType::All
  };
  Ok(listing_type)
}
