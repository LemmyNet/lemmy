use lemmy_db_schema::{
  newtypes::CommunityId,
  source::tag::{Tag, TagInsertForm},
};
use serde::{Deserialize, Serialize};
use url::Url;

/// The [ActivityStreams vocabulary](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-tag)
/// defines that any object can have a list of tags associated with it.
/// Tags in AS can be of any type, so we define our own types.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
enum CommunityTagType {
  #[default]
  PostTag,
}

/// A tag that a community owns, that is (currently) added to a post.
/// In the community (group), we attach the list of available tags as the "lemmy:tagsForPosts"
/// property.
///
/// In the post, the tags are added to the standard "tag" property.
///
/// Or in AP terms, this is a tag that is owned by a group, and added to a page that has the group
/// as the audience.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CommunityTag {
  #[serde(rename = "type")]
  kind: CommunityTagType,
  pub id: Url,
  pub name: String,
  pub display_name: Option<String>,
}

impl From<Tag> for CommunityTag {
  fn from(tag: Tag) -> Self {
    CommunityTag {
      kind: Default::default(),
      id: tag.ap_id.into(),
      name: tag.name,
      display_name: tag.display_name,
    }
  }
}

impl CommunityTag {
  pub fn into_insert_form(&self, community_id: CommunityId) -> TagInsertForm {
    TagInsertForm {
      ap_id: self.id.clone().into(),
      name: self.name.clone(),
      display_name: self.display_name.clone(),
      community_id,
      deleted: Some(false),
    }
  }
}
