use crate::{
  check_apub_id_valid_with_strictness,
  local_instance,
  objects::read_from_string_or_source,
  protocol::{
    objects::chat_message::{ChatMessage, ChatMessageType},
    Source,
  },
};
use activitypub_federation::{
  core::object_id::ObjectId,
  deser::values::MediaTypeHtml,
  traits::ApubObject,
  utils::verify_domains_match,
};
use chrono::NaiveDateTime;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::{
    person::Person,
    private_message::{PrivateMessage, PrivateMessageForm},
  },
  traits::Crud,
};
use lemmy_utils::{
  error::LemmyError,
  utils::{convert_datetime, markdown_to_html},
};
use lemmy_websocket::LemmyContext;
use std::ops::Deref;
use url::Url;

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
    ApubPrivateMessage(pm)
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubPrivateMessage {
  type DataType = LemmyContext;
  type ApubType = ChatMessage;
  type DbType = PrivateMessage;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  #[tracing::instrument(skip_all)]
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

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, context: &LemmyContext) -> Result<ChatMessage, LemmyError> {
    let creator_id = self.creator_id;
    let creator = blocking(context.pool(), move |conn| Person::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient =
      blocking(context.pool(), move |conn| Person::read(conn, recipient_id)).await??;

    let note = ChatMessage {
      r#type: ChatMessageType::ChatMessage,
      id: ObjectId::new(self.ap_id.clone()),
      attributed_to: ObjectId::new(creator.actor_id),
      to: [ObjectId::new(recipient.actor_id)],
      content: markdown_to_html(&self.content),
      media_type: Some(MediaTypeHtml::Html),
      source: Some(Source::new(self.content.clone())),
      published: Some(convert_datetime(self.published)),
      updated: self.updated.map(convert_datetime),
    };
    Ok(note)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    note: &ChatMessage,
    expected_domain: &Url,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(note.id.inner(), expected_domain)?;
    verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
    check_apub_id_valid_with_strictness(note.id.inner(), false, context.settings())?;
    let person = note
      .attributed_to
      .dereference(context, local_instance(context), request_counter)
      .await?;
    if person.banned {
      return Err(LemmyError::from_message("Person is banned from site"));
    }
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    note: ChatMessage,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubPrivateMessage, LemmyError> {
    let creator = note
      .attributed_to
      .dereference(context, local_instance(context), request_counter)
      .await?;
    let recipient = note.to[0]
      .dereference(context, local_instance(context), request_counter)
      .await?;

    let form = PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content: read_from_string_or_source(&note.content, &None, &note.source),
      published: note.published.map(|u| u.naive_local()),
      updated: note.updated.map(|u| u.naive_local()),
      deleted: None,
      read: None,
      ap_id: Some(note.id.into()),
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
  use crate::{
    objects::{
      instance::{tests::parse_lemmy_instance, ApubSite},
      person::ApubPerson,
      tests::init_context,
    },
    protocol::tests::file_to_json_object,
  };
  use assert_json_diff::assert_json_include;
  use lemmy_db_schema::source::site::Site;
  use serial_test::serial;

  async fn prepare_comment_test(
    url: &Url,
    context: &LemmyContext,
  ) -> (ApubPerson, ApubPerson, ApubSite) {
    let lemmy_person = file_to_json_object("assets/lemmy/objects/person.json").unwrap();
    let site = parse_lemmy_instance(context).await;
    ApubPerson::verify(&lemmy_person, url, context, &mut 0)
      .await
      .unwrap();
    let person1 = ApubPerson::from_apub(lemmy_person, context, &mut 0)
      .await
      .unwrap();
    let pleroma_person = file_to_json_object("assets/pleroma/objects/person.json").unwrap();
    let pleroma_url = Url::parse("https://queer.hacktivis.me/users/lanodan").unwrap();
    ApubPerson::verify(&pleroma_person, &pleroma_url, context, &mut 0)
      .await
      .unwrap();
    let person2 = ApubPerson::from_apub(pleroma_person, context, &mut 0)
      .await
      .unwrap();
    (person1, person2, site)
  }

  fn cleanup(data: (ApubPerson, ApubPerson, ApubSite), context: &LemmyContext) {
    Person::delete(&*context.pool().get().unwrap(), data.0.id).unwrap();
    Person::delete(&*context.pool().get().unwrap(), data.1.id).unwrap();
    Site::delete(&*context.pool().get().unwrap(), data.2.id).unwrap();
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_pm() {
    let context = init_context();
    let url = Url::parse("https://enterprise.lemmy.ml/private_message/1621").unwrap();
    let data = prepare_comment_test(&url, &context).await;
    let json: ChatMessage = file_to_json_object("assets/lemmy/objects/chat_message.json").unwrap();
    let mut request_counter = 0;
    ApubPrivateMessage::verify(&json, &url, &context, &mut request_counter)
      .await
      .unwrap();
    let pm = ApubPrivateMessage::from_apub(json.clone(), &context, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(pm.ap_id.clone(), url.into());
    assert_eq!(pm.content.len(), 20);
    assert_eq!(request_counter, 0);

    let pm_id = pm.id;
    let to_apub = pm.into_apub(&context).await.unwrap();
    assert_json_include!(actual: json, expected: to_apub);

    PrivateMessage::delete(&*context.pool().get().unwrap(), pm_id).unwrap();
    cleanup(data, &context);
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_pleroma_pm() {
    let context = init_context();
    let url = Url::parse("https://enterprise.lemmy.ml/private_message/1621").unwrap();
    let data = prepare_comment_test(&url, &context).await;
    let pleroma_url = Url::parse("https://queer.hacktivis.me/objects/2").unwrap();
    let json = file_to_json_object("assets/pleroma/objects/chat_message.json").unwrap();
    let mut request_counter = 0;
    ApubPrivateMessage::verify(&json, &pleroma_url, &context, &mut request_counter)
      .await
      .unwrap();
    let pm = ApubPrivateMessage::from_apub(json, &context, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(pm.ap_id, pleroma_url.into());
    assert_eq!(pm.content.len(), 3);
    assert_eq!(request_counter, 0);

    PrivateMessage::delete(&*context.pool().get().unwrap(), pm.id).unwrap();
    cleanup(data, &context);
  }
}
