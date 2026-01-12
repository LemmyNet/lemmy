use crate::{
  objects::person::ApubPerson,
  protocol::tags::{ApubTag, Mention},
};
use activitypub_federation::{
  config::Data,
  fetch::webfinger::webfinger_resolve_actor,
  kinds::link::MentionType,
  traits::Object,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::{comment::Comment, person::Person, post::Post};
use lemmy_diesel_utils::{connection::DbPool, traits::Crud};
use lemmy_utils::{
  error::{LemmyResult, UntranslatedError},
  utils::mention::scrape_text_for_mentions,
};
use url::Url;

pub(crate) struct MentionsAndAddresses {
  pub ccs: Vec<Url>,
  pub mentions: Vec<ApubTag>,
}

/// This takes a markdown text, and builds a list of to_addresses, inboxes,
/// and mention tags, so they know where to be sent to.
/// Addresses are the persons / addresses that go in the cc field.
pub(crate) async fn collect_non_local_mentions(
  content: Option<&str>,
  parent_creator: Option<ApubPerson>,
  context: &Data<LemmyContext>,
) -> LemmyResult<MentionsAndAddresses> {
  let mut addressed_ccs: Vec<Url> = vec![];
  let mut mentions = vec![];
  if let Some(parent_creator) = parent_creator {
    addressed_ccs.push(parent_creator.id().clone());
    mentions.push(Mention {
      href: parent_creator.id().clone().into(),
      name: Some(format!(
        "@{}@{}",
        &parent_creator.name,
        &parent_creator
          .id()
          .domain()
          .ok_or(UntranslatedError::UrlWithoutDomain)?
      )),
      kind: MentionType::Mention,
    });
  }

  // Get the person IDs for any mentions
  let scraped = content
    .map(scrape_text_for_mentions)
    .into_iter()
    .flatten()
    // Filter only the non-local ones
    .filter(|m| !m.is_local(&context.settings().hostname));

  for mention in scraped {
    let identifier = format!("{}@{}", mention.name, mention.domain);
    let person = webfinger_resolve_actor::<LemmyContext, ApubPerson>(&identifier, context).await;
    if let Ok(person) = person {
      addressed_ccs.push(person.ap_id.to_string().parse()?);

      let mention_tag = Mention {
        href: person.id().clone().into(),
        name: Some(mention.full_name()),
        kind: MentionType::Mention,
      };
      mentions.push(mention_tag);
    }
  }

  Ok(MentionsAndAddresses {
    ccs: addressed_ccs,
    mentions: mentions.into_iter().map(ApubTag::Mention).collect(),
  })
}

/// Returns the apub ID of the person this comment is responding to. Meaning, in case this is a
/// top-level comment, the creator of the post, otherwise the creator of the parent comment.
pub(crate) async fn get_comment_parent_creator(
  pool: &mut DbPool<'_>,
  comment: &Comment,
) -> LemmyResult<ApubPerson> {
  let parent_creator_id = if let Some(parent_comment_id) = comment.parent_comment_id() {
    let parent_comment = Comment::read(pool, parent_comment_id).await?;
    parent_comment.creator_id
  } else {
    let parent_post_id = comment.post_id;
    let parent_post = Post::read(pool, parent_post_id).await?;
    parent_post.creator_id
  };
  Ok(Person::read(pool, parent_creator_id).await?.into())
}
