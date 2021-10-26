use crate::{
  context::lemmy_context,
  fetcher::object_id::ObjectId,
  objects::{create_tombstone, person::ApubPerson, Source},
};
use activitystreams::{
  base::AnyBase,
  chrono::NaiveDateTime,
  object::{kind::NoteType, Tombstone},
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use html2md::parse_html;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  traits::ApubObject,
  values::{MediaTypeHtml, MediaTypeMarkdown},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{
    person::Person,
    private_message::{PrivateMessage, PrivateMessageForm},
  },
  traits::Crud,
};
use lemmy_utils::{utils::convert_datetime, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::ops::Deref;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  r#type: ChatMessageType,
  id: Url,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  to: [ObjectId<ApubPerson>; 1],
  content: String,
  media_type: Option<MediaTypeHtml>,
  source: Option<Source>,
  published: Option<DateTime<FixedOffset>>,
  updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

/// https://docs.pleroma.social/backend/development/ap_extensions/#chatmessages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChatMessageType {
  ChatMessage,
}

impl Note {
  pub(crate) fn id_unchecked(&self) -> &Url {
    &self.id
  }
  pub(crate) fn id(&self, expected_domain: &Url) -> Result<&Url, LemmyError> {
    verify_domains_match(&self.id, expected_domain)?;
    Ok(&self.id)
  }

  pub(crate) async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(self.attributed_to.inner(), &self.id)?;
    let person = self
      .attributed_to
      .dereference(context, request_counter)
      .await?;
    if person.banned {
      return Err(anyhow!("Person is banned from site").into());
    }
    Ok(())
  }
}

#[derive(Clone, Debug)]
pub struct ApubPrivateMessage(PrivateMessage);

impl Deref for ApubPrivateMessage {
  type Target = PrivateMessage;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<PrivateMessage> for ApubPrivateMessage {
  fn from(pm: PrivateMessage) -> Self {
    ApubPrivateMessage { 0: pm }
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubPrivateMessage {
  type DataType = LemmyContext;
  type ApubType = Note;
  type TombstoneType = Tombstone;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  async fn read_from_apub_id(
    object_id: Url,
    context: &LemmyContext,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      blocking(context.pool(), move |conn| {
        PrivateMessage::read_from_apub_id(conn, object_id)
      })
      .await??
      .map(Into::into),
    )
  }

  async fn delete(self, _context: &LemmyContext) -> Result<(), LemmyError> {
    // do nothing, because pm can't be fetched over http
    unimplemented!()
  }

  async fn to_apub(&self, context: &LemmyContext) -> Result<Note, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(context.pool(), move |conn| Person::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let note = Note {
      context: lemmy_context(),
      r#type: ChatMessageType::ChatMessage,
      id: self.ap_id.clone().into(),
      attributed_to: ObjectId::new(creator.actor_id),
      to: [ObjectId::new(recipient.actor_id)],
      content: self.content.clone(),
      media_type: Some(MediaTypeHtml::Html),
      source: Some(Source {
        content: self.content.clone(),
        media_type: MediaTypeMarkdown::Markdown,
      }),
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
      unparsed: Default::default(),
    };
    Ok(note)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(
      self.deleted,
      self.ap_id.to_owned().into(),
      self.updated,
      NoteType::Note,
    )
  }

  async fn from_apub(
    note: &Note,
    context: &LemmyContext,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<ApubPrivateMessage, LemmyError> {
    let ap_id = Some(note.id(expected_domain)?.clone().into());
    let creator = note
      .attributed_to
      .dereference(context, request_counter)
      .await?;
    let recipient = note.to[0].dereference(context, request_counter).await?;
    let content = if let Some(source) = &note.source {
      source.content.clone()
    } else {
      parse_html(&note.content)
    };

    let form = PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content,
      published: note.published.map(|u| u.to_owned().naive_local()),
      updated: note.updated.map(|u| u.to_owned().naive_local()),
      deleted: None,
      read: None,
      ap_id,
      local: Some(false),
    };
    let pm = blocking(context.pool(), move |conn| {
      PrivateMessage::upsert(conn, &form)
    })
    .await??;
    Ok(pm.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::objects::tests::{file_to_json_object, init_context};
  use assert_json_diff::assert_json_include;
  use serial_test::serial;

  async fn prepare_comment_test(url: &Url, context: &LemmyContext) -> (ApubPerson, ApubPerson) {
    let lemmy_person = file_to_json_object("assets/lemmy-person.json");
    let person1 = ApubPerson::from_apub(&lemmy_person, context, url, &mut 0)
      .await
      .unwrap();
    let pleroma_person = file_to_json_object("assets/pleroma-person.json");
    let pleroma_url = Url::parse("https://queer.hacktivis.me/users/lanodan").unwrap();
    let person2 = ApubPerson::from_apub(&pleroma_person, context, &pleroma_url, &mut 0)
      .await
      .unwrap();
    (person1, person2)
  }

  fn cleanup(data: (ApubPerson, ApubPerson), context: &LemmyContext) {
    Person::delete(&*context.pool().get().unwrap(), data.0.id).unwrap();
    Person::delete(&*context.pool().get().unwrap(), data.1.id).unwrap();
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_pm() {
    let context = init_context();
    let url = Url::parse("https://enterprise.lemmy.ml/private_message/1621").unwrap();
    let data = prepare_comment_test(&url, &context).await;
    let json = file_to_json_object("assets/lemmy-private-message.json");
    let mut request_counter = 0;
    let pm = ApubPrivateMessage::from_apub(&json, &context, &url, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(pm.ap_id.clone().into_inner(), url);
    assert_eq!(pm.content.len(), 20);
    assert_eq!(request_counter, 0);

    let to_apub = pm.to_apub(&context).await.unwrap();
    assert_json_include!(actual: json, expected: to_apub);

    PrivateMessage::delete(&*context.pool().get().unwrap(), pm.id).unwrap();
    cleanup(data, &context);
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_pleroma_pm() {
    let context = init_context();
    let url = Url::parse("https://enterprise.lemmy.ml/private_message/1621").unwrap();
    let data = prepare_comment_test(&url, &context).await;
    let pleroma_url = Url::parse("https://queer.hacktivis.me/objects/2").unwrap();
    let json = file_to_json_object("assets/pleroma-private-message.json");
    let mut request_counter = 0;
    let pm = ApubPrivateMessage::from_apub(&json, &context, &pleroma_url, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(pm.ap_id.clone().into_inner(), pleroma_url);
    assert_eq!(pm.content.len(), 3);
    assert_eq!(request_counter, 0);

    PrivateMessage::delete(&*context.pool().get().unwrap(), pm.id).unwrap();
    cleanup(data, &context);
  }
}
