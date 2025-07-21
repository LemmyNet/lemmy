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

/// A tag that a community owns, that is added to a post.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CommunityTag {
  #[serde(rename = "type")]
  kind: CommunityTagType,
  pub id: Url,
  pub name: String,
}

impl CommunityTag {
  pub fn to_json(tag: Tag) -> Self {
    CommunityTag {
      kind: Default::default(),
      id: tag.ap_id.into(),
      name: tag.name,
    }
  }

  pub fn into_insert_form(&self, community_id: CommunityId) -> TagInsertForm {
    TagInsertForm {
      ap_id: self.id.clone().into(),
      name: self.name.clone(),
      community_id,
      deleted: Some(false),
    }
  }
}
