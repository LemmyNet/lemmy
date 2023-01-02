use crate::{
  activities::verify_community_matches,
  fetcher::user_or_community::{PersonOrGroupType, UserOrCommunity},
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{objects::LanguageTag, ImageObject, InCommunity, Source},
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  deser::{
    helpers::{deserialize_one_or_many, deserialize_skip_error},
    values::MediaTypeMarkdownOrHtml,
  },
  traits::{ActivityHandler, ApubObject},
};
use activitystreams_kinds::{
  link::LinkType,
  object::{DocumentType, ImageType},
};
use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;
use crate::protocol::objects::Emoji;

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
  pub(crate) name: String,

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
  pub(crate) stickied: Option<bool>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
  pub(crate) language: Option<LanguageTag>,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) tag: Vec<Emoji>
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
  /// Only mods can change the post's stickied/locked status. So if either of these is changed from
  /// the current value, it is a mod action and needs to be verified as such.
  ///
  /// Both stickied and locked need to be false on a newly created post (verified in [[CreatePost]].
  pub(crate) async fn is_mod_action(&self, context: &LemmyContext) -> Result<bool, LemmyError> {
    let old_post = ObjectId::<ApubPost>::new(self.id.clone())
      .dereference_local(context)
      .await;

    let featured_changed = Page::is_featured_changed(&old_post, &self.stickied);
    let locked_changed = Page::is_locked_changed(&old_post, &self.comments_enabled);
    Ok(featured_changed || locked_changed)
  }

  pub(crate) fn is_featured_changed<E>(
    old_post: &Result<ApubPost, E>,
    new_featured_community: &Option<bool>,
  ) -> bool {
    if let Some(new_featured_community) = new_featured_community {
      if let Ok(old_post) = old_post {
        return new_featured_community != &old_post.featured_community;
      }
    }

    false
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
        .map(|a| ObjectId::<ApubPerson>::new(a.id.clone().into_inner()))
        .ok_or_else(|| LemmyError::from_message("page does not specify creator person")),
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
#[async_trait::async_trait(?Send)]
impl ActivityHandler for Page {
  type DataType = LemmyContext;
  type Error = LemmyError;
  fn id(&self) -> &Url {
    unimplemented!()
  }
  fn actor(&self) -> &Url {
    unimplemented!()
  }
  async fn verify(
    &self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    ApubPost::verify(self, self.id.inner(), data, request_counter).await
  }
  async fn receive(
    self,
    data: &Data<Self::DataType>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    ApubPost::from_apub(self, data, request_counter).await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl InCommunity for Page {
  async fn community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let instance = local_instance(context).await;
    let community = match &self.attributed_to {
      AttributedTo::Lemmy(_) => {
        let mut iter = self.to.iter().merge(self.cc.iter());
        loop {
          if let Some(cid) = iter.next() {
            let cid = ObjectId::new(cid.clone());
            if let Ok(c) = cid.dereference(context, instance, request_counter).await {
              break c;
            }
          } else {
            return Err(LemmyError::from_message("No community found in cc"));
          }
        }
      }
      AttributedTo::Peertube(p) => {
        p.iter()
          .find(|a| a.kind == PersonOrGroupType::Group)
          .map(|a| ObjectId::<ApubCommunity>::new(a.id.clone().into_inner()))
          .ok_or_else(|| LemmyError::from_message("page does not specify group"))?
          .dereference(context, instance, request_counter)
          .await?
      }
    };
    if let Some(audience) = &self.audience {
      let audience = audience
        .dereference(context, instance, request_counter)
        .await?;
      verify_community_matches(&audience, community.id)?;
      Ok(audience)
    } else {
      Ok(community)
    }
  }
}
