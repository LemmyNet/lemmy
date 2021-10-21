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
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  traits::{ApubObject, FromApub, ToApub},
  values::{MediaTypeHtml, MediaTypeMarkdown},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{
    person::Person,
    private_message::{PrivateMessage, PrivateMessageForm},
  },
  traits::Crud,
  DbPool,
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
  r#type: NoteType,
  id: Url,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  to: ObjectId<ApubPerson>,
  content: String,
  media_type: MediaTypeHtml,
  source: Source,
  published: Option<DateTime<FixedOffset>>,
  updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  unparsed: Unparsed,
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
}

#[async_trait::async_trait(?Send)]
impl ToApub for ApubPrivateMessage {
  type ApubType = Note;
  type TombstoneType = Tombstone;
  type DataType = DbPool;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| Person::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| Person::read(conn, recipient_id)).await??;

    let note = Note {
      context: lemmy_context(),
      r#type: NoteType::Note,
      id: self.ap_id.clone().into(),
      attributed_to: ObjectId::new(creator.actor_id),
      to: ObjectId::new(recipient.actor_id),
      content: self.content.clone(),
      media_type: MediaTypeHtml::Html,
      source: Source {
        content: self.content.clone(),
        media_type: MediaTypeMarkdown::Markdown,
      },
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
}

#[async_trait::async_trait(?Send)]
impl FromApub for ApubPrivateMessage {
  type ApubType = Note;
  type DataType = LemmyContext;

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
    let recipient = note.to.dereference(context, request_counter).await?;

    let form = PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content: note.source.content.clone(),
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

  #[actix_rt::test]
  #[serial]
  async fn test_fetch_lemmy_pm() {
    let context = init_context();
    let url = Url::parse("https://lemmy.ml/private_message/1621").unwrap();
    let lemmy_person = file_to_json_object("assets/lemmy-person.json");
    let person1 = ApubPerson::from_apub(&lemmy_person, &context, &url, &mut 0)
      .await
      .unwrap();
    let pleroma_person = file_to_json_object("assets/pleroma-person.json");
    let pleroma_url = Url::parse("https://queer.hacktivis.me/users/lanodan").unwrap();
    let person2 = ApubPerson::from_apub(&pleroma_person, &context, &pleroma_url, &mut 0)
      .await
      .unwrap();
    let json = file_to_json_object("assets/lemmy-private-message.json");
    let mut request_counter = 0;
    let pm = ApubPrivateMessage::from_apub(&json, &context, &url, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(pm.ap_id.clone().into_inner(), url);
    assert_eq!(pm.content.len(), 4);
    assert_eq!(request_counter, 0);

    let to_apub = pm.to_apub(context.pool()).await.unwrap();
    assert_json_include!(actual: json, expected: to_apub);

    PrivateMessage::delete(&*context.pool().get().unwrap(), pm.id).unwrap();
    Person::delete(&*context.pool().get().unwrap(), person1.id).unwrap();
    Person::delete(&*context.pool().get().unwrap(), person2.id).unwrap();
  }
}
