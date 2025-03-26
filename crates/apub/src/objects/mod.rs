use crate::protocol::{objects::page::Attachment, Source};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::values::MediaTypeMarkdownOrHtml,
};
use community::ApubCommunity;
use html2md::parse_html;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::community::{CommunityActions, CommunityModeratorForm},
  traits::Joinable,
};
use lemmy_db_views::structs::CommunityModeratorView;
use lemmy_utils::error::LemmyResult;
use person::ApubPerson;

pub mod comment;
pub mod community;
pub mod instance;
pub mod person;
pub mod post;
pub mod private_message;

pub(crate) fn read_from_string_or_source(
  content: &str,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> String {
  if let Some(s) = source {
    // markdown sent by lemmy in source field
    s.content.clone()
  } else if media_type == &Some(MediaTypeMarkdownOrHtml::Markdown) {
    // markdown sent by peertube in content field
    content.to_string()
  } else {
    // otherwise, convert content html to markdown
    parse_html(content)
  }
}

pub(crate) fn read_from_string_or_source_opt(
  content: &Option<String>,
  media_type: &Option<MediaTypeMarkdownOrHtml>,
  source: &Option<Source>,
) -> Option<String> {
  content
    .as_ref()
    .map(|content| read_from_string_or_source(content, media_type, source))
}

pub(crate) async fn append_attachments_to_comment(
  content: String,
  attachments: &[Attachment],
  context: &Data<LemmyContext>,
) -> LemmyResult<String> {
  let mut content = content;
  // Don't modify comments with no attachments
  if !attachments.is_empty() {
    content += "\n";
    for attachment in attachments {
      content = content + "\n" + &attachment.as_markdown(context).await?;
    }
  }

  Ok(content)
}

pub(crate) async fn handle_community_moderators(
  new_mods: &Vec<ObjectId<ApubPerson>>,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let community_id = community.id;
  let current_moderators =
    CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;
  // Remove old mods from database which arent in the moderators collection anymore
  for mod_user in &current_moderators {
    let mod_id = ObjectId::from(mod_user.moderator.ap_id.clone());
    if !new_mods.contains(&mod_id) {
      let community_moderator_form =
        CommunityModeratorForm::new(mod_user.community.id, mod_user.moderator.id);
      CommunityActions::leave(&mut context.pool(), &community_moderator_form).await?;
    }
  }

  // Add new mods to database which have been added to moderators collection
  for mod_id in new_mods {
    // Ignore errors as mod accounts might be deleted or instances unavailable.
    let mod_user: Option<ApubPerson> = mod_id.dereference(context).await.ok();
    if let Some(mod_user) = mod_user {
      if !current_moderators
        .iter()
        .any(|x| x.moderator.ap_id == mod_user.ap_id)
      {
        let community_moderator_form = CommunityModeratorForm::new(community.id, mod_user.id);
        CommunityActions::join(&mut context.pool(), &community_moderator_form).await?;
      }
    }
  }
  Ok(())
}
