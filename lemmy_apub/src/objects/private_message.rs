use crate::{
  check_is_apub_id_valid,
  fetcher::get_or_fetch_and_upsert_user,
  objects::{check_object_domain, create_tombstone},
  FromApub,
  ToApub,
};
use activitystreams::{
  object::{kind::NoteType, Note, Tombstone},
  prelude::*,
};
use anyhow::Context;
use lemmy_db::{
  private_message::{PrivateMessage, PrivateMessageForm},
  user::User_,
  Crud,
  DbPool,
};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, utils::convert_datetime, LemmyError};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for PrivateMessage {
  type ApubType = Note;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let mut private_message = Note::new();

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

    private_message
      .set_context(activitystreams::context())
      .set_id(Url::parse(&self.ap_id.to_owned())?)
      .set_published(convert_datetime(self.published))
      .set_content(self.content.to_owned())
      .set_to(recipient.actor_id)
      .set_attributed_to(creator.actor_id);

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
impl FromApub for PrivateMessageForm {
  type ApubType = Note;

  async fn from_apub(
    note: &Note,
    context: &LemmyContext,
    expected_domain: Option<Url>,
  ) -> Result<PrivateMessageForm, LemmyError> {
    let creator_actor_id = note
      .attributed_to()
      .context(location_info!())?
      .clone()
      .single_xsd_any_uri()
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(&creator_actor_id, context).await?;
    let recipient_actor_id = note
      .to()
      .context(location_info!())?
      .clone()
      .single_xsd_any_uri()
      .context(location_info!())?;
    let recipient = get_or_fetch_and_upsert_user(&recipient_actor_id, context).await?;
    let ap_id = note.id_unchecked().context(location_info!())?.to_string();
    check_is_apub_id_valid(&Url::parse(&ap_id)?)?;

    Ok(PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content: note
        .content()
        .context(location_info!())?
        .as_single_xsd_string()
        .context(location_info!())?
        .to_string(),
      published: note.published().map(|u| u.to_owned().naive_local()),
      updated: note.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      read: None,
      ap_id: Some(check_object_domain(note, expected_domain)?),
      local: false,
    })
  }
}
