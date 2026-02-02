use crate::objects::person::ApubPerson;
use activitypub_federation::{fetch::object_id::ObjectId, kinds::link::MentionType};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::community_tag::{CommunityTag, CommunityTagInsertForm},
};
use lemmy_db_schema_file::enums::TagColor;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

/// Possible values in the `tag` field of a federated post or comment. Note that we don't support
/// hashtags or community tags in comments, but its easier to use the same struct for both
/// (anyway unsupported values are ignored).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ApubTag {
  Hashtag(Hashtag),
  CommunityTag(ApubCommunityTag),
  Mention(Mention),
  Unknown(Value),
}

impl ApubTag {
  pub(crate) fn community_tag_id(&self) -> Option<&Url> {
    match self {
      ApubTag::CommunityTag(t) => Some(&t.id),
      _ => None,
    }
  }
  pub fn mention_id(&self) -> Option<&ObjectId<ApubPerson>> {
    match self {
      ApubTag::Mention(m) => Some(&m.href),
      _ => None,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mention {
  pub href: ObjectId<ApubPerson>,
  pub(crate) name: Option<String>,
  #[serde(rename = "type")]
  pub kind: MentionType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hashtag {
  pub(crate) href: Url,
  pub(crate) name: String,
  #[serde(rename = "type")]
  pub(crate) kind: HashtagType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum HashtagType {
  Hashtag,
}

/// The [ActivityStreams vocabulary](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-tag)
/// defines that any object can have a list of tags associated with it.
/// Tags in AS can be of any type, so we define our own types.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
enum CommunityTagType {
  #[default]
  CommunityPostTag,
}

/// A tag that a community owns, that is added to a post.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApubCommunityTag {
  #[serde(rename = "type")]
  kind: CommunityTagType,
  pub id: Url,
  pub name: Option<String>,
  pub preferred_username: String,
  pub content: Option<String>,
  pub color: Option<TagColor>,
}

impl ApubCommunityTag {
  pub fn to_json(tag: CommunityTag) -> Self {
    ApubCommunityTag {
      kind: Default::default(),
      id: tag.ap_id.into(),
      name: tag.display_name,
      preferred_username: tag.name,
      content: tag.summary,
      color: Some(tag.color),
    }
  }

  pub fn to_insert_form(&self, community_id: CommunityId) -> CommunityTagInsertForm {
    CommunityTagInsertForm {
      ap_id: self.id.clone().into(),
      name: self.preferred_username.clone(),
      display_name: self.name.clone(),
      summary: self.content.clone(),
      community_id,
      deleted: Some(false),
      color: self.color,
    }
  }
}
