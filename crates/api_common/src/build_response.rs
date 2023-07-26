use crate::{
  comment::CommentResponse,
  community::CommunityResponse,
  context::LemmyContext,
  post::PostResponse,
  utils::{
    check_person_block,
    get_interface_language,
    is_mod_or_admin,
    send_email_to_user,
    NotificationKind,
  },
};
use actix_web::web::Json;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, LocalUserId, PersonId, PostId},
  source::{
    actor_language::CommunityLanguage,
    comment::Comment,
    comment_reply::{CommentReply, CommentReplyInsertForm},
    person::Person,
    person_mention::{PersonMention, PersonMentionInsertForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::{CommentView, LocalUserView, PostView};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{error::LemmyError, utils::mention::MentionData};

pub async fn build_comment_response(
  context: &LemmyContext,
  comment_id: CommentId,
  local_user_view: Option<LocalUserView>,
  form_id: Option<String>,
  recipient_ids: Vec<LocalUserId>,
) -> Result<CommentResponse, LemmyError> {
  let person_id = local_user_view.map(|l| l.person.id);
  let comment_view = CommentView::read(&mut context.pool(), comment_id, person_id).await?;
  Ok(CommentResponse {
    comment_view,
    recipient_ids,
    form_id,
  })
}

pub async fn build_community_response(
  context: &LemmyContext,
  local_user_view: LocalUserView,
  community_id: CommunityId,
) -> Result<Json<CommunityResponse>, LemmyError> {
  let is_mod_or_admin =
    is_mod_or_admin(&mut context.pool(), local_user_view.person.id, community_id)
      .await
      .is_ok();
  let person_id = local_user_view.person.id;
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    Some(person_id),
    Some(is_mod_or_admin),
  )
  .await?;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}

pub async fn build_post_response(
  context: &LemmyContext,
  community_id: CommunityId,
  person_id: PersonId,
  post_id: PostId,
) -> Result<Json<PostResponse>, LemmyError> {
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), person_id, community_id)
    .await
    .is_ok();
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(person_id),
    Some(is_mod_or_admin),
  )
  .await?;
  Ok(Json(PostResponse { post_view }))
}

