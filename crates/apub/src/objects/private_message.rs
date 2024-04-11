use super::verify_is_remote_object;
use crate::{
  check_apub_id_valid_with_strictness,
  objects::read_from_string_or_source,
  protocol::{
    objects::chat_message::{ChatMessage, ChatMessageType},
    Source,
  },
};
use activitypub_federation::{
  config::Data,
  protocol::{values::MediaTypeHtml, verification::verify_domains_match},
  traits::Object,
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{check_person_block, get_url_blocklist, local_site_opt_to_slur_regex, process_markdown},
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    person::Person,
    private_message::{PrivateMessage, PrivateMessageInsertForm},
  },
  traits::Crud,
};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType},
  utils::markdown::markdown_to_html,
};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubPrivateMessage(pub(crate) PrivateMessage);

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

#[async_trait::async_trait]
impl Object for ApubPrivateMessage {
  type DataType = LemmyContext;
  type Kind = ChatMessage;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      PrivateMessage::read_from_apub_id(&mut context.pool(), object_id)
        .await?
        .map(Into::into),
    )
  }

  async fn delete(self, _context: &Data<Self::DataType>) -> Result<(), LemmyError> {
    // do nothing, because pm can't be fetched over http
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn into_json(self, context: &Data<Self::DataType>) -> Result<ChatMessage, LemmyError> {
    let creator_id = self.creator_id;
    let creator = Person::read(&mut context.pool(), creator_id).await?;

    let recipient_id = self.recipient_id;
    let recipient = Person::read(&mut context.pool(), recipient_id).await?;

    let note = ChatMessage {
      r#type: ChatMessageType::ChatMessage,
      id: self.ap_id.clone().into(),
      attributed_to: creator.actor_id.into(),
      to: [recipient.actor_id.into()],
      content: markdown_to_html(&self.content),
      media_type: Some(MediaTypeHtml::Html),
      source: Some(Source::new(self.content.clone())),
      published: Some(self.published),
      updated: self.updated,
    };
    Ok(note)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    note: &ChatMessage,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> Result<(), LemmyError> {
    verify_domains_match(note.id.inner(), expected_domain)?;
    verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
    verify_is_remote_object(&note.id, context)?;

    check_apub_id_valid_with_strictness(note.id.inner(), false, context).await?;
    let person = note.attributed_to.dereference(context).await?;
    if person.banned {
      Err(LemmyErrorType::PersonIsBannedFromSite(
        person.actor_id.to_string(),
      ))?
    } else {
      Ok(())
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(
    note: ChatMessage,
    context: &Data<Self::DataType>,
  ) -> Result<ApubPrivateMessage, LemmyError> {
    let creator = note.attributed_to.dereference(context).await?;
    let recipient = note.to[0].dereference(context).await?;
    check_person_block(creator.id, recipient.id, &mut context.pool()).await?;

    let local_site = LocalSite::read(&mut context.pool()).await.ok();
    let slur_regex = &local_site_opt_to_slur_regex(&local_site);
    let url_blocklist = get_url_blocklist(context).await?;
    let content = read_from_string_or_source(&note.content, &None, &note.source);
    let content = process_markdown(&content, slur_regex, &url_blocklist, context).await?;

    let form = PrivateMessageInsertForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content,
      published: note.published.map(Into::into),
      updated: note.updated.map(Into::into),
      deleted: Some(false),
      read: None,
      ap_id: Some(note.id.into()),
      local: Some(false),
    };
    let pm = PrivateMessage::create(&mut context.pool(), &form).await?;
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
    },
    protocol::tests::file_to_json_object,
  };
  use assert_json_diff::assert_json_include;
  use lemmy_db_schema::source::site::Site;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn prepare_comment_test(
    url: &Url,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(ApubPerson, ApubPerson, ApubSite)> {
    let context2 = context.reset_request_count();
    let lemmy_person = file_to_json_object("assets/lemmy/objects/person.json")?;
    let site = parse_lemmy_instance(&context2).await?;
    ApubPerson::verify(&lemmy_person, url, &context2).await?;
    let person1 = ApubPerson::from_json(lemmy_person, &context2).await?;
    let pleroma_person = file_to_json_object("assets/pleroma/objects/person.json")?;
    let pleroma_url = Url::parse("https://queer.hacktivis.me/users/lanodan")?;
    ApubPerson::verify(&pleroma_person, &pleroma_url, &context2).await?;
    let person2 = ApubPerson::from_json(pleroma_person, &context2).await?;
    Ok((person1, person2, site))
  }

  async fn cleanup(
    data: (ApubPerson, ApubPerson, ApubSite),
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    Person::delete(&mut context.pool(), data.0.id).await?;
    Person::delete(&mut context.pool(), data.1.id).await?;
    Site::delete(&mut context.pool(), data.2.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_pm() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let url = Url::parse("https://enterprise.lemmy.ml/private_message/1621")?;
    let data = prepare_comment_test(&url, &context).await?;
    let json: ChatMessage = file_to_json_object("assets/lemmy/objects/chat_message.json")?;
    ApubPrivateMessage::verify(&json, &url, &context).await?;
    let pm = ApubPrivateMessage::from_json(json.clone(), &context).await?;

    assert_eq!(pm.ap_id.clone(), url.into());
    assert_eq!(pm.content.len(), 20);
    assert_eq!(context.request_count(), 0);

    let pm_id = pm.id;
    let to_apub = pm.into_json(&context).await?;
    assert_json_include!(actual: json, expected: to_apub);

    PrivateMessage::delete(&mut context.pool(), pm_id).await?;
    cleanup(data, &context).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_pleroma_pm() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let url = Url::parse("https://enterprise.lemmy.ml/private_message/1621")?;
    let data = prepare_comment_test(&url, &context).await?;
    let pleroma_url = Url::parse("https://queer.hacktivis.me/objects/2")?;
    let json = file_to_json_object("assets/pleroma/objects/chat_message.json")?;
    ApubPrivateMessage::verify(&json, &pleroma_url, &context).await?;
    let pm = ApubPrivateMessage::from_json(json, &context).await?;

    assert_eq!(pm.ap_id, pleroma_url.into());
    assert_eq!(pm.content.len(), 3);
    assert_eq!(context.request_count(), 0);

    PrivateMessage::delete(&mut context.pool(), pm.id).await?;
    cleanup(data, &context).await?;
    Ok(())
  }
}
