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
use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::error::{LemmyError, LemmyErrorType};
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
  #[serde(deserialize_with = "deserialize_one_or_many")]
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
  pub(crate) comments_enabled: Option<bool>,
  pub(crate) sensitive: Option<bool>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Link {
  pub(crate) href: Url,
  pub(crate) r#type: LinkType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Image {
  #[serde(rename = "type")]
  pub(crate) kind: ImageType,
  pub(crate) url: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Document {
  #[serde(rename = "type")]
  pub(crate) kind: DocumentType,
  pub(crate) url: Url,
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

impl Page {
  /// Only mods can change the post's locked status. So if it is changed from the default value,
  /// it is a mod action and needs to be verified as such.
  ///
  /// Locked needs to be false on a newly created post (verified in [[CreatePost]].
  pub(crate) async fn is_mod_action(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<bool, LemmyError> {
    let old_post = self.id.clone().dereference_local(context).await;
    Ok(Page::is_locked_changed(&old_post, &self.comments_enabled))
  }

  pub(crate) fn is_locked_changed<E>(
    old_post: &Result<ApubPost, E>,
    new_comments_enabled: &Option<bool>,
  ) -> bool {
    if let Some(new_comments_enabled) = new_comments_enabled {
      if let Ok(old_post) = old_post {
        return new_comments_enabled != &!old_post.locked;
      }
    }

    false
  }

  pub(crate) fn creator(&self) -> Result<ObjectId<ApubPerson>, LemmyError> {
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
  pub(crate) fn new(url: DbUrl) -> Attachment {
    Attachment::Link(Link {
      href: url.into(),
      r#type: Default::default(),
    })
  }
}

// Used for community outbox, so that it can be compatible with Pleroma/Mastodon.
#[async_trait::async_trait]
impl ActivityHandler for Page {
  type DataType = LemmyContext;
  type Error = LemmyError;
  fn id(&self) -> &Url {
    unimplemented!()
  }
  fn actor(&self) -> &Url {
    unimplemented!()
  }
  async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), LemmyError> {
    ApubPost::verify(self, self.id.inner(), data).await
  }
  async fn receive(self, data: &Data<Self::DataType>) -> Result<(), LemmyError> {
    ApubPost::from_json(self, data).await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl InCommunity for Page {
  async fn community(&self, context: &Data<LemmyContext>) -> Result<ApubCommunity, LemmyError> {
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
            return Err(LemmyErrorType::NoCommunityFoundInCc)?;
          }
        }
      }
      AttributedTo::Peertube(p) => {
        p.iter()
          .find(|a| a.kind == PersonOrGroupType::Group)
          .map(|a| ObjectId::<ApubCommunity>::from(a.id.clone().into_inner()))
          .ok_or(LemmyErrorType::PageDoesNotSpecifyGroup)?
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
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::protocol::{objects::page::Page, tests::test_parse_lemmy_item};

  #[test]
  fn test_not_parsing_note_as_page() {
    assert!(test_parse_lemmy_item::<Page>("assets/lemmy/objects/note.json").is_err());
  }
}