// TODO: this function is a mess and should be split up to handle email seperately
#[tracing::instrument(skip_all)]
pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment: &Comment,
  person: &Person,
  post: &Post,
  do_send_email: bool,
  context: &LemmyContext,
) -> Result<Vec<LocalUserId>, LemmyError> {
  let mut recipient_ids = Vec::new();
  let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&person.name))
  {
    let mention_name = mention.name.clone();
    let user_view = LocalUserView::read_from_name(&mut context.pool(), &mention_name).await;
    if let Ok(mention_user_view) = user_view {
      // TODO
      // At some point, make it so you can't tag the parent creator either

      // This can cause two notifications, one for reply and the other for mention
      recipient_ids.push(mention_user_view.local_user.id);

      let user_mention_form = PersonMentionInsertForm {
        recipient_id: mention_user_view.person.id,
        comment_id: comment.id,
        read: None,
      };

      // Allow this to fail softly, since comment edits might re-update or replace it
      // Let the uniqueness handle this fail
      PersonMention::create(&mut context.pool(), &user_mention_form)
        .await
        .ok();

      // Send an email to those local users that have notifications on
      if do_send_email {
        let lang = get_interface_language(&mention_user_view);
        send_email_to_user(
          &mention_user_view,
          &lang.notification_mentioned_by_subject(&person.name),
          &lang.notification_mentioned_by_body(&comment.content, &inbox_link, &person.name),
          context,
          NotificationKind::Mention,
        )
        .await
      }
    }
  }

  // Send comment_reply to the parent commenter / poster
  if let Some(parent_comment_id) = comment.parent_comment_id() {
    let parent_comment = Comment::read(&mut context.pool(), parent_comment_id).await?;

    // Get the parent commenter local_user
    let parent_creator_id = parent_comment.creator_id;

    // Only add to recipients if that person isn't blocked
    let creator_blocked = check_person_block(person.id, parent_creator_id, &mut context.pool())
      .await
      .is_err();

    // Don't send a notif to yourself
    if parent_comment.creator_id != person.id && !creator_blocked {
      let user_view = LocalUserView::read_person(&mut context.pool(), parent_creator_id).await;
      if let Ok(parent_user_view) = user_view {
        recipient_ids.push(parent_user_view.local_user.id);

        let comment_reply_form = CommentReplyInsertForm {
          recipient_id: parent_user_view.person.id,
          comment_id: comment.id,
          read: None,
        };

        // Allow this to fail softly, since comment edits might re-update or replace it
        // Let the uniqueness handle this fail
        CommentReply::create(&mut context.pool(), &comment_reply_form)
          .await
          .ok();

        if do_send_email {
          let lang = get_interface_language(&parent_user_view);
          send_email_to_user(
            &parent_user_view,
            &lang.notification_comment_reply_subject(&person.name),
            &lang.notification_comment_reply_body(&comment.content, &inbox_link, &person.name),
            context,
            NotificationKind::CommentReply,
          )
          .await
        }
      }
    }
  } else {
    // If there's no parent, its the post creator
    // Only add to recipients if that person isn't blocked
    let creator_blocked = check_person_block(person.id, post.creator_id, &mut context.pool())
      .await
      .is_err();

    if post.creator_id != person.id && !creator_blocked {
      let creator_id = post.creator_id;
      let parent_user = LocalUserView::read_person(&mut context.pool(), creator_id).await;
      if let Ok(parent_user_view) = parent_user {
        recipient_ids.push(parent_user_view.local_user.id);

        let comment_reply_form = CommentReplyInsertForm {
          recipient_id: parent_user_view.person.id,
          comment_id: comment.id,
          read: None,
        };

        // Allow this to fail softly, since comment edits might re-update or replace it
        // Let the uniqueness handle this fail
        CommentReply::create(&mut context.pool(), &comment_reply_form)
          .await
          .ok();

        if do_send_email {
          let lang = get_interface_language(&parent_user_view);
          send_email_to_user(
            &parent_user_view,
            &lang.notification_post_reply_subject(&person.name),
            &lang.notification_post_reply_body(&comment.content, &inbox_link, &person.name),
            context,
            NotificationKind::PostReply,
          )
          .await
        }
      }
    }
  }

  Ok(recipient_ids)
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    build_response::send_local_notifs,
    context::LemmyContext,
    test_utils::create_context,
  };
  use async_trait::async_trait;
  use lemmy_db_schema::{
    source::{
      community::Community,
      instance::Instance,
      local_user::{LocalUser, LocalUserUpdateForm},
      person::Person,
      post::Post,
    },
    test_utils::TestDao,
  };
  use lemmy_utils::{
    email::EmailSender,
    error::LemmyError,
    settings::structs::Settings,
    utils::mention::MentionData,
  };
  use serial_test::serial;
  use std::sync::{Arc, Mutex};

  #[derive(Debug, PartialEq, Clone)]
  struct FakeEmail {
    to_user: String,
    subject: String,
  }

  impl FakeEmail {
    pub fn new(to_user: &str, subject: &str) -> Self {
      FakeEmail {
        to_user: to_user.to_string(),
        subject: subject.to_string(),
      }
    }
  }

  struct FakeEmailSender {
    sent: Arc<Mutex<Vec<FakeEmail>>>,
  }

  impl FakeEmailSender {
    fn new() -> Self {
      FakeEmailSender {
        sent: Arc::new(Mutex::new(vec![])),
      }
    }
    fn reset(&self) {
      self.sent.lock().unwrap().clear();
    }
  }

  #[async_trait]
  impl EmailSender for FakeEmailSender {
    /// Stores the user/subject "sent" in a list
    async fn send(
      &self,
      subject: &str,
      _to_email: &str,
      to_username: &str,
      _html: &str,
      _settings: &Settings,
    ) -> Result<(), LemmyError> {
      self
        .sent
        .lock()
        .unwrap()
        .push(FakeEmail::new(to_username, subject));
      Ok(())
    }
  }

  struct TestLemmy {
    ctx: LemmyContext,
    email_sender: Arc<FakeEmailSender>,
    instance: Option<Instance>,
    community: Option<Community>,
  }

  impl TestDao for TestLemmy {
    fn pool(&self) -> lemmy_db_schema::utils::DbPool {
      self.ctx.pool()
    }
  }

  impl TestLemmy {
    pub async fn new() -> Self {
      let email_sender = Arc::new(FakeEmailSender::new());
      let ctx = create_context(email_sender.clone()).await;

      TestLemmy {
        ctx,
        email_sender,
        instance: None,
        community: None,
      }
    }

    pub async fn init(&mut self) {
      let instance = Instance::read_or_create(&mut self.ctx.pool(), "test_domain.tld".to_string())
        .await
        .unwrap();
      let community = self.create_community(&instance, "test_community").await;

      self.instance = Some(instance);
      self.community = Some(community);
    }

    pub async fn instance_user(&self, name: &str) -> (Person, LocalUser) {
      self
        .create_user(self.instance.as_ref().unwrap(), name)
        .await
    }

    pub async fn community_post(&self, person: &Person, name: &str) -> Post {
      self
        .create_post(person, self.community.as_ref().unwrap(), name)
        .await
    }

    pub fn mention(&self, mentioning_name: &str) -> MentionData {
      MentionData {
        name: mentioning_name.into(),
        domain: self.ctx.settings().hostname.clone(),
      }
    }

    pub fn sent_emails(&self) -> Vec<FakeEmail> {
      self.email_sender.sent.lock().unwrap().to_vec()
    }

    pub fn clear_emails(&self) {
      self.email_sender.reset();
    }
  }

  #[tokio::test]
  #[serial]
  async fn test_notifications() {
    let mut lemmy = TestLemmy::new().await;
    lemmy.init().await;

    let turn_notifications_on_form = LocalUserUpdateForm::builder()
      .send_notifications_to_email(Some(true))
      .build();
    let (bob, bob_user) = lemmy.instance_user("bob").await;
    let _bob_user = lemmy
      .update_user(&bob_user, &turn_notifications_on_form)
      .await;
    let (jim, jim_user) = lemmy.instance_user("jim").await;
    let _jim_user = lemmy
      .update_user(&jim_user, &turn_notifications_on_form)
      .await;
    let mentions = vec![lemmy.mention("jim")];
    let post = lemmy.community_post(&bob, "content").await;
    let parent_comment = lemmy.create_comment(&jim, &post, "content", None).await;
    let comment = lemmy
      .create_comment(&bob, &post, "content", Some(&parent_comment))
      .await;

    let _ = send_local_notifs(mentions, &comment, &bob, &post, true, &lemmy.ctx).await;

    let sent_emails_by_bob = lemmy.sent_emails();
    let expected_emails_by_bob: Vec<FakeEmail> = vec![
      FakeEmail::new("jim", "Mentioned by bob"),
      FakeEmail::new("jim", "Reply from bob"),
    ];
    assert_eq!(sent_emails_by_bob, expected_emails_by_bob);

    lemmy.clear_emails();
    assert_eq!(lemmy.sent_emails(), vec![]);

    let _ = send_local_notifs(vec![], &parent_comment, &jim, &post, true, &lemmy.ctx).await;

    let sent_emails_by_jim = lemmy.sent_emails();
    let expected_emails_by_jim: Vec<FakeEmail> = vec![FakeEmail::new("bob", "Reply from jim")];
    assert_eq!(sent_emails_by_jim, expected_emails_by_jim);
  }

  #[tokio::test]
  #[serial]
  async fn test_notifications_off() {
    let mut lemmy = TestLemmy::new().await;
    lemmy.init().await;

    let turn_notifications_off_form = LocalUserUpdateForm::builder()
      .send_notifications_to_email(Some(true))
      .send_notifications_for_post_replies(Some(false))
      .send_notifications_for_comment_replies(Some(false))
      .send_notifications_for_mentions(Some(false))
      .send_notifications_for_private_messages(Some(false))
      .build();
    let (bob, bob_user) = lemmy.instance_user("bob_off").await;
    let _bob_user = lemmy
      .update_user(&bob_user, &turn_notifications_off_form)
      .await;
    let (jim, jim_user) = lemmy.instance_user("jim_off").await;
    let _jim_user = lemmy
      .update_user(&jim_user, &turn_notifications_off_form)
      .await;
    let mentions = vec![lemmy.mention("jim_off")];
    let post = lemmy.community_post(&bob, "content").await;
    let parent_comment = lemmy.create_comment(&jim, &post, "content", None).await;
    let comment = lemmy
      .create_comment(&bob, &post, "content", Some(&parent_comment))
      .await;

    let _ = send_local_notifs(mentions, &comment, &bob, &post, true, &lemmy.ctx).await;
    let _ = send_local_notifs(vec![], &parent_comment, &jim, &post, true, &lemmy.ctx).await;

    assert_eq!(lemmy.sent_emails(), vec![]);
  }
}
