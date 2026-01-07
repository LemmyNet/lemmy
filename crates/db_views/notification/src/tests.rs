use crate::{NotificationData, NotificationView, impls::NotificationQuery};
use lemmy_db_schema::{
  assert_length,
  source::{
    comment::{Comment, CommentInsertForm},
    community::{Community, CommunityInsertForm},
    instance::Instance,
    modlog::{Modlog, ModlogInsertForm},
    notification::{Notification, NotificationInsertForm},
    person::{Person, PersonInsertForm},
    post::{Post, PostInsertForm},
    private_message::{PrivateMessage, PrivateMessageInsertForm},
  },
};
use lemmy_db_schema_file::enums::NotificationType;
use lemmy_diesel_utils::{
  connection::{DbPool, build_db_pool_for_tests},
  traits::Crud,
};
use lemmy_utils::error::LemmyResult;
use pretty_assertions::assert_eq;
use serial_test::serial;

struct Data {
  alice: Person,
  bob: Person,
}

async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
  let instance = Instance::read_or_create(pool, "my_domain.tld").await?;

  let alice_form = PersonInsertForm::test_form(instance.id, "alice2");
  let alice = Person::create(pool, &alice_form).await?;

  let bob_form = PersonInsertForm::test_form(instance.id, "bob2");
  let bob = Person::create(pool, &bob_form).await?;

  Ok(Data { alice, bob })
}

async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
  Instance::delete(pool, data.bob.instance_id).await?;
  Ok(())
}

#[tokio::test]
#[serial]
async fn test_private_message() -> LemmyResult<()> {
  let pool = &build_db_pool_for_tests();
  let pool = &mut pool.into();
  let data = init_data(pool).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(0, count);
  let notifs = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(0, notifs);

  let form = &PrivateMessageInsertForm::new(data.bob.id, data.alice.id, "my message".to_string());
  let pm = PrivateMessage::create(pool, form).await?;
  let form = NotificationInsertForm::new_private_message(&pm);
  Notification::create(pool, &[form]).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(1, count);
  let notifs = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(1, notifs);
  assert_eq!(Some(pm.id), notifs[0].notification.private_message_id);
  assert_eq!(pm.recipient_id, notifs[0].notification.recipient_id);
  assert!(!notifs[0].notification.read);
  let NotificationData::PrivateMessage(notif_pm) = &notifs[0].data else {
    panic!();
  };
  assert_eq!(pm, notif_pm.private_message);

  cleanup(data, pool).await
}

#[tokio::test]
#[serial]
async fn test_post() -> LemmyResult<()> {
  let pool = &build_db_pool_for_tests();
  let pool = &mut pool.into();
  let data = init_data(pool).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(0, count);
  let notifs = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(0, notifs);

  let community_form = CommunityInsertForm::new(
    data.alice.instance_id,
    "comm".to_string(),
    "title".to_string(),
    "pubkey".to_string(),
  );
  let community = Community::create(pool, &community_form).await?;

  let post_form = PostInsertForm::new("title".to_string(), data.bob.id, community.id);
  let post = Post::create(pool, &post_form).await?;

  let notif_form =
    NotificationInsertForm::new_post(post.id, data.alice.id, NotificationType::Subscribed);
  Notification::create(pool, &[notif_form]).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(1, count);
  let notifs1 = NotificationQuery::default().list(pool, &data.alice).await?;
  assert_length!(1, notifs1);
  assert_eq!(Some(post.id), notifs1[0].notification.post_id);
  assert!(!notifs1[0].notification.read);
  let NotificationData::Post(notif_post) = &notifs1[0].data else {
    panic!();
  };
  assert_eq!(post, notif_post.post);
  Notification::mark_read_by_id_and_person(pool, notifs1[0].notification.id, data.alice.id, true)
    .await?;
  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(0, count);

  // create a notification entry for removed post
  let mod_remove_post_form = ModlogInsertForm::mod_remove_post(data.bob.id, &post, true, "reason");
  let mod_remove_post = &Modlog::create(pool, &[mod_remove_post_form]).await?[0];
  let notif_form = NotificationInsertForm {
    modlog_id: Some(mod_remove_post.id),
    ..NotificationInsertForm::new(data.alice.id, NotificationType::ModAction)
  };
  Notification::create(pool, &[notif_form]).await?;

  let count = NotificationView::get_unread_count(pool, &data.alice, false).await?;
  assert_eq!(1, count);
  let notifs2 = NotificationQuery {
    unread_only: Some(true),
    ..Default::default()
  }
  .list(pool, &data.alice)
  .await?;
  assert_length!(1, notifs2);
  assert_eq!(Some(mod_remove_post.id), notifs2[0].notification.modlog_id);
  assert!(!notifs2[0].notification.read);
  let NotificationData::ModAction(notif_remove_post) = &notifs2[0].data else {
    panic!();
  };
  assert_eq!(mod_remove_post, &notif_remove_post.modlog);

  Notification::delete(pool, notifs1[0].notification.id).await?;
  Notification::delete(pool, notifs2[0].notification.id).await?;
  cleanup(data, pool).await
}

#[tokio::test]
#[serial]
async fn test_modlog() -> LemmyResult<()> {
  let pool = &build_db_pool_for_tests();
  let pool = &mut pool.into();
  let data = init_data(pool).await?;

  // create a community and post
  let form = CommunityInsertForm::new(
    data.alice.instance_id,
    "test".to_string(),
    "test".to_string(),
    String::new(),
  );
  let community = Community::create(pool, &form).await?;

  let form = PostInsertForm {
    ..PostInsertForm::new("123".to_string(), data.bob.id, community.id)
  };
  let post = Post::create(pool, &form).await?;

  let form = CommentInsertForm {
    removed: Some(true),
    ..CommentInsertForm::new(data.bob.id, post.id, String::new())
  };
  let comment = Comment::create(pool, &form, None).await?;

  // remove the comment and check notifs
  let form = ModlogInsertForm::mod_remove_comment(data.alice.id, &comment, true, "rule 1");
  let modlog = &Modlog::create(pool, &[form]).await?[0];

  let form = NotificationInsertForm {
    modlog_id: Some(modlog.id),
    ..NotificationInsertForm::new(data.bob.id, NotificationType::ModAction)
  };
  let notification = &Notification::create(pool, &[form]).await?[0];

  let notifs = NotificationQuery::default().list(pool, &data.bob).await?;
  assert_length!(1, notifs);
  let NotificationData::ModAction(m) = &notifs[0].data else {
    panic!();
  };
  assert_eq!(notification, &notifs[0].notification);
  assert_eq!(modlog, &m.modlog);
  assert_eq!(Some(data.alice.id), m.moderator.as_ref().map(|m| m.id));
  assert_eq!(Some(data.bob.id), m.target_person.as_ref().map(|p| p.id));
  assert_eq!(Some(comment.id), m.target_comment.as_ref().map(|c| c.id));

  cleanup(data, pool).await
}
