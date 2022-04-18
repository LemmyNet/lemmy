use lemmy_websocket::LemmyContext;

use crate::objects::community::ApubCommunity;

pub(crate) mod community_moderators;
pub(crate) mod community_outbox;

/// Put community in the data, so we dont have to read it again from the database.
pub(crate) struct CommunityContext(pub ApubCommunity, pub LemmyContext);
