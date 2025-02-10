use crate::{
  check_apub_id_valid_with_strictness,
  fetcher::markdown_links::markdown_rewrite_remote_links,
  objects::read_from_string_or_source,
  protocol::{
    objects::private_message::{PrivateMessage, PrivateMessageType},
    Source,
  },
};
use activitypub_federation::{
  config::Data,
  protocol::{
    values::MediaTypeHtml,
    verification::{verify_domains_match, verify_is_remote_object},
  },
  traits::Object,
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{
    check_private_messages_enabled,
    get_url_blocklist,
    local_site_opt_to_slur_regex,
    process_markdown,
  },
};
use lemmy_db_schema::{
  source::{
    instance::Instance,
    local_site::LocalSite,
    person::Person,
    person_block::PersonBlock,
    private_message::{PrivateMessage as DbPrivateMessage, PrivateMessageInsertForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{FederationError, LemmyError, LemmyErrorType, LemmyResult},
  utils::markdown::markdown_to_html,
};
use semver::{Version, VersionReq};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubPrivateMessage(pub(crate) DbPrivateMessage);

impl Deref for ApubPrivateMessage {
  type Target = DbPrivateMessage;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<DbPrivateMessage> for ApubPrivateMessage {
  fn from(pm: DbPrivateMessage) -> Self {
    ApubPrivateMessage(pm)
  }
}

#[async_trait::async_trait]
impl Object for ApubPrivateMessage {
  type DataType = LemmyContext;
  type Kind = PrivateMessage;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    None
  }

  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    Ok(
      DbPrivateMessage::read_from_apub_id(&mut context.pool(), object_id)
        .await?
        .map(Into::into),
    )
  }

  async fn delete(self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    // do nothing, because pm can't be fetched over http
    Err(LemmyErrorType::NotFound.into())
  }

  async fn into_json(self, context: &Data<Self::DataType>) -> LemmyResult<PrivateMessage> {
    let creator_id = self.creator_id;
    let creator = Person::read(&mut context.pool(), creator_id).await?;

    let recipient_id = self.recipient_id;
    let recipient = Person::read(&mut context.pool(), recipient_id).await?;

    let instance = Instance::read(&mut context.pool(), recipient.instance_id).await?;
    let mut kind = PrivateMessageType::Note;

    // Deprecated: For Lemmy versions before 0.20, send private messages with old type
    if let (Some(software), Some(version)) = (instance.software, &instance.version) {
      let req = VersionReq::parse("<0.20")?;
      if software == "lemmy" && req.matches(&Version::parse(version)?) {
        kind = PrivateMessageType::ChatMessage
      }
    }

    let note = PrivateMessage {
      kind,
      id: self.ap_id.clone().into(),
      attributed_to: creator.ap_id.into(),
      to: [recipient.ap_id.into()],
      content: markdown_to_html(&self.content),
      media_type: Some(MediaTypeHtml::Html),
      source: Some(Source::new(self.content.clone())),
      published: Some(self.published),
      updated: self.updated,
    };
    Ok(note)
  }

  async fn verify(
    note: &PrivateMessage,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(note.id.inner(), expected_domain)?;
    verify_domains_match(note.attributed_to.inner(), note.id.inner())?;
    verify_is_remote_object(&note.id, context)?;

    check_apub_id_valid_with_strictness(note.id.inner(), false, context).await?;
    let person = note.attributed_to.dereference(context).await?;
    if person.banned {
      Err(FederationError::PersonIsBannedFromSite(
        person.ap_id.to_string(),
      ))?
    } else {
      Ok(())
    }
  }

  async fn from_json(
    note: PrivateMessage,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<ApubPrivateMessage> {
    let creator = note.attributed_to.dereference(context).await?;
    let recipient = note.to[0].dereference(context).await?;
    PersonBlock::read(&mut context.pool(), recipient.id, creator.id).await?;

    // Check that they can receive private messages
    if let Ok(recipient_local_user) =
      LocalUserView::read_person(&mut context.pool(), recipient.id).await
    {
      check_private_messages_enabled(&recipient_local_user)?;
    }
    let local_site = LocalSite::read(&mut context.pool()).await.ok();
    let slur_regex = &local_site_opt_to_slur_regex(&local_site);
    let url_blocklist = get_url_blocklist(context).await?;

    let content = read_from_string_or_source(&note.content, &None, &note.source);
    let content = process_markdown(&content, slur_regex, &url_blocklist, context).await?;
    let content = markdown_rewrite_remote_links(content, context).await;

    let form = PrivateMessageInsertForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content,
      published: note.published,
      updated: note.updated,
      deleted: Some(false),
      read: None,
      ap_id: Some(note.id.into()),
      local: Some(false),
    };
    let timestamp = note.updated.or(note.published).unwrap_or_else(Utc::now);
    let pm = DbPrivateMessage::insert_apub(&mut context.pool(), timestamp, &form).await?;
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
    (person1, person2, site): (ApubPerson, ApubPerson, ApubSite),
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    Person::delete(&mut context.pool(), person1.id).await?;
    Person::delete(&mut context.pool(), person2.id).await?;
    Site::delete(&mut context.pool(), site.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_pm() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let url = Url::parse("https://enterprise.lemmy.ml/private_message/1621")?;
    let data = prepare_comment_test(&url, &context).await?;
    let json: PrivateMessage = file_to_json_object("assets/lemmy/objects/private_message.json")?;
    ApubPrivateMessage::verify(&json, &url, &context).await?;
    let pm = ApubPrivateMessage::from_json(json.clone(), &context).await?;

    assert_eq!(pm.ap_id.clone(), url.into());
    assert_eq!(pm.content.len(), 20);
    assert_eq!(context.request_count(), 0);

    let pm_id = pm.id;
    let to_apub = pm.into_json(&context).await?;
    assert_json_include!(actual: json, expected: to_apub);

    DbPrivateMessage::delete(&mut context.pool(), pm_id).await?;
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

    DbPrivateMessage::delete(&mut context.pool(), pm.id).await?;
    cleanup(data, &context).await?;
    Ok(())
  }
}
