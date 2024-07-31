use crate::{
  activities::verify_community_matches,
  fetcher::user_or_community::{PersonOrGroupType, UserOrCommunity},
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{objects::LanguageTag, ImageObject, InCommunity, Source},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::{
    link::LinkType,
    object::{DocumentType, ImageType},
  },
  protocol::{
    helpers::{deserialize_one_or_many, deserialize_skip_error},
    values::MediaTypeMarkdownOrHtml,
  },
  traits::{ActivityHandler, Object},
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PageType {
  Page,
  Article,
  Note,
  Video,
  Event,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
  #[serde(rename = "type")]
  pub(crate) kind: PageType,
  pub(crate) id: ObjectId<ApubPost>,
  pub(crate) attributed_to: AttributedTo,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) to: Vec<Url>,
  // If there is inReplyTo field this is actually a comment and must not be parsed
  #[serde(deserialize_with = "deserialize_not_present", default)]
  pub(crate) in_reply_to: Option<String>,

  pub(crate) name: Option<String>,
  #[serde(deserialize_with = "deserialize_one_or_many", default)]
  pub(crate) cc: Vec<Url>,
  pub(crate) content: Option<String>,
  pub(crate) media_type: Option<MediaTypeMarkdownOrHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  /// most software uses array type for attachment field, so we do the same. nevertheless, we only
  /// use the first item
  #[serde(default)]
  pub(crate) attachment: Vec<Attachment>,
  pub(crate) image: Option<ImageObject>,
  pub(crate) sensitive: Option<bool>,
  pub(crate) published: Option<DateTime<Utc>>,
  pub(crate) updated: Option<DateTime<Utc>>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) tag: Vec<Hashtag>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Link {
  href: Url,
  media_type: Option<String>,
  r#type: LinkType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Image {
  #[serde(rename = "type")]
  kind: ImageType,
  url: Url,
  /// Used for alt_text
  name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Document {
  #[serde(rename = "type")]
  kind: DocumentType,
  url: Url,
  /// Used for alt_text
  name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum Attachment {
  Link(Link),
  Image(Image),
  Document(Document),
}

impl Attachment {
  pub(crate) fn url(self) -> Url {
    match self {
      // url as sent by Lemmy (new)
      Attachment::Link(l) => l.href,
      // image sent by lotide
      Attachment::Image(i) => i.url,
      // sent by mobilizon
      Attachment::Document(d) => d.url,
    }
  }

  pub(crate) fn alt_text(self) -> Option<String> {
    match self {
      Attachment::Image(i) => i.name,
      Attachment::Document(d) => d.name,
      _ => None,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum AttributedTo {
  Lemmy(ObjectId<ApubPerson>),
  Peertube([AttributedToPeertube; 2]),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttributedToPeertube {
  #[serde(rename = "type")]
  pub kind: PersonOrGroupType,
  pub id: ObjectId<UserOrCommunity>,
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

impl Page {
  pub(crate) fn creator(&self) -> LemmyResult<ObjectId<ApubPerson>> {
    match &self.attributed_to {
      AttributedTo::Lemmy(l) => Ok(l.clone()),
      AttributedTo::Peertube(p) => p
        .iter()
        .find(|a| a.kind == PersonOrGroupType::Person)
        .map(|a| ObjectId::<ApubPerson>::from(a.id.clone().into_inner()))
        .ok_or_else(|| LemmyErrorType::PageDoesNotSpecifyCreator.into()),
    }
  }
}

impl Attachment {
  /// Creates new attachment for a given link and mime type.
  pub(crate) fn new(url: Url, media_type: Option<String>, alt_text: Option<String>) -> Attachment {
    let is_image = media_type.clone().unwrap_or_default().starts_with("image");
    if is_image {
      Attachment::Image(Image {
        kind: Default::default(),
        url,
        name: alt_text,
      })
    } else {
      Attachment::Link(Link {
        href: url,
        media_type,
        r#type: Default::default(),
      })
    }
  }
}

// Used for community outbox, so that it can be compatible with Pleroma/Mastodon.
#[async_trait::async_trait]
impl ActivityHandler for Page {
  type DataType = LemmyContext;
  type Error = LemmyError;
  fn id(&self) -> &Url {
    self.id.inner()
  }

  fn actor(&self) -> &Url {
    debug_assert!(false);
    self.id.inner()
  }
  async fn verify(&self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    ApubPost::verify(self, self.id.inner(), data).await
  }
  async fn receive(self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    ApubPost::from_json(self, data).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl InCommunity for Page {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    if let Some(audience) = &self.audience {
      return audience.dereference(context).await;
    }

    let community = match &self.attributed_to {
      AttributedTo::Lemmy(_) => {
        let mut iter = self.to.iter().merge(self.cc.iter());
        loop {
          if let Some(cid) = iter.next() {
            let cid = ObjectId::from(cid.clone());
            if let Ok(c) = cid.dereference(context).await {
              break c;
            }
          } else {
            Err(LemmyErrorType::CouldntFindCommunity)?;
          }
        }
      }
      AttributedTo::Peertube(p) => {
        p.iter()
          .find(|a| a.kind == PersonOrGroupType::Group)
          .map(|a| ObjectId::<ApubCommunity>::from(a.id.clone().into_inner()))
          .ok_or(LemmyErrorType::CouldntFindCommunity)?
          .dereference(context)
          .await?
      }
    };

    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community)
  }
}

/// Only allows deserialization if the field is missing or null. If it is present, throws an error.
pub fn deserialize_not_present<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  let result: Option<String> = Deserialize::deserialize(deserializer)?;
  match result {
    None => Ok(None),
    Some(_) => Err(D::Error::custom("Post must not have inReplyTo property")),
  }
}

#[cfg(test)]
mod tests {
  use crate::protocol::{objects::page::Page, tests::test_parse_lemmy_item};

  #[test]
  fn test_not_parsing_note_as_page() {
    assert!(test_parse_lemmy_item::<Page>("assets/lemmy/objects/note.json").is_err());
  }
}
