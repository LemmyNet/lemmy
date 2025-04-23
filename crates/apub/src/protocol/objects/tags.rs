use lemmy_db_schema::source::tag::Tag;
use serde::{Deserialize, Serialize};
use url::Url;

/// The [ActivityStreams vocabulary](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-tag)
/// defines that any object can have a list of tags associated with it.
/// Tags in AS can be of any type, so we define our own types. For now, only `CommunityPostTag`:
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
enum LemmyCommunityPostTagType {
  #[serde(rename = "lemmy:CommunityPostTag")]
  LemmyCommunityPostTagType,
}

/// A tag that a community owns, that is (currently) added to a post.
/// In the community (group), we attach the list of available tags as the "lemmy:postTags" property.
///
/// In the post, the tags are added to the standard "tag" property.
///
/// Or in AP terms, this is a tag that is owned by a group, and added to a page that has the group
/// as the audience.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LemmyCommunityTag {
  #[serde(rename = "type")]
  kind: LemmyCommunityPostTagType,
  pub id: Url,
  // the name of the tag can be updated by the moderators of the community. The ID is fixed.
  pub display_name: String,
}

impl From<Tag> for LemmyCommunityTag {
  fn from(tag: Tag) -> Self {
    LemmyCommunityTag {
      kind: LemmyCommunityPostTagType::LemmyCommunityPostTagType,
      id: tag.ap_id.into(),
      display_name: tag.display_name,
    }
  }
}
