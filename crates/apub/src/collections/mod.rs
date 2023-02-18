use crate::objects::community::ApubCommunity;
use lemmy_api_common::context::LemmyContext;

pub(crate) mod community_featured;
pub(crate) mod community_moderators;
pub(crate) mod community_outbox;

/// Put community in the data, so we dont have to read it again from the database.
pub(crate) struct CommunityContext(pub ApubCommunity, pub LemmyContext);
