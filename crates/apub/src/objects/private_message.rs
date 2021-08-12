use crate::{
  extensions::context::lemmy_context,
  fetcher::person::get_or_fetch_and_upsert_person,
  objects::{create_tombstone, FromApub, Source, ToApub},
};
use activitystreams::{
  base::AnyBase,
  object::{kind::NoteType, Tombstone},
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  values::{MediaTypeHtml, MediaTypeMarkdown},
  verify_domains_match,
};
use lemmy_db_queries::{ApubObject, Crud, DbPool};
use lemmy_db_schema::source::{
  person::Person,
  private_message::{PrivateMessage, PrivateMessageForm},
};
use lemmy_utils::{utils::convert_datetime, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  r#type: NoteType,
  id: Url,
  pub(crate) attributed_to: Url,
  to: Url,
  content: String,
  media_type: MediaTypeHtml,
  source: Source,
  published: DateTime<FixedOffset>,
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
    verify_domains_match(&self.attributed_to, &self.id)?;
    let person =
      get_or_fetch_and_upsert_person(&self.attributed_to, context, request_counter).await?;
    if person.banned {
      return Err(anyhow!("Person is banned from site").into());
    }
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ToApub for PrivateMessage {
  type ApubType = Note;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| Person::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| Person::read(conn, recipient_id)).await??;

    let note = Note {
      context: lemmy_context(),
      r#type: NoteType::Note,
      id: self.ap_id.clone().into(),
      attributed_to: creator.actor_id.into_inner(),
      to: recipient.actor_id.into(),
      content: self.content.clone(),
      media_type: MediaTypeHtml::Html,
      source: Source {
        content: self.content.clone(),
        media_type: MediaTypeMarkdown::Markdown,
      },
      published: convert_datetime(self.published),
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
impl FromApub for PrivateMessage {
  type ApubType = Note;

  async fn from_apub(
    note: &Note,
    context: &LemmyContext,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<PrivateMessage, LemmyError> {
    let ap_id = Some(note.id(expected_domain)?.clone().into());
    let creator =
      get_or_fetch_and_upsert_person(&note.attributed_to, context, request_counter).await?;
    let recipient = get_or_fetch_and_upsert_person(&note.to, context, request_counter).await?;

    let form = PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content: note.source.content.clone(),
      published: Some(note.published.naive_local()),
      updated: note.updated.map(|u| u.to_owned().naive_local()),
      deleted: None,
      read: None,
      ap_id,
      local: Some(false),
    };
    Ok(
      blocking(context.pool(), move |conn| {
        PrivateMessage::upsert(conn, &form)
      })
      .await??,
    )
  }
}
