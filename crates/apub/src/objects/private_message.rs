use crate::{
  check_is_apub_id_valid,
  extensions::context::lemmy_context,
  fetcher::user::get_or_fetch_and_upsert_user,
  objects::{
    check_object_domain,
    create_tombstone,
    get_object_from_apub,
    get_source_markdown_value,
    set_content_and_source,
    FromApub,
    FromApubToForm,
    ToApub,
  },
  NoteExt,
};
use activitystreams::{
  object::{kind::NoteType, ApObject, Note, Tombstone},
  prelude::*,
};
use anyhow::Context;
use lemmy_db_queries::{Crud, DbPool};
use lemmy_db_schema::source::{
  private_message::{PrivateMessage, PrivateMessageForm},
  user::User_,
};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, utils::convert_datetime, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for PrivateMessage {
  type ApubType = NoteExt;

  async fn to_apub(&self, pool: &DbPool) -> Result<NoteExt, LemmyError> {
    let mut private_message = ApObject::new(Note::new());

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

    private_message
      .set_many_contexts(lemmy_context()?)
      .set_id(Url::parse(&self.ap_id.to_owned())?)
      .set_published(convert_datetime(self.published))
      .set_to(recipient.actor_id)
      .set_attributed_to(creator.actor_id);

    set_content_and_source(&mut private_message, &self.content)?;

    if let Some(u) = self.updated {
      private_message.set_updated(convert_datetime(u));
    }

    Ok(private_message)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(self.deleted, &self.ap_id, self.updated, NoteType::Note)
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for PrivateMessage {
  type ApubType = NoteExt;

  async fn from_apub(
    note: &NoteExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<PrivateMessage, LemmyError> {
    get_object_from_apub(note, context, expected_domain, request_counter).await
  }
}

#[async_trait::async_trait(?Send)]
impl FromApubToForm<NoteExt> for PrivateMessageForm {
  async fn from_apub(
    note: &NoteExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<PrivateMessageForm, LemmyError> {
    let creator_actor_id = note
      .attributed_to()
      .context(location_info!())?
      .clone()
      .single_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(&creator_actor_id, context, request_counter).await?;
    let recipient_actor_id = note
      .to()
      .context(location_info!())?
      .clone()
      .single_xsd_any_uri()
      .context(location_info!())?;
    let recipient =
      get_or_fetch_and_upsert_user(&recipient_actor_id, context, request_counter).await?;
    let ap_id = note.id_unchecked().context(location_info!())?.to_string();
    check_is_apub_id_valid(&Url::parse(&ap_id)?)?;

    let content = get_source_markdown_value(note)?.context(location_info!())?;

    Ok(PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content,
      published: note.published().map(|u| u.to_owned().naive_local()),
      updated: note.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      read: None,
      ap_id: Some(check_object_domain(note, expected_domain)?),
      local: false,
    })
  }
}
